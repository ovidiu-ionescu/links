#!/bin/bash

#cargo build --release --target x86_64-unknown-linux-musl

# saw it recommended here https://github.com/sfackler/rust-openssl/issues/603
# source here: https://github.com/clux/muslrust
# can't run it other than root, it can't find cargo

sudo echo "Build a new docker image and container"

docker run  --rm -t \
	-v $PWD:/volume \
	clux/muslrust \
	cargo build --release

USER=$(whoami)
sudo chown -R $USER:$USER target
NAME=links-server
strip target/x86_64-unknown-linux-musl/release/$NAME
cp settings-prod.toml target/x86_64-unknown-linux-musl/release/settings.toml

ID=$(cargo pkgid | awk -F '#' '{ print $2 }')

docker stop $NAME
docker rm $NAME

docker image rm $NAME:$ID

docker build --tag $NAME:$ID .

docker run \
  -p 7880:7880 \
  --name $NAME \
  -d --restart unless-stopped \
  --mount type=bind,source=/home/ovidiu/Projects/Links/Data,target=/data \
  --user $(id -u ovidiu):$(id -g ovidiu) \
  -e RUST_LOG=info \
  $NAME:$ID
