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
DOMAIN_NAME=jakewray.dev
LEPTOS_SITE_ADDR=0.0.0.0:3000
RUST_LOG=info
DATABASE_URL=sqlite:////app/data/sqlite.db
ENVIRONMENT=production
JWT_SECRET=$(openssl rand -base64 48 | tr -d '\n')
TRUSTED_PROXY_IPS=172.18.0.2,172.18.0.3
EOF

if [ "$TARGET" = "all" ] || [ "$TARGET" = "backend" ]; then
    echo "Building chef base image (with cache)..."
    sudo DOCKER_BUILDKIT=1 docker build \
        --target chef \
        --cache-from portfolio-chef:latest \
        -t portfolio-chef .

    echo "Ensuring data directory exists..."
    mkdir -p data && chmod 700 data && sudo chown 1000:1000 data
fi

if [ "$TARGET" = "all" ]; then
    echo "Building and starting ALL services with BuildKit caching..."
    sudo DOCKER_BUILDKIT=1 docker compose -f compose.prod.yaml build \
        --build-arg BUILDKIT_INLINE_CACHE=1
    mkdir -p data && chmod 700 data && sudo chown 1000:1000 data
    sudo docker compose -f compose.prod.yaml up -d --remove-orphans
elif [ "$TARGET" = "backend" ]; then
    echo "Building and restarting BACKEND (portfolio) service with caching..."
    sudo DOCKER_BUILDKIT=1 docker compose -f compose.prod.yaml build \
        --build-arg BUILDKIT_INLINE_CACHE=1 portfolio
    mkdir -p data && chmod 700 data && sudo chown 1000:1000 data
    sudo docker compose -f compose.prod.yaml up -d --no-deps portfolio
elif [ "$TARGET" = "frontend" ]; then
    echo "Frontend is part of the backend binary in this setup (SSR)."
    echo "Please use 'backend' or 'all' target."
    exit 1
else
    echo "Unknown target: $TARGET"
    exit 1
fi
