#!/bin/bash

LIB="genetic.so"
PROGRAM="mainqt.py"

############
# SETTINGS #
############

# Paths
BASE_PATH=data
PROBLEM_NUMBER=05
export PROBLEM_PATH=$BASE_PATH/problems/p$PROBLEM_NUMBER
export LOAD_SOLUTION=true
export SOLUTION_PATH=$BASE_PATH/solutions/p$PROBLEM_NUMBER.res

export RAYON_NUM_THREADS=12
export DRAW_RATE=1

export POPULATION_SIZE=10000
export POPULATION_GEN_STEP=100
export ELITE_COUNT=2

# Mutations
export SINGLE_SWAP_MUT_RATE=0.1
export SINGLE_SWAP_MUT_MAX=5

export VEHICLE_REMOVE_MUT_RATE=0.1
export VEHICLE_REMOVE_MUT_MAX=1

# Crossover
export CROSSOVER_RATE=1.0
export PARENT_SELECTION_K=10

export INFEASIBILITY_PENALTY=1000
export CWS_BIAS=2

# Build library
cargo build --release

# Move library
mv target/release/lib$LIB $LIB

# Run python
python $PROGRAM
