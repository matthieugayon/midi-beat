use itertools::Itertools;
use midly::num::u28;
use midly::num::u4;
use midly::num::u7;
use midly::MidiMessage;
use midly::Smf;
use midly::TrackEvent;
use midly::TrackEventKind;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::datatypes::{Drum, DrumTrack};

pub fn track_has_beat_event(track: &Vec<TrackEvent>) -> bool {
    track.iter().any(|&e| match e.kind {
        TrackEventKind::Midi {
            channel,
            message: _,
        } => return channel.as_int() == 9,
        _ => return false,
    })
}

pub fn filter_beat_events(track: &Vec<TrackEvent>, ppqn: u16, drum_channel: bool) -> DrumTrack {
    let mut delta_count: u32 = 0;
    let mut time_signature: (u8, u8, u8, u8) = (4, 4, 0, 0);
    
    // Temporary struct to store note-on events waiting for their note-off
    let mut note_on_events: HashMap<u8, (u32, u8)> = HashMap::new();
    let mut drum_events: Vec<Drum> = Vec::new();
    
    // Default duration in ticks (quarter note)
    let default_duration: u32 = ppqn as u32;

    // First pass: collect all events and find note pairs
    track
        .into_iter()
        .take_while(|e| {
            let test_counter: u64 = e.delta.as_int() as u64;
            test_counter < 100_000
        })
        .for_each(|e| {
            delta_count += e.delta.as_int();

            match e.kind {
                TrackEventKind::Midi { channel, message } => {
                    if channel.as_int() == 9 || !drum_channel {
                        match message {
                            midly::MidiMessage::NoteOn { key, vel } => {
                                if vel.as_int() > 0 {
                                    // Store note-on event with time and velocity
                                    note_on_events.insert(key.as_int(), (delta_count, vel.as_int()));
                                } else {
                                    // A note-on with velocity 0 is effectively a note-off
                                    if let Some((start_time, velocity)) = note_on_events.remove(&key.as_int()) {
                                        let duration = delta_count - start_time;
                                        drum_events.push(Drum {
                                            time: start_time,
                                            key: key.as_int(),
                                            velocity: velocity,
                                            duration: duration,
                                        });
                                    }
                                }
                            },
                            midly::MidiMessage::NoteOff { key, vel: _ } => {
                                // Handle note-off event
                                if let Some((start_time, velocity)) = note_on_events.remove(&key.as_int()) {
                                    let duration = delta_count - start_time;
                                    drum_events.push(Drum {
                                        time: start_time,
                                        key: key.as_int(),
                                        velocity: velocity,
                                        duration: duration,
                                    });
                                }
                            },
                            _ => {}
                        }
                    }
                }
                TrackEventKind::Meta(meta_message) => {
                    match meta_message {
                        midly::MetaMessage::TimeSignature(
                            numerator,
                            denominator,
                            midi_clocks_per_click,
                            notes_per_quarter,
                        ) => {
                            time_signature = (
                                numerator,
                                denominator,
                                midi_clocks_per_click,
                                notes_per_quarter,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });

    // Handle any remaining note-on events that didn't get a note-off
    for (key, (start_time, velocity)) in note_on_events {
        drum_events.push(Drum {
            time: start_time,
            key,
            velocity,
            duration: default_duration,
        });
    }
    
    // Sort events by time
    drum_events.sort_by_key(|e| e.time);

    // scale according to timesig
    // 4/4 -> x1
    // 4/2 -> x2
    if time_signature.1 > 0 {
        let stretch = time_signature.0 / time_signature.1;
        if stretch > 1 {
          for evt in drum_events.iter_mut() {
            evt.time *= stretch as u32;
            evt.duration *= stretch as u32;
          };
        }
    }

    DrumTrack {
        events: drum_events,
        time_signature,
        ppqn,
    }
}

pub fn get_unique_time_signature(tracks: &Vec<DrumTrack>) -> Vec<(u8, u8, u8, u8)> {
    tracks
        .iter()
        .map(|track| track.time_signature)
        .unique()
        .collect()
}

// could be better but ... who cares
pub fn key_footprints_intersect(kfp_a: Vec<u8>, kfp_b: Vec<u8>) -> bool {
    let mut counter: u32 = 0;
    kfp_a.iter().for_each(|key_a| {
        kfp_b.iter().for_each(|key_b| {
            if key_b == key_a {
                counter += 1
            }
        })
    });

    counter > 0
}

// turn smf original instance with modified tracks into midi file
#[allow(dead_code)]
pub fn to_smf(mut smf: Smf, tracks: &Vec<DrumTrack>, path: PathBuf) {
    smf.tracks = tracks
        .iter()
        .map(|track| {
            let mut counter: u32 = 0;

            return track
                .events
                .clone()
                .into_iter()
                .map(|drum| {
                    return vec![
                        TrackEvent {
                            delta: u28::try_from(drum.time).unwrap(),
                            kind: TrackEventKind::Midi {
                                channel: u4::try_from(9).unwrap(),
                                message: MidiMessage::NoteOn {
                                    key: u7::try_from(drum.key).unwrap(),
                                    vel: u7::try_from(120).unwrap(),
                                },
                            },
                        },
                        TrackEvent {
                            delta: u28::try_from(drum.time + drum.duration).unwrap(),
                            kind: TrackEventKind::Midi {
                                channel: u4::try_from(9).unwrap(),
                                message: MidiMessage::NoteOff {
                                    key: u7::try_from(drum.key).unwrap(),
                                    vel: u7::try_from(0).unwrap(),
                                },
                            },
                        },
                    ];
                })
                .flatten()
                .sorted_by(|&a, b| a.delta.as_int().cmp(&b.delta.as_int()))
                .map(|mut e| {
                    // println!("Midi reconstructed event {}", e.delta.as_int());
                    let int_delta = e.delta.as_int();
                    e.delta = u28::try_from(int_delta - counter).unwrap();
                    counter = int_delta;
                    e
                })
                .collect();
        })
        .collect();

    let prepend_str: &str = "./generated_";
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_name = format!("{}{}", prepend_str, file_name);
    let new_path = Path::new(&new_name);

    smf.save(new_path).unwrap();
}

pub fn div_rem<T: std::ops::Div<Output = T> + std::ops::Rem<Output = T> + Copy>(
    x: T,
    y: T,
) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

pub fn div_rem_usize(x: usize, y: usize) -> (usize, usize) {
    div_rem(x, y)
}

pub fn normalize_velocity(vel: usize) -> f32 {
    vel as f32 / 127.
}

pub fn normalize_offset(ticks_offset: isize, step_tick_duration: usize) -> f32 {
    ticks_offset as f32 / step_tick_duration as f32
}

pub fn normalize_duration(duration_ticks: u32, bar_duration_ticks: u32) -> f32 {
    (duration_ticks as f32 / bar_duration_ticks as f32).min(1.0)
}