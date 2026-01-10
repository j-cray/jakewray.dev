#!/bin/bash
set -e

# Install gcsfuse if not installed
if ! command -v gcsfuse &> /dev/null; then
    echo 'Installing gcsfuse...'
    export GCSFUSE_REPO=gcsfuse-`lsb_release -c -s`
    echo "deb https://packages.cloud.google.com/apt $GCSFUSE_REPO main" | sudo tee /etc/apt/sources.list.d/gcsfuse.list
    curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
    sudo apt-get update
    sudo apt-get install -y gcsfuse
fi

# Create mount point
mkdir -p ~/media_mount

# Mount bucket (if not already mounted)
if ! mount | grep -q 'media_mount'; then
    echo 'Mounting GCS bucket...'
    gcsfuse --implicit-dirs jakewray-portfolio-media ~/media_mount
fi

cd ~/app

# Generate .env file with defaults for production
cat <<EOF > .env
POSTGRES_USER=admin
POSTGRES_PASSWORD=password
POSTGRES_DB=portfolio
DOMAIN_NAME=jakewray.ca
LEPTOS_SITE_ADDR=0.0.0.0:3000
RUST_LOG=info
DATABASE_URL=postgres://admin:password@db:5432/portfolio
EOF

echo "Building dependencies image..."
sudo docker build --target deps -t portfolio-deps .

echo "Starting database for preparation..."
sudo docker compose -f docker-compose.prod.yml up -d db
echo "Waiting for DB..."
sleep 15

echo "Running sqlx prepare on server..."
# We mount current dir to /app so sqlx-data.json is written back to host
sudo docker run --rm \
    --network jake_net \
    -v $(pwd):/app \
    -w /app \
    -e DATABASE_URL=postgres://admin:password@db:5432/portfolio \
    -e SQLX_OFFLINE=false \
    portfolio-deps \
    cargo sqlx prepare --workspace

echo "Building and starting application..."
sudo docker compose -f docker-compose.prod.yml up -d --build --remove-orphans
