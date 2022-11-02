# midi-beat


## Dataset creation (from .mid / .midi)

`cargo run --bin parser-cli -- -i "absolute/path/to/folder/**/*.midi" -o path/to/output.npz`

## data filtering

`cargo run --bin data-filter -- --input ~/Desktop/real_batter.npz --output ~/Desktop/filt.npz --num-samples 5000`

## display test

`cargo run --bin display-test -- -i "your/file/path"`


## Snipps

`cargo run --bin parser-cli --release -- -i "/Users/nunja2/Documents/Datasets/midi/GROSDRUM/**/*.mid"  -o ~/Desktop/reddit.npz`

`cargo run --bin parser-cli --release -- -i "/Users/nunja2/Documents/Datasets/midi/BIGTURKISH/**/*.mid"  -o ~/Desktop/turk.npz`
 