#!/bin/bash
# Database migration script for Ruflo platform

set -e

# Configuration
DB_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/ruflo}"
MIGRATIONS_DIR="${MIGRATIONS_DIR:-./migrations}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create migrations table if not exists
create_migrations_table() {
    psql "$DB_URL" -c "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT NOW()
        );
    " 2>/dev/null || true
}

# Get applied migrations
get_applied_migrations() {
    psql "$DB_URL" -t -c "SELECT version FROM schema_migrations ORDER BY version;" 2>/dev/null || echo ""
}

# Apply migration
apply_migration() {
    local version=$1
    local file=$2
    
    log_info "Applying migration $version..."
    
    if psql "$DB_URL" -f "$file"; then
        psql "$DB_URL" -c "INSERT INTO schema_migrations (version) VALUES ('$version');"
        log_info "Migration $version applied successfully"
    else
        log_error "Failed to apply migration $version"
        exit 1
    fi
}

# Main migration function
migrate() {
    log_info "Starting database migration..."
    log_info "Database: $DB_URL"
    
    # Create migrations table
    create_migrations_table
    
    # Get applied migrations
    applied=$(get_applied_migrations)
    
    # Find and apply pending migrations
    pending=0
    for file in "$MIGRATIONS_DIR"/*.up.sql; do
        if [ -f "$file" ]; then
            version=$(basename "$file" | cut -d'_' -f1)
            
            if ! echo "$applied" | grep -q "$version"; then
                apply_migration "$version" "$file"
                pending=$((pending + 1))
            fi
        fi
    done
    
    if [ $pending -eq 0 ]; then
        log_info "No pending migrations"
    else
        log_info "Applied $pending migration(s)"
    fi
}

# Rollback function
rollback() {
    local steps=${1:-1}
    
    log_info "Rolling back $steps migration(s)..."
    
    applied=$(get_applied_migrations | tail -n "$steps")
    
    for version in $applied; do
        down_file="$MIGRATIONS_DIR/${version}_*.down.sql"
        
        if [ -f $down_file ]; then
            log_info "Rolling back migration $version..."
            psql "$DB_URL" -f $down_file
            psql "$DB_URL" -c "DELETE FROM schema_migrations WHERE version = '$version';"
        else
            log_warn "No down migration found for $version"
        fi
    done
}

# Status function
status() {
    log_info "Migration status:"
    
    applied=$(get_applied_migrations)
    
    echo "Applied migrations:"
    echo "$applied" | while read -r version; do
        if [ -n "$version" ]; then
            echo "  ✓ $version"
        fi
    done
    
    echo ""
    echo "Pending migrations:"
    for file in "$MIGRATIONS_DIR"/*.up.sql; do
        if [ -f "$file" ]; then
            version=$(basename "$file" | cut -d'_' -f1)
            if ! echo "$applied" | grep -q "$version"; then
                echo "  ○ $version"
            fi
        fi
    done
}

# Create new migration
create() {
    local name=$1
    
    if [ -z "$name" ]; then
        log_error "Migration name required"
        exit 1
    fi
    
    timestamp=$(date +%Y%m%d%H%M%S)
    filename="${timestamp}_${name}"
    
    mkdir -p "$MIGRATIONS_DIR"
    
    cat > "$MIGRATIONS_DIR/${filename}.up.sql" << EOF
-- Migration: $name
-- Created at: $(date)

BEGIN;

-- Add your migration here

COMMIT;
EOF
    
    cat > "$MIGRATIONS_DIR/${filename}.down.sql" << EOF
-- Rollback: $name
-- Created at: $(date)

BEGIN;

-- Add your rollback here

COMMIT;
EOF
    
    log_info "Created migration: $filename"
}

# Show usage
usage() {
    echo "Usage: $0 {migrate|rollback|status|create <name>}"
    echo ""
    echo "Commands:"
    echo "  migrate           Apply all pending migrations"
    echo "  rollback [n]      Rollback n migrations (default: 1)"
    echo "  status            Show migration status"
    echo "  create <name>     Create a new migration"
    echo ""
    echo "Environment variables:"
    echo "  DATABASE_URL      PostgreSQL connection string"
    echo "  MIGRATIONS_DIR    Directory containing migrations (default: ./migrations)"
}

# Main
case "${1:-}" in
    migrate)
        migrate
        ;;
    rollback)
        rollback "${2:-1}"
        ;;
    status)
        status
        ;;
    create)
        create "$2"
        ;;
    *)
        usage
        exit 1
        ;;
esac
