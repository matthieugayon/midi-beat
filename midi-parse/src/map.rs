use std::collections::HashMap;

use crate::datatypes::DrumTrack;
use drawille::Canvas;
use itertools::Itertools;
use ndarray::{Array, ArrayView, Ix1, Ix2, Ix3, Ix4, ShapeError};

pub const RESOLUTION: usize = 32;
pub const NUMBER_OF_TRACKS: usize = 8;
pub const THRESH_NON_EMPTY_TRACKS: usize = 0; // 0 means

pub fn get_perc_map() -> [Vec<u8>; NUMBER_OF_TRACKS] {
    [
        // KICK
        vec![35, 36],
        // SNARE / RIMS
        vec![37, 38, 39, 40],

        // TOMS
        vec![41, 45, 61, 64, 66], // low tom
        vec![47, 48, 58, 60, 63, 68, 74, 77, 78, 79], // mid tom
        vec![43, 50, 54, 56, 62, 65, 67, 69, 70, 71, 72, 73, 75, 76, 81], // high tom

        // HH / CYMB
        vec![42, 44],  // muted hh
        vec![46, 49, 55, 57], // open hat / splash / crash
        vec![51, 52, 53, 59, 80], // ride
    ]
}

pub fn get_alt_reverse_perc_map() -> HashMap<u8, Vec<usize>> {
    let mut rmap: HashMap<u8, Vec<usize>> = HashMap::new();

    // kicks can go into the low perc group
    rmap.insert(35, vec![2]);
    rmap.insert(36, vec![2]);

    // snares can go into the mid and high perc group
    rmap.insert(37, vec![4, 3]);
    rmap.insert(38, vec![4, 3]);
    rmap.insert(39, vec![4, 3]);
    rmap.insert(40, vec![4, 3]);

    // low percs can go into the kick group, or mid and high perc group
    rmap.insert(41, vec![0, 3, 4]);
    rmap.insert(45, vec![0, 3, 4]);
    rmap.insert(61, vec![0, 3, 4]);
    rmap.insert(64, vec![0, 3, 4]);
    rmap.insert(66, vec![0, 3, 4]);

    // mid percs can go into the snare group, or low and high perc group
    rmap.insert(47, vec![4, 2, 1]);
    rmap.insert(48, vec![4, 2, 1]);
    rmap.insert(58, vec![4, 2, 1]);
    rmap.insert(60, vec![4, 2, 1]);
    rmap.insert(63, vec![4, 2, 1]);
    rmap.insert(68, vec![4, 2, 1]);
    rmap.insert(74, vec![4, 2, 1]);
    rmap.insert(77, vec![4, 2, 1]);
    rmap.insert(78, vec![4, 2, 1]);
    rmap.insert(79, vec![4, 2, 1]);

    // high percs can go into the snare group, or low and mid perc group
    rmap.insert(43, vec![3, 2, 1]);
    rmap.insert(50, vec![3, 2, 1]);
    rmap.insert(54, vec![3, 2, 1]);
    rmap.insert(56, vec![3, 2, 1]);
    rmap.insert(62, vec![3, 2, 1]);
    rmap.insert(65, vec![3, 2, 1]);
    rmap.insert(67, vec![3, 2, 1]);
    rmap.insert(69, vec![3, 2, 1]);
    rmap.insert(70, vec![3, 2, 1]);
    rmap.insert(71, vec![3, 2, 1]);
    rmap.insert(72, vec![3, 2, 1]);
    rmap.insert(73, vec![3, 2, 1]);
    rmap.insert(75, vec![3, 2, 1]);
    rmap.insert(76, vec![3, 2, 1]);
    rmap.insert(81, vec![3, 2, 1]);

    // muted hh are flexible
    rmap.insert(42, vec![6, 7, 4, 3, 1]);
    rmap.insert(44, vec![6, 7, 4, 3, 1]);

    // open hat / splash / crash are flexible
    rmap.insert(46, vec![5, 7, 4, 3, 1]);
    rmap.insert(49, vec![5, 7, 4, 3, 1]);
    rmap.insert(55, vec![5, 7, 4, 3, 1]);
    rmap.insert(57, vec![5, 7, 4, 3, 1]);

    // ridoids is flexible
    rmap.insert(51, vec![5, 6, 4, 3, 1]);
    rmap.insert(52, vec![5, 6, 4, 3, 1]);
    rmap.insert(53, vec![5, 6, 4, 3, 1]);
    rmap.insert(59, vec![5, 6, 4, 3, 1]);
    rmap.insert(80, vec![5, 6, 4, 3, 1]);


    rmap
}

