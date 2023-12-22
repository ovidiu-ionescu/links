#!/bin/bash

docker run \
    -p 7880:3000 \
    --name links-server \
    -d --restart unless-stopped \
    --mount type=bind,source=/home/ovidiu/Projects/Rust/links/data,target=/data \
    --user $(id -u ovidiu):$(id -g ovidiu) \
    -e RUST_LOG=info \
    links-server:latest
