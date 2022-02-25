FROM rust:slim

WORKDIR /usr/src/aot-backend

RUN apt-get update -y && apt-get install -y libpq-dev netcat

RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo install cargo-watch