pub fn process_track_pool(track_pool: &Vec<DrumTrack>) -> Result<Array<f32, Ix4>, ShapeError> {
    let flattened_bars: Vec<[[[f32; 2]; NUMBER_OF_TRACKS]; RESOLUTION]> = track_pool
        .into_iter()
        .map(|track| {
            return track;
        })
        .map(|track| (track, track.get_track_perc_map()))

        // filter tracks with less than 1 mapped percs
        .filter(|(_, track_perc_map)| {
            let percs_number = track_perc_map
                .into_iter()
                .filter(|option_key| match option_key {
                    Some(_) => true,
                    None => false,
                })
                .count();

            percs_number > THRESH_NON_EMPTY_TRACKS
        })

        // @TODO switch to 96 ???
        // filter tracks whose TS not compatible with bar RESOLUTION of 32
        .filter(|(track, _)| track.get_bar_track_duration() % RESOLUTION == 0)
        // filter TS only 4/4
        // @TODO will need other TS
        .filter(|(track, _)| {
            return (track.time_signature.0 == 4 && track.time_signature.1 == 4)
                || (track.time_signature.0 == 4 && track.time_signature.1 == 2)
                || (track.time_signature.0 == 2 && track.time_signature.1 == 2);
        })
        // map to a Vec of bars
        .map(|(track, track_perc_map)| return track.to_grid(&track_perc_map))
        // flatten everything into a vec of bars
        .flatten()
        .unique_by(|bar| {
            let mut quantized_bar = [[0 as isize; NUMBER_OF_TRACKS]; RESOLUTION];
            bar.iter().enumerate().for_each(|(step_index, step)| {
                step.iter().enumerate().for_each(|(perc_index, event)| {
                    // @TODO augment the quantization ??
                    quantized_bar[step_index][perc_index] =
                        (event[0] * 2.) as isize;
                })
            });
            quantized_bar
        })
        .collect();

    let number_of_bars = flattened_bars.len();

    // special filtering operation
    // used to shape datasets better
    // like "select only four to floor for techno"
    // and other properties

    let flattened_data: Vec<f32> = flattened_bars
        .iter()
        .flatten()
        .flatten()
        .flatten()
        .map(|val| *val)
        .collect();

    Array::from_shape_vec((number_of_bars, RESOLUTION, NUMBER_OF_TRACKS, 2), flattened_data)
}

// old terminal display
#[allow(dead_code)]
fn draw_array(array: Array<f32, Ix4>) {
    let mut canvas = Canvas::new(200, 100);

    const STEP_HEIGHT: u32 = 4;
    const PADDING: u32 = 2;
    const TRACK_HEIGHT: u32 = STEP_HEIGHT + PADDING;
    const STEP_WIDTH: u32 = 8;
    const BAR_HEIGHT: u32 = NUMBER_OF_TRACKS as u32 * TRACK_HEIGHT;

    let mut bar_index: u32 = 0;

    array.outer_iter().for_each(|arr3: ArrayView<f32, Ix3>| {
        // canvas.line(0, bar_index * BAR_HEIGHT, BAR_WIDTH, bar_index * BAR_HEIGHT);
        println!("bar_index {}", bar_index);
        let mut step_index: u32 = 0;

        arr3.outer_iter().for_each(|arr2: ArrayView<f32, Ix2>| {
            let mut perc_index: u32 = 0;

            arr2.outer_iter()
                .rev()
                .for_each(|arr1: ArrayView<f32, Ix1>| {
                    match arr1.as_slice() {
                        Some(step) => {
                            let offset = step[0];
                            let velocity = step[1];
                            let step_x = step_index * STEP_WIDTH + step_index / 4 * (PADDING * 2);
                            let step_y =
                                bar_index * (BAR_HEIGHT + 20) + (perc_index + 1) * TRACK_HEIGHT;
                            let step_height = (STEP_HEIGHT as f32 * velocity) as u32;
                            let step_offset = ((STEP_WIDTH - 2 * PADDING) as f32 * offset) as u32;

                            // draw step marker
                            canvas.set(step_x, step_y);

                            // 2 lines wide
                            if velocity > 0. {
                                canvas.line(
                                    step_x + step_offset + PADDING,
                                    step_y - step_height,
                                    step_x + step_offset + PADDING,
                                    step_y,
                                );
                                canvas.line(
                                    step_x + step_offset + PADDING + 1,
                                    step_y - step_height,
                                    step_x + step_offset + PADDING + 1,
                                    step_y,
                                );
                            }
                        }
                        None => {
                            println!("!PANIC")
                        }
                    }

                    perc_index += 1;
                });

            step_index += 1;
        });

        bar_index += 1;
    });

    println!("{}", canvas.frame());
}


// has kick every beat filter
fn has_kick_every_beat(patt: [[[f32; 2]; NUMBER_OF_TRACKS]; RESOLUTION]) -> bool {
    false
}
