use glob::glob_with;
use glob::MatchOptions;
use midly::Smf;
use ndarray_npy::NpzWriter;
use std::collections::BTreeMap;
use std::fs::File;
use std::{fs, time::Instant};
use structopt::StructOpt;

use midi_parse::datatypes::DrumTrack;
use midi_parse::map::process_track_pool;
use midi_parse::parse::filter_beat;
use midi_parse::stats::{display_stats, fill_stats, filter_densities, filter_gridicity};

use ndarray::s;

// parse args in a clean struct
#[derive(Debug, StructOpt)]
#[structopt(name = "parser-cli", about = "MIDI beat Dataset Builder")]
struct Opt {
    /// Filter on channel 9 only (drum GM midi)
    #[structopt(short, long)]
    drum_channel: bool,
    /// Input path
    #[structopt(short, long)]
    input: String,
    /// Output path
    #[structopt(short, long)]
    output: String,
}

fn main() {
    // read options
    let opt = Opt::from_args();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    // task time elapsed
    let start = Instant::now();
    let mut counter: u32 = 0;

    // for stats
    let mut key_map: BTreeMap<u8, u64> = BTreeMap::new();
    let mut ts_map: BTreeMap<(u8, u8, u8, u8), u64> = BTreeMap::new();
    let count: u64 = 1;
    let ts_count: u64 = 1;

    // track pool
    let mut track_pool: Vec<DrumTrack> = Vec::new();

    if opt.drum_channel {
        println!("Filter on Channel 10 only ....");
    } else {
        println!("No Channel filtering");
    }

    println!("Reading files in : {}", opt.input);

    match glob_with(&opt.input, options) {
        Ok(paths) => {
            paths.into_iter().for_each(|path| {
                match path {
                    Ok(path) => {
                        // println!("Parsing file: {}", path.display());
                        let data = fs::read(path.as_path()).unwrap();

                        // Parse the raw bytes
                        match Smf::parse(&data) {
                            Ok(smf) => {
                                let mut tracks = filter_beat(smf, opt.drum_channel);
                                fill_stats(&tracks, count, &mut key_map, ts_count, &mut ts_map);
                                track_pool.append(&mut tracks);
                            }
                            Err(e) => {
                                println!("SMF parsing error: {}", e);
                                counter += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Path error: {}", e);
                    }
                }
            });
        }
        Err(e) => {
            println!("Pattern error: {}", e);
        }
    }

    display_stats(&key_map, &ts_map, counter);

    match process_track_pool(&track_pool) {
        Ok(array) => {
            println!(
                "Successful cast of bars vec into Array4, shape: {:?}",
                array.shape()
            );

            // filter densities
            match filter_densities(&array) {
                Ok(filtered) => {
                    println!("Filtered shape: {:?}", filtered.shape());
                    let size_limit = 2_000_000; // Adjust this as needed
                    let num_chunks = (filtered.shape()[0] + size_limit - 1) / size_limit;

                    for i in 0..num_chunks {
                        let start = i * size_limit;
                        let end = start + size_limit.min(filtered.shape()[0] - start);
                        let chunk = filtered.slice(s![start..end, .., .., ..]);

                        let output_path = format!("{}_{}.npz", opt.output, i);

                        let mut npz = NpzWriter::new_compressed(
                            File::create(&output_path).expect("Output path error"),
                        );

                        npz.add_array("x", &chunk).expect("Can't write our array");

                        println!("Successfully generated NPZ for path: '{}'", output_path);
                    }
                }
                Err(e) => {
                    println!("Shape error: {}", e);
                }
            }
        }
        Err(err) => {
            println!("Shape error: {}", err);
        }
    }

    let round = |num: f64| (num * 100.0).round() / 100.0;
    let time = round((start.elapsed().as_micros() as f64) / 1000.0);

    println!("Process lasted for {} ms", time);
}
