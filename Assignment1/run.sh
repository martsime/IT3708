#!/bin/bash

LIB="pygenetic.so"
PROGRAM="mainqt.py"

############
# SETTINGS #
############

# Paths
source envconf

# Build library
cargo build --release

# Move library
mv target/release/lib$LIB $LIB

# Run python
python $PROGRAM
