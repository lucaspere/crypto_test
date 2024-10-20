# List available commands
default:
    @just --list

# Set up the development environment
setup-dev:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f .env ]; then
        cp .env.example .env
        echo ".env file created from .env.example"
    fi
    cargo install sqlx-cli
    just run-migrations
    just run-seed

# Run database migrations
run-migrations:
    sqlx migrate run

# Run database seed
run-seed:
    cargo run --bin seed

# Start the application in development mode
dev:
    cargo run --bin crypto_social_db

# Build the application for production
build:
    cargo build --release

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Format code
format:
    cargo fmt

# Check code formatting
check-format:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run the application in production mode
prod:
    ENVIRONMENT=PROD cargo run --release

# Run the application in docker
docker-run:
    docker compose up --build -d