FROM rust:1.85.1 AS builder
COPY . .
CMD cargo run
