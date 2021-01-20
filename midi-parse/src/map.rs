use itertools::Itertools;
use ndarray::{Array, Ix4, Ix3, Ix2, Ix1, ArrayView, ShapeError};
use crate::datatypes::DrumTrack;
use drawille::Canvas;

pub const RESOLUTION: usize = 32;
pub const NUMBER_OF_TRACKS: usize = 10;

pub fn get_perc_map() -> [Vec<u8>; NUMBER_OF_TRACKS] {
  [
    // KICK
    vec![35, 36],
    // SNARE / RIMS
    vec![38, 40, 37, 39],
    // TOMS
    vec![41, 45], // low
    vec![47, 48], // mid
    vec![43, 50], // high
    vec![61, 64, 66, 76, 78, 79, 68, 74, 75, 56, 58, 71, 72], // low percs african
    vec![60, 62, 63, 65, 67, 69, 54, 70, 73, 77, 81], // high percs african
    // HH / CYMB
    vec![42, 44], // muted hh
    vec![46, 55], // open hat / splash
    vec![49, 52, 57, 51, 53, 59] // rides / crash
  ]
}

pub fn process_track_pool(track_pool: &Vec<DrumTrack>) -> Result<Array<f32, Ix4>, ShapeError> {
  let flattened_bars: Vec<[[[f32; 2]; 10]; RESOLUTION]> = track_pool
    .into_iter()
    .map(|track| {
      println!("TS : {} {}", track.time_signature.0, track.time_signature.1);

      return track;
    })
    .map(|track| (track, track.get_track_perc_map()))
    // filter tracks with less than 2 mapped percs
    .filter(|(_, track_perc_map)| {
      let percs_number = track_perc_map
        .into_iter()  
        .filter(|option_key| match option_key {
          Some(_) => true,
          None => false
        })
        .count();

      percs_number > 2
    })
    // filter tracks whose TS not compatible with bar RESOLUTION of 32
    .filter(|(track, _)| track.get_bar_track_duration() % RESOLUTION == 0)
    // map to a Vec of bars 
    .map(|(track, track_perc_map)| {
      return track
        .to_grid(&track_perc_map)
    })
    // flatten everything into a vec of bars 
    .flatten()
    .unique_by(|bar| {
      let mut quantized_bar = [[[0 as isize; 2]; 10]; RESOLUTION];
      bar.iter()
        .enumerate()
        .for_each(|(step_index, step)| {
          step
            .iter()
            .enumerate()
            .for_each(|(perc_index, event)| {
              // juste need to reverse here because of the final ML format
                // @WARN
              quantized_bar[step_index][perc_index] = [(event[1] * 128.) as isize, (event[0] * 256.) as isize];
            })
        });
      quantized_bar
    })
    .collect();

  let number_of_bars = flattened_bars.len();

  let flattened_data: Vec<f32> = flattened_bars
    .iter()
    .flatten()
    .flatten()
    .flatten()
    .map(|val| *val)
    .collect();
  
  Array::from_shape_vec((number_of_bars, 32, 10, 2), flattened_data)
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

  array
    .outer_iter()
    .for_each(|arr3: ArrayView<f32, Ix3>| {
      // canvas.line(0, bar_index * BAR_HEIGHT, BAR_WIDTH, bar_index * BAR_HEIGHT);
      println!("bar_index {}", bar_index);
      let mut step_index: u32 = 0;

      arr3
        .outer_iter()
        .for_each(|arr2: ArrayView<f32, Ix2>| {    
          let mut perc_index: u32 = 0;

          arr2
            .outer_iter()
            .rev()
            .for_each(|arr1: ArrayView<f32, Ix1>| {
              match arr1.as_slice() {
                Some(step) => {
                  let offset = step[0];
                  let velocity = step[1];
                  let step_x =  step_index * STEP_WIDTH + step_index/4 * (PADDING * 2);
                  let step_y =  bar_index * (BAR_HEIGHT + 20) + (perc_index + 1) * TRACK_HEIGHT;
                  let step_height = (STEP_HEIGHT as f32 * velocity) as u32 ;
                  let step_offset = ((STEP_WIDTH - 2*PADDING) as f32 * offset) as u32;

                  // draw step marker 
                  canvas.set(step_x, step_y);

                  // 2 lines wide
                  if velocity > 0. {
                    canvas.line(step_x + step_offset + PADDING, step_y - step_height, step_x + step_offset + PADDING, step_y);
                    canvas.line(step_x + step_offset + PADDING + 1, step_y - step_height, step_x + step_offset + PADDING + 1, step_y);
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



