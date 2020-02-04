#!/bin/bash

source envconf

cd profiler

cargo build --release

valgrind --tool=callgrind target/release/profiler
