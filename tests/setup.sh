#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Starting test Postgres container..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d

echo "Waiting for Postgres to be ready..."
for i in $(seq 1 30); do
    if pg_isready -h localhost -p 5433 -U test -d crabase_test > /dev/null 2>&1; then
        echo "Postgres is ready."
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "ERROR: Postgres did not become ready in time."
        exit 1
    fi
    sleep 1
done

echo "Running seed SQL..."
PGPASSWORD=test psql -h localhost -p 5433 -U test -d crabase_test -f "$SCRIPT_DIR/seed.sql"

echo "Test database is ready."
