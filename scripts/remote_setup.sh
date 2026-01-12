#!/bin/bash
set -e

echo "Running Remote Setup..."

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
