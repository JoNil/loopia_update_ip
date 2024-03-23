FROM rust:latest as builder

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/loopia_update_ip
COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /usr/src/loopia_update_ip/target/x86_64-unknown-linux-musl/release/loopia_update_ip .

ENTRYPOINT ["./loopia_update_ip"]
