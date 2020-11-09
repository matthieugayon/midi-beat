use std::collections::BTreeMap;
use itertools::Itertools;

use crate::datatypes::DrumTrack;

#[allow(dead_code)]
pub fn fill_stats(tracks: &Vec<DrumTrack>, mut count: u64, key_map: &mut BTreeMap<u8, u64>, mut ts_count: u64, ts_map: &mut BTreeMap<(u8, u8, u8, u8), u64>) {
  tracks
      .iter()
      .map(|track| track.get_key_footprint())
      .flatten()
      .for_each(|key| {
          if key_map.contains_key(&key) {
              count = key_map.get(&key).unwrap() + 1;
              key_map.insert(key, count);
          } else {
              key_map.insert(key, 1);
          }
      });
     
  tracks
      .iter()
      .map(|track| track.time_signature)
      .for_each(|ts| {
          if ts_map.contains_key(&ts) {
              ts_count = ts_map.get(&ts).unwrap() + 1;
              ts_map.insert(ts, ts_count);
          } else {
              ts_map.insert(ts, 1);
          }
      });
}

#[allow(dead_code)]
pub fn display_stats(key_map: &BTreeMap<u8, u64>, ts_map: &BTreeMap<(u8, u8, u8, u8), u64>, counter: u32) {
  key_map
      .iter()
      .for_each(|(key, value)|{
          print!("[{}]: {} | ", key, value);
      });

  println!("");
  println!("--------------- most used keys");

  key_map
      .iter()
      .sorted_by(|a, b| b.1.cmp(&a.1))
      .take(12)
      .sorted_by(|a, b| b.0.cmp(&a.0))
      .for_each(|(key, value)|{
          println!("KEY [{}]: {} | ", key, value);
      });

  println!("");
  println!("--------------- time signatures");

  ts_map
      .iter()
      .for_each(|(ts, value)|{
          println!("TS [{}/{} , {}, {}]: {} | ", ts.0, ts.1, ts.2, ts.3, value);
      });

  println!("====> {} files were corrupted", counter);
}