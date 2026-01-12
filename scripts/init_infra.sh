#!/usr/bin/env bash
set -e

# Configuration
PROJECT_ID="jakewray-portfolio"
INSTANCE_NAME="jakewray-portfolio"
ZONE="us-central1-a"
MACHINE_TYPE="e2-medium"
IMAGE_FAMILY="debian-12"
IMAGE_PROJECT="debian-cloud"

echo "Checking Google Cloud Infrastructure..."

# 1. Create VM if not exists
if ! gcloud compute instances describe $INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE &>/dev/null; then
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
IP_ADDRESS=$(gcloud compute instances describe $INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "VM IP Address: $IP_ADDRESS"
echo "Ensure your DNS (A Record) for jakewray.dev points to $IP_ADDRESS"
