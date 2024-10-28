FROM rust:1.80-slim-bookworm as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --manifest-path crates/social-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/social-service .

EXPOSE 3000

CMD ["./social-service"]
