#!/bin/sh

DATA_NUMBER=7

export DATA_NUMBER=$DATA_NUMBER

feh image-$DATA_NUMBER.png &

cargo run --release


