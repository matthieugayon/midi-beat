# midi-beat


## Dataset creation (from .mid / .midi)

`cargo run --bin parser-cli -- -i "absolute/path/to/folder/**/*.midi" -o path/to/output.npz`

## display test

`cargo run --bin display-test -- -i "your/file/path"`
 
## data filtering

`cargo run --bin data-filter -- --input ~/Desktop/real_batter.npz --output ~/Desktop/filt.npz --num-samples 5000`