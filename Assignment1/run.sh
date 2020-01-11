#!/bin/bash

LIB="genetic.so"
PROGRAM="mainqt.py"

# Build library
cargo build --release

# Move library
mv target/release/lib$LIB $LIB

# Run python
python $PROGRAM
