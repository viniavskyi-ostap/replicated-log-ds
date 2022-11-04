FROM rust:1.64

RUN USER=root cargo new --bin secondary
WORKDIR /secondary

RUN cargo generate-lockfile
# Copy Rust manifest
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --bin secondary