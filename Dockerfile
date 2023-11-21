# Build the links-server image with using a two stage Dockerfile.

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
RUN cargo install wasm-pack; cd links-wasm; wasm-pack build --target web

### Finished building, assemble the final image for links-server
FROM debian:bookworm-slim
ARG version
#RUN echo "Oh dang look at that ${version}"
RUN apt-get update && apt-get install -y \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/links-server /usr/local/bin/links-server
# maybe embed in the executable
COPY html html
# get the wasm files
COPY --from=builder /usr/src/myapp/links-wasm/pkg/links_wasm_bg.wasm /usr/src/myapp/links-wasm/pkg/links_wasm.js html
# maybe put in configmap
COPY settings.toml settings.toml
MAINTAINER Ovidiu Ionescu <ovidiu@ionescu.net>
LABEL maintainer="Ovidiu Ionescu <ovidiu@ionescu.net>" \
  version="${version}" \
  tag="links-server:v${version}" \
  description="A server for managing lists of urls." \
  repository="https://github.com/ovidiu-ionescu/links-server" \
  name="links-server"\
  app="links-server"

#ENTRYPOINT ["/usr/local/bin/links-server"]
EXPOSE 8080/tcp 8081/tcp

CMD ["links-server"]

