use itertools::Itertools;
use ndarray::{array, Array, Ix4, ShapeError, s};
use std::collections::BTreeMap;

use crate::{datatypes::DrumTrack, map::RESOLUTION, map::NUMBER_OF_TRACKS};

#[allow(dead_code)]
pub fn fill_stats(
    tracks: &Vec<DrumTrack>,
    mut count: u64,
    key_map: &mut BTreeMap<u8, u64>,
    mut ts_count: u64,
    ts_map: &mut BTreeMap<(u8, u8, u8, u8), u64>,
) {
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
pub fn display_stats(
    key_map: &BTreeMap<u8, u64>,
    ts_map: &BTreeMap<(u8, u8, u8, u8), u64>,
    counter: u32,
) {
    key_map.iter().for_each(|(key, value)| {
        print!("[{}]: {} | ", key, value);
    });

    println!("");
    println!("--------------- most used keys");

    key_map
        .iter()
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .take(12)
        .sorted_by(|a, b| b.0.cmp(&a.0))
        .for_each(|(key, value)| {
            println!("KEY [{}]: {} | ", key, value);
        });

    println!("");
    println!("--------------- time signatures");

    ts_map.iter().for_each(|(ts, value)| {
        println!("TS [{}/{} , {}, {}]: {} | ", ts.0, ts.1, ts.2, ts.3, value);
    });

    println!("====> {} files were corrupted", counter);
}

#[allow(dead_code)]
pub fn filter_densities(bars_array: &Array<f32, Ix4>) -> Result<Array<f32, Ix4>, ShapeError> {
    // pure vec with filtered results
    let mut res: Vec<f32> = vec![];
    let mut filtered_ct = 0;

    let mut highest_dens = 0.0;
    let mut accum = 0.0;

    for bar in bars_array.outer_iter() {
        // remove offset information to calculate density
        let vel_only = bar.slice(s![.., .., 0]);
        let density = vel_only.mean().unwrap();

        if density > 0.003 && density < 0.3 { 

            if density > highest_dens {
                highest_dens = density;
            }

            accum += density;

            filtered_ct += 1;
            let v = bar.to_slice().unwrap();
            res.extend_from_slice(v);
        }
    }

    println!("kept: {}, highest density {}, avg density {}", filtered_ct, highest_dens, accum / filtered_ct as f32);
    Array::from_shape_vec((filtered_ct, RESOLUTION, NUMBER_OF_TRACKS, 2), res)
}

pub fn filter_gridicity(bars_array: &Array<f32, Ix4>) -> Result<Array<f32, Ix4>, ShapeError> {
    // pure vec with filtered results
    let mut res: Vec<f32> = vec![];
    let mut filtered_ct = 0;

    let mut velocity_kernel: Vec<f32> = Vec::new();

    // kick
    velocity_kernel.extend_from_slice(&[1.0, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
        1.0, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
        1.0, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
        1.0, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1
    ]);

    // 7 times, extends
    for _ in 0..7 {
        velocity_kernel.extend_from_slice(&[0.75, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
            0.75, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
            0.75, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1, 
            0.75, 0.1, 0.25, 0.1, 0.75, 0.1, 0.25, 0.1
        ]);
    }

    let velocity_kernel = Array::from_shape_vec((NUMBER_OF_TRACKS, RESOLUTION, 1), velocity_kernel).unwrap();

    // permute 0 and 1 axis
    let velocity_kernel = velocity_kernel.permuted_axes([1, 0, 2]);

    // println!("velocity kernel shape: {:?}", velocity_kernel.shape());

    // print the kernel
    // println!("velocity kernel: {:?}", velocity_kernel);
    for bar in bars_array.outer_iter() {
        // get vels and offsets only
        let vel_only = bar.slice(s![.., .., 0..1]).to_owned();
        let offs_only = bar.slice(s![.., .., 1..2]).to_owned();

        // count how many events are in vel_only > 0.05
        let vel_evts = &vel_only.mapv(|v| {
            if v > 0.05 {
                1.0f32
            } else {
                0.0f32
            }
        });
        let vel_ct = vel_evts.sum();

        // apply kernel on vels
        let vel_conv = vel_only * &velocity_kernel;

        // apply a penalty for offsets
        let offs_only = 1.0 - offs_only.mapv(f32::abs);

        let vel_conv = vel_conv * offs_only;

        let gridicity = vel_conv.sum() / vel_ct;

        if gridicity > 0.19 && gridicity < 0.9 {
            filtered_ct += 1;
            let v = bar.to_slice().unwrap();
            res.extend_from_slice(v);
        }
    }

    println!("kept: {}, rejected: {}", filtered_ct, bars_array.shape()[0] - filtered_ct);
    Array::from_shape_vec((filtered_ct, RESOLUTION, NUMBER_OF_TRACKS, 2), res)
}