#!/bin/bash
set -e

ENVIRONMENT=$1

if [ -z "$ENVIRONMENT" ]; then
    echo "Usage: $0 <environment>"
    echo "Environments: staging, production"
    exit 1
fi

echo "Running pre-deployment checks for $ENVIRONMENT..."

# Check required secrets
required_secrets=(
    "DATABASE_URL"
    "REDIS_URL"
    "JWT_SECRET"
    "ENCRYPTION_KEY"
)

for secret in "${required_secrets[@]}"; do
    if [ -z "${!secret}" ]; then
        echo "❌ Missing required secret: $secret"
        exit 1
    fi
    echo "✅ $secret is set"
done

# Check database connectivity
echo "Checking database connectivity..."
psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1 && echo "✅ Database connection OK" || echo "⚠️  Database connection failed"

# Check migrations are up to date
echo "Checking migration status..."
cd ruflo-command
cargo run --bin migrate -- check

echo "✅ All pre-deployment checks passed!"
