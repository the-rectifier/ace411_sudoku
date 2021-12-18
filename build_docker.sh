#!/usr/bin/bash

docker build -t canopus/rust_builder . && docker run --rm -v $(pwd):/root/build --name rusty canopus/rust_builder 
