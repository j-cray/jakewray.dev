#!/bin/bash
set -e

echo "Running Remote Setup..."
# Ensure Docker Compose is installed correctly
./scripts/install_compose_plugin.sh

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

# Configure Swap (4GB)
if [ ! -f /swapfile ]; then
    echo "Creating 4GB swap file..."
    sudo fallocate -l 4G /swapfile
    sudo chmod 600 /swapfile
    sudo mkswap /swapfile
    sudo swapon /swapfile
    echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
    echo "Swap created."
else
    echo "Swap file already exists."
fi
