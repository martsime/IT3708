#!/bin/bash

LIB="genetic.so"
PROGRAM="mainqt.py"

# Env variables
export RAYON_NUM_THREADS=11
export DRAW_RATE=1

# SETTINGS
export POPULATION_SIZE=1000
export ELITE_COUNT=2
export MUTATION_RATE=0.05
export MUTATION_NUM_MAX=5
export CROSSOVER_RATE=1.0
export PARENT_SELECTION_K=5

# Build library
cargo build --release

# Move library
mv target/release/lib$LIB $LIB

# Run python
python $PROGRAM
