#!/bin/bash

LIB="genetic.so"
PROGRAM="mainqt.py"

# Env variables
export RAYON_NUM_THREADS=11

# Build library
cargo build --release

# Move library
mv target/release/lib$LIB $LIB

# Run python
python $PROGRAM
