FROM rust as builder

RUN rustup component add rustfmt

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY bot/Cargo.toml ./bot/
COPY server/Cargo.toml ./server/

RUN mkdir ./bot/src && echo "fn main(){}" > ./bot/src/main.rs
RUN mkdir ./server/src && echo "fn main(){}" > ./server/src/main.rs

RUN cargo build -p bot --release

COPY . .
RUN cargo build -p bot --release

FROM debian

RUN apt update && apt install -y ca-certificates

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/bot /app/bot