FROM rust:latest AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release


FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/decloneify .

CMD ["./decloneify"]