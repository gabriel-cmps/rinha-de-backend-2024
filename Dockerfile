FROM rust:1-slim-buster AS build

RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev

RUN cargo new --bin app
WORKDIR /app

COPY Cargo.toml /app/
COPY Cargo.lock /app/
RUN cargo build --release 

COPY src /app/src
RUN touch /app/src/main.rs
RUN cargo build --release 

FROM debian:buster-slim

COPY --from=build /app/target/release/rinha-de-backend-2024 /app/rinha

COPY db /app/db

RUN apt-get update && apt-get install -y sqlite3

RUN sqlite3 /app/db/db.sqlite < /app/db/init.sql

CMD "/app/rinha"