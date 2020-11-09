use itertools::Itertools;
use time_calc::TimeSig;

use crate::map::{get_perc_map, RESOLUTION};
use crate::utils::{
  div_rem_usize,
  normalize_velocity,
  normalize_offset
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Drum {
  pub time: u32,
  pub velocity: u8,
  pub key: u8
}
pub struct DrumTrack {
  pub events: Vec<Drum>,
  pub time_signature: (u8, u8, u8, u8),
  pub ppqn: u16
}

impl Clone for DrumTrack {
  fn clone(&self) -> DrumTrack {
    let ev = self.events.iter().cloned().collect();
    DrumTrack {
      events: ev,
      time_signature: self.time_signature,
      ppqn: self.ppqn
    }
  }
}

impl DrumTrack {
  pub fn new(events: Vec<Drum>, time_signature: (u8, u8, u8, u8), ppqn: u16) -> DrumTrack {
    let mut ts = time_signature;

    if ts.0 == 0 {
      ts.0 = 4;
    }
    if ts.1 == 0 {
      ts.1 = 4;
    }
    
    DrumTrack { 
      events,
      time_signature: ts,
      ppqn
    }
  }

  pub fn get_key_footprint(&self) -> Vec<u8> {
    self.events
      .iter()
      .map(|e| e.key)
      .unique()
      .sorted()
      .collect()
  }

  pub fn get_track_perc_map(&self) -> Vec<Option<u8>>{
    let perc_map = get_perc_map();

    perc_map
      .iter()
      .map(|perc_group| self.get_key_for_group(perc_group))
      .collect()
  }

  pub fn get_bar_track_duration(&self) -> usize {
    let ts = TimeSig { 
      top: self.time_signature.0 as u16,
      bottom: self.time_signature.1 as u16
    };

    // println!("get_bar_track_duration TS {} / {} PPQN: {}", self.time_signature.0 as u16, self.time_signature.1 as u16, self.ppqn as u32);

    ts.ticks_per_bar(self.ppqn as u32).ticks() as usize
  }

  pub fn get_step_track_duration(&self) -> usize {
    let bar_tick_duration = self.get_bar_track_duration();
    bar_tick_duration / RESOLUTION
  }

  pub fn to_grid(&self, perc_map: &Vec<Option<u8>>) -> Vec<Vec<Vec<Vec<f32>>>> {
    let unwrapped_perc_map: Vec<u8> = perc_map
      .iter()
      .map(|option_key| match option_key {
        Some(key) => *key,
        None => 0 // we don't care about the 0 pitch 
      })
      .collect();

    // duration of a step (bar / RESOLUTION) in ticks  
    let step_tick_duration = self.get_step_track_duration();
    // minimum of distance between 2 events on a same step
    let minimum_distance = step_tick_duration / 10;

    // last event of the track, since track is already sorted, this gives us the length of our grid vector
    let last_event: &Drum = self.events.last().unwrap();
    let (event_len, _): (usize, usize) = self.get_step_index_offset_tuple(last_event, step_tick_duration);

    // calculate grid len in multiples of RESOLUTION
    let mut bars_number = event_len / RESOLUTION;
    if event_len % RESOLUTION > 0 {
      bars_number += 1
    }
    let grid_len = bars_number * RESOLUTION;
    
    // data structure to be filled from track events
    let mut grid: Vec<[Option<[isize; 2]>; 10]> = vec![[None; 10]; grid_len];

    // parsing and filling the grid 
    self.events
      .iter()
      .filter(|drum| unwrapped_perc_map.contains(&drum.key))
      .for_each(|drum| {
        let perc_index  = unwrapped_perc_map.iter().position(|&key| key == drum.key).unwrap();
        let (grid_index, offset) = self.get_step_index_offset_tuple(drum, step_tick_duration);  
        let event_payload = Some([offset as isize, drum.velocity as isize]);

        match &grid[grid_index][perc_index] {
          Some(event) => {
            if event[0] < 0 { 
              grid[grid_index][perc_index] = event_payload;
            } else if offset > 0 && offset as isize - event[0] > minimum_distance as isize && grid_index < grid_len - 1 {
              match grid[grid_index + 1][perc_index] {
                None => grid[grid_index + 1][perc_index] = Some([offset as isize - step_tick_duration as isize, drum.velocity as isize]),
                _ => {}
              }
            }
          },
          None => grid[grid_index][perc_index] = event_payload
        }
      });

    grid[..]
      .chunks_exact(RESOLUTION as usize)
      .unique()
      .map(|chunk| {
        chunk
          .iter()
          .map(|&step| {
            step
              .iter()
              .map(|option_perc| match option_perc {
                  Some(step_payload) => {
                    return vec![
                      normalize_offset(step_payload[0], step_tick_duration),
                      normalize_velocity(step_payload[1] as usize)
                    ]
                  },
                  None => return vec![0., 0.]
              })
              .collect()
          })
          .collect()
      })
      .collect()
  }

  fn get_key_for_group(&self, perc_group: &Vec<u8>) -> Option<u8> {
    let key_footprint = self.get_key_footprint();
  
    let has_any_key_for_group = perc_group
      .iter()
      .any(|key| key_footprint.contains(key));
  
    if has_any_key_for_group {
      let key_events_sorted_by_occurences: Vec<(u8, usize)> = perc_group
        .into_iter()
        .map(|&key| (key, self.events.iter().filter(|e| e.key == key).count()))
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .collect();

      let max_nb_of_events = key_events_sorted_by_occurences[0].1;

      let key_events: Vec<u8> = key_events_sorted_by_occurences
        .iter()
        .filter(|(_, count)| *count == max_nb_of_events)
        .map(|(key, _)| *key)
        .collect();

      let &chosen_key = perc_group
        .iter()
        .find(|&&key| key_events.contains(&key))
        .unwrap();
  
      return Some(chosen_key);
    }
  
    None
  }

  fn get_step_index_offset_tuple(&self, event: &Drum, step_tick_duration: usize) -> (usize, usize) {
    let (quotient, rest) = div_rem_usize(event.time as usize, step_tick_duration);
    // let offset = rest as f32 / step_tick_duration as f32;
    (quotient, rest)
  }
}
