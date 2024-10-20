# Use the official Rust image as a parent image
FROM rust:1.80-slim-bookworm as builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

COPY migrations ./migrations

RUN apt-get update && apt-get install -y libssl-dev pkg-config
# Build the application
RUN cargo build --release

# Use a smaller base image for the final image
FROM debian:bookworm-slim

# Install necessary libraries
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/crypto_social_db .

# Expose the port the app runs on
EXPOSE 3000

# Command to run the application
CMD ["./crypto_social_db"]

