use kmeans::*;
use ndarray::{Array, Array3, Axis, Ix4, Ix3, s};
use ndarray_npy::{NpzReader, NpzWriter};
use rand::Rng;
use std::{fs::File};
use structopt::StructOpt;

// parse args in a clean struct
#[derive(Debug, StructOpt)]
#[structopt(name = "data-filter", about = "filter datasets for maximum diversity")]
struct Opt {
    /// Input path
    #[structopt(short, long)]
    input: String,
    /// Output path
    #[structopt(short, long)]
    output: String,
    /// Subgroup len
    #[structopt(short, long)]
    num_samples: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read options
    let opt = Opt::from_args();

    let mut npz = NpzReader::new(File::open(opt.input)?)?;

    let a: ndarray::Array<f32, Ix4> = npz.by_name("x")?;
    let ashape = a.shape();

    println!("dataset shape {:?}, size {}", a.shape(), a.len());

    if opt.num_samples > a.shape()[0] / 2 {
        panic!("dataset is too small");
    }

    // remove offsets
    let vel_only = a.slice(s![.., .., .., 0]).to_owned();

    let (sample_cnt, sample_dims, k, max_iter) = (ashape[0], ashape[1] * ashape[2], 100, 100); // 100 modes

    let kmean = KMeans::new(
        vel_only.as_slice().unwrap().to_vec(),
        sample_cnt,
        sample_dims,
    );

    let result = kmean.kmeans_lloyd(
        k,
        max_iter,
        KMeans::init_kmeanplusplus,
        &KMeansConfig::default(),
    );

    println!("Centroids: {:?}", result.centroids.len());

    println!("Cluster-Assignments: {:?}", result.assignments.len());

    // println!("how many samples per group do we need ? {:?}", );
    let samples_per_mode = opt.num_samples / 100;

    let mut selecta : Vec<f32> = Vec::new();

    for i in 0..100usize {
        let mut ct = 0;

        let mut rng = rand::thread_rng();

        while ct < samples_per_mode {
            // randomly select an index
            let mut idx: usize = rng.gen_range(0, result.assignments.len());

            while *result.assignments.get(idx).unwrap() as usize != i {
                idx = rng.gen_range(0, result.assignments.len());
            }

            let candidate = a.index_axis(Axis(0), idx);

            selecta.extend_from_slice(candidate.to_slice().unwrap());
            // selecta.push(candidate.to_owned());
            ct += 1;
        }
    }

    // convert to proper 
    println!("new set len {}", selecta.len());

    let res = Array::from_shape_vec((opt.num_samples, 32, 10, 2), selecta).unwrap();

    println!("res shape {:?}", res.shape());

    let mut npz = NpzWriter::new(File::create(opt.output.clone()).expect("Output path error"));
    npz.add_array("x", &res).expect("Can't write our array");
    println!("Successfully generated NPZ for path: '{}'", opt.output);

    Ok(())
}
