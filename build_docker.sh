#!/usr/bin/bash

set -e

docker build -t canopus/rust_builder . 

docker run --rm \
    -v $(pwd):/tmp/build \
    --name rusty \
    canopus/rust_builder 
