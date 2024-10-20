#!/bin/bash

# check if cargo is installed
if ! command -v cargo &>/dev/null; then
    echo "Cargo is not installed. Please install Cargo first."
    exit 1
fi

# Intall Just if it's not installed
if ! command -v just &>/dev/null; then
    cargo install just
fi

# Install sqlx-cli if it's not installed
if ! command -v sqlx &>/dev/null; then
    cargo install sqlx-cli
fi

# Set up the environment
just setup-dev

# Run migrations
just run-migrations

# Run the application
just run
