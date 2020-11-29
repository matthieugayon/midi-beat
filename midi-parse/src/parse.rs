use midly::Smf;
use itertools::Itertools;

use crate::datatypes::DrumTrack;

use crate::utils::{
  track_has_beat_event,
  filter_beat_events,
  get_unique_time_signature,
  key_footprints_intersect,
  // to_smf
};

pub fn filter_beat(smf: Smf) -> Vec<DrumTrack> {
  // println!("Midi parsing file has {} tracks", smf.tracks.len());

  let mut ppqn: u16 = 0;

  match smf.header.timing {
    midly::Timing::Metrical(tpb) => {
      // println!("Midi timing is in ticks per beat {}", tpb.as_int());
      ppqn = tpb.as_int();
    }
    midly::Timing::Timecode(_, _) => {}
  }

  println!("number of tracks {:?}", smf.tracks.iter().len());

  // keeping and filtering tracks with channel 9 events
  let tracks: Vec<DrumTrack> = smf.tracks.iter()
    .filter(|track| track_has_beat_event(track))
    .map(|track| filter_beat_events(track, ppqn))
    .collect();

  println!("number of tracks after first filtering {:?}", tracks.iter().len());

  let unique_time_signature = get_unique_time_signature(&tracks);

  println!("unique_time_signature {:?}", unique_time_signature);

  // call merge_same_signature_tracks with tracks featuring same time_signature
  let merged_drum_tracks: Vec<DrumTrack> = unique_time_signature
    .iter()
    .map(|time_signature| {
      let same_signature_tracks: Vec<DrumTrack> = tracks
        .iter()
        .filter(|track| &track.time_signature == time_signature)
        .cloned()
        .collect();
        
      merge_same_signature_tracks(same_signature_tracks, *time_signature, ppqn)  
    })
    .flatten()
    .collect();

  // re write smf files just for debug  
  // to_smf(smf, &merged_drum_tracks, path);

  merged_drum_tracks
}


fn merge_same_signature_tracks(mut tracks: Vec<DrumTrack>, time_signature: (u8, u8, u8, u8), ppqn: u16) -> Vec<DrumTrack> {
  let mut mergeable_tracks: Vec<DrumTrack> = vec![];
  let mut resulting_tracks: Vec<DrumTrack> = vec![];

  // get widest key footprint 
  let index_of_widest_key_footprint: usize = tracks
    .iter()
    .map(|track| track.get_key_footprint())
    .position_max_by(|x, y| x.len().cmp(&y.len())).unwrap();

  // get associated DrumTrack 
  let base_track = &tracks[index_of_widest_key_footprint].clone();
  // push it straight into mergeable tracks Vec
  mergeable_tracks.push(base_track.clone());

  // remove it form the tracks to be tested against itself
  tracks.remove(index_of_widest_key_footprint);

  // iterate over
  for track in tracks {
    let base_key_footprint = base_track.get_key_footprint();
    let key_footprint = track.get_key_footprint();
    let key_footprint_len = key_footprint.len();

    if !key_footprints_intersect(base_key_footprint, key_footprint) {
      mergeable_tracks.push(track)
    } else if key_footprint_len > 1 {
      // only keep track if there is more than one pitch
      resulting_tracks.push(track)
    }
  }

  /* merge tracks which have exclusively different key footprint */
  let base_drum_track =  DrumTrack::new(vec![], time_signature, ppqn);

  let merged_track = mergeable_tracks
    .iter()
    .fold(base_drum_track,|acc, track| {
      DrumTrack::new([acc.events, track.clone().events].concat(), acc.time_signature, acc.ppqn)
    });
  
  resulting_tracks.push(merged_track);

  resulting_tracks
}
