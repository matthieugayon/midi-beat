use glob::glob_with;
use midly::Smf;
use std::{fs, time::Instant};
use structopt::StructOpt;
use std::collections::BTreeMap;
use glob::MatchOptions;

use midi_parse::parse::filter_beat;
use midi_parse::datatypes::DrumTrack;
use midi_parse::stats::{fill_stats, display_stats};
use midi_parse::map::process_track_pool;

// parse args in a clean struct
#[derive(Debug, StructOpt)]
#[structopt(name = "parser-cli", about = "MIDI beat Dataset Builder")]
struct Opt {
    /// Input path
    #[structopt(short, long)]
    input: String,
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
                                let mut tracks = filter_beat(smf);
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
        Ok(_array) => {
            println!("Successful cast of bars vec into Array4 =>");
        }
        Err(err) => {
            println!("Shape error: {}", err);
        }
    }

    let round = |num: f64| (num * 100.0).round() / 100.0;
    let time = round((start.elapsed().as_micros() as f64) / 1000.0);

    println!("Process lasted for {} ms", time);
}

