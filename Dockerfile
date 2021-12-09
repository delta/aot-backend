FROM rust:1.57.0-slim

WORKDIR /usr/src/aot-backend

COPY . .

RUN apt-get update -y && apt-get install -y libpq-dev netcat

RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo install cargo-watch
