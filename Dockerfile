FROM rust:slim as builder
WORKDIR /usr/src/maldness_bot
RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev

RUN USER=root cargo init

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./diesel.toml diesel.toml
COPY ./migrations ./migrations
COPY ./src ./src
RUN touch -a -m ./src/main.rs
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/maldness_bot /usr/local/bin/maldness_bot
CMD ["maldness_bot"]
