FROM rust:1.80-slim-bookworm as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --manifest-path crates/token_price/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/token_price .

EXPOSE 3000

CMD ["./token_price"]
