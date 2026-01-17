#!/bin/bash
set -e

TARGET=${1:-all} # Default to 'all' if no argument provided
cd ~/app

echo "Remote Build Target: $TARGET"

# Generate .env file with defaults for production
# Check if .env exists, if so we might want to keep it or just overwrite it to be safe?
# The original script overwrote it every time. Let's stick to that for consistency.
cat <<EOF > .env
POSTGRES_USER=admin
POSTGRES_PASSWORD=password
POSTGRES_DB=portfolio
DOMAIN_NAME=jakewray.dev
LEPTOS_SITE_ADDR=0.0.0.0:3000
RUST_LOG=info
DATABASE_URL=postgres://admin:password@db:5432/portfolio
EOF

if [ "$TARGET" = "all" ] || [ "$TARGET" = "backend" ]; then
    echo "Building dependencies image..."
    sudo docker build --target deps -t portfolio-deps .

    echo "Ensuring DB is up for preparation..."
    sudo docker compose -f docker-compose.prod.yml up -d db
    echo "Waiting for DB..."
    sleep 5 # Reduced from 15, usually 5 is enough if it was already running

    echo "Running sqlx prepare on server..."
    sudo docker compose -f docker-compose.prod.yml run --rm \
        -v $(pwd):/app \
        -w /app \
        -e DATABASE_URL=postgres://admin:password@db:5432/portfolio \
        -e SQLX_OFFLINE=false \
        portfolio-deps \
        cargo sqlx prepare --workspace
fi

if [ "$TARGET" = "all" ]; then
    echo "Building and starting ALL services..."
    sudo docker compose -f docker-compose.prod.yml up -d --build --remove-orphans
elif [ "$TARGET" = "backend" ]; then
    echo "Building and restarting BACKEND (portfolio) service..."
    sudo docker compose -f docker-compose.prod.yml up -d --build --no-deps portfolio
    # We probably want to restart nginx too in case it lost connection?
    # Usually strictly not needed, but good practice if backend container IP changes.
    # But Nginx handles it via dynamic DNS/service names.
elif [ "$TARGET" = "frontend" ]; then
    echo "Frontend is part of the backend binary in this setup (SSR)."
    echo "Please use 'backend' or 'all' target."
    exit 1
else
    echo "Unknown target: $TARGET"
    exit 1
fi
