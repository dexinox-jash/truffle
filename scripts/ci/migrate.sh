#!/bin/bash
set -e

echo "Running database migrations..."

if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL not set"
    exit 1
fi

cd ruflo-command

cargo run --bin migrate -- "$@"
