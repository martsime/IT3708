#!/bin/bash

source envconf

cd profiler

# Build binary
cargo build --release

# Profile it
perf record --call-graph dwarf target/release/profiler

# Show report
perf report
