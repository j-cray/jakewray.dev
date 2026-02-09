#!/bin/bash
set -e

TARGET=${1:-all} # Default to 'all' if no argument provided
cd ~/app

echo "Remote Build Target: $TARGET"

# Enable Docker BuildKit for better caching
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Generate .env file with defaults for production
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
    echo "Building chef base image (with cache)..."
    sudo DOCKER_BUILDKIT=1 docker build \
        --target chef \
        --cache-from portfolio-chef:latest \
        -t portfolio-chef .

    echo "Ensuring DB is up for preparation..."
    sudo docker compose -f docker-compose.prod.yml up -d db
    echo "Waiting for DB..."
    sleep 5

    echo "Running sqlx prepare on server..."
    DB_CONTAINER=$(sudo docker compose -f docker-compose.prod.yml ps -q db | head -n1)

    # We use the chef image which has sqlx-cli installed, and mount source code
    sudo docker run --rm \
        --network container:$DB_CONTAINER \
        -v "$(pwd)":/app \
        -w /app \
        -u root \
        -e DATABASE_URL=postgres://admin:password@localhost:5432/portfolio \
        -e SQLX_OFFLINE=false \
        portfolio-chef \
        cargo sqlx prepare --workspace
    sudo chown -R jake-user:jake-user .
fi

if [ "$TARGET" = "all" ]; then
    echo "Building and starting ALL services with BuildKit caching..."
    sudo DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build \
        --build-arg BUILDKIT_INLINE_CACHE=1
    sudo docker compose -f docker-compose.prod.yml up -d --remove-orphans
elif [ "$TARGET" = "backend" ]; then
    echo "Building and restarting BACKEND (portfolio) service with caching..."
    sudo DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build \
        --build-arg BUILDKIT_INLINE_CACHE=1 portfolio
    sudo docker compose -f docker-compose.prod.yml up -d --no-deps portfolio
elif [ "$TARGET" = "frontend" ]; then
    echo "Frontend is part of the backend binary in this setup (SSR)."
    echo "Please use 'backend' or 'all' target."
    exit 1
else
    echo "Unknown target: $TARGET"
    exit 1
fi
