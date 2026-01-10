#!/usr/bin/env bash
set -e

# Configuration
PROJECT_ID="jakewray-portfolio"
INSTANCE_NAME="jakewray-portfolio"
ZONE="us-central1-a"
MACHINE_TYPE="e2-medium"
IMAGE_FAMILY="debian-12"
IMAGE_PROJECT="debian-cloud"

echo "Deploying to Google Cloud..."

# 1. Create VM if not exists
if ! gcloud compute instances describe $INSTANCE_NAME --zone=$ZONE &>/dev/null; then
    echo "Creating VM instance..."
    gcloud compute instances create $INSTANCE_NAME \
        --project=$PROJECT_ID \
        --zone=$ZONE \
        --machine-type=$MACHINE_TYPE \
        --image-family=$IMAGE_FAMILY \
        --image-project=$IMAGE_PROJECT \
        --tags=http-server,https-server \
        --metadata=startup-script='#! /bin/bash
        # Install Docker
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh

        # Install Docker Compose
        curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
        '

    echo "Waiting for VM to initialize..."
    sleep 30
else
    echo "VM $INSTANCE_NAME already exists."
fi

# 2. Get IP
IP_ADDRESS=$(gcloud compute instances describe $INSTANCE_NAME --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "VM IP Address: $IP_ADDRESS"
echo "IMPORTANT: Update your DNS (A Record) for jakewray.ca to point to $IP_ADDRESS"

# 3. Copy files to VM
echo "Copying project files..."
gcloud compute scp --recurse \
    ./Dockerfile \
    ./docker-compose.prod.yml \
    ./migrations \
    ./Cargo.toml \
    ./backend \
    ./frontend \
    ./shared \
    ./migration \
    ./style \
    ./assets \
    jake-user@$INSTANCE_NAME:~/app \
    --zone=$ZONE

# 4. SSH and Deploy
echo "Starting services on VM..."
gcloud compute ssh jake-user@$INSTANCE_NAME --zone=$ZONE --command="
    # Install gcsfuse if not installed
    if ! command -v gcsfuse &> /dev/null; then
        echo 'Installing gcsfuse...'
        export GCSFUSE_REPO=gcsfuse-\`lsb_release -c -s\`
        echo \"deb https://packages.cloud.google.com/apt \$GCSFUSE_REPO main\" | sudo tee /etc/apt/sources.list.d/gcsfuse.list
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
    sleep 10

    echo "Running sqlx prepare on server..."
    # We mount current dir to /app so sqlx-data.json is written back to host
    sudo docker run --rm \
        --network jake_net \
        -v \$(pwd):/app \
        -w /app \
        -e DATABASE_URL=postgres://admin:password@db:5432/portfolio \
        -e SQLX_OFFLINE=false \
        portfolio-deps \
        cargo sqlx prepare --workspace

    echo "Building and starting application..."
    sudo docker compose -f docker-compose.prod.yml up -d --build --remove-orphans
"

echo "Deployment complete! Visit https://jakewray.ca (after DNS propagation)."
