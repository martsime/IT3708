#!/bin/zsh

DATA_NUMBER=7

export DATA_NUMBER=$DATA_NUMBER

cargo run --release

# Open the last created image
LAST_IMAGE=$(ls -lt images/* | head -1 | awk '{print $NF}')
echo $LAST_IMAGE
feh $LAST_IMAGE

