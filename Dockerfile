# This file is generated by docker-build
# DO NOT EDIT BY HAND;
# Edit docker-build instead.
FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/myapp
# maybe use --link
COPY . .
LABEL stage="builder"
RUN apt-get update && apt-get install -y \
  libssl-dev \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*
RUN cargo build --release
### links-server
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/links-server /usr/local/bin/links-server
# maybe embed in the executable
COPY html html
# maybe put in configmap
COPY settings.toml settings.toml
MAINTAINER Ovidiu Ionescu <ovidiu@ionescu.net>
LABEL maintainer="Ovidiu Ionescu <ovidiu@ionescu.net>" \
  version="0.1.2" \
  tag="links-server:v0.1.2" \
  description="A server for managing lists of urls." \
  repository="https://github.com/ovidiu-ionescu/links-server" \
  name="links-server"\
  app="links-server"

#ENTRYPOINT ["/usr/local/bin/links-server"]
EXPOSE 8080/tcp 8081/tcp

CMD ["links-server"]

