FROM scratch
COPY \
  target/x86_64-unknown-linux-musl/release/links-server \
  target/x86_64-unknown-linux-musl/release/settings.toml \
  ./
ENTRYPOINT ["./links-server"]
EXPOSE 7880/tcp

