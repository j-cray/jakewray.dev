#!/usr/bin/env bash
set -e

# Configuration
PROJECT_ID="jakewray-portfolio"
INSTANCE_NAME="jakewray-portfolio"
ZONE="us-west1-a"
MACHINE_TYPE="c4a-standard-4" # Google Axion (ARM): 4 vCPU, 16GB RAM
IMAGE_FAMILY="debian-12-arm64"
IMAGE_PROJECT="debian-cloud"

echo "Checking Google Cloud Infrastructure..."

# 1. Setup Static IP
ADDRESS_NAME="$INSTANCE_NAME-ip"
if ! gcloud compute addresses describe $ADDRESS_NAME --project=$PROJECT_ID --region=us-west1 &>/dev/null; then
    echo "Creating static IP address..."
    gcloud compute addresses create $ADDRESS_NAME --project=$PROJECT_ID --region=us-west1
fi
STATIC_IP=$(gcloud compute addresses describe $ADDRESS_NAME --project=$PROJECT_ID --region=us-west1 --format='get(address)')
echo "Using Static IP: $STATIC_IP"

# 2. Create VM if not exists
if ! gcloud compute instances describe $INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE &>/dev/null; then
    echo "Creating VM instance..."
    gcloud compute instances create $INSTANCE_NAME \
        --project=$PROJECT_ID \
        --zone=$ZONE \
        --machine-type=$MACHINE_TYPE \
        --image-family=$IMAGE_FAMILY \
        --image-project=$IMAGE_PROJECT \
        --boot-disk-size=64GB \
        --address=$STATIC_IP \
        --tags=http-server,https-server \
        --metadata=startup-script='#! /bin/bash
        # Install Docker
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh

        # Install Docker Compose (standalone)
        COMPOSE_VERSION="v2.24.6"
        OS=$(uname -s | tr '[:upper:]' '[:lower:]')
        ARCH=$(uname -m)
        curl -L "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-${OS}-${ARCH}" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose

        # Install Docker Compose (plugin)
        mkdir -p /usr/local/lib/docker/cli-plugins
        ln -s /usr/local/bin/docker-compose /usr/local/lib/docker/cli-plugins/docker-compose
        '

    echo "Waiting for VM to initialize..."
    sleep 30
else
    echo "VM $INSTANCE_NAME already exists."
fi

# 2. Get IP
IP_ADDRESS=$(gcloud compute instances describe $INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "VM IP Address: $IP_ADDRESS"
echo "Ensure your DNS (A Record) for jakewray.dev points to $IP_ADDRESS"
