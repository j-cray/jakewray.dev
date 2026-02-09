#!/usr/bin/env bash
set -e

TARGET=${1:-all}
PROJECT_ID="jakewray-portfolio"
INSTANCE_NAME="jakewray-portfolio"
ZONE="us-west1-a"

echo "Deploying target: $TARGET"



# 0. Clean remote directory (preserving persistent data)
# We NO LONGER wipe the directory to preserve Docker cache and valid files.
# rsync/scp will overwrite changed files.
echo "Preparing remote directory..."
gcloud compute ssh jake-user@$INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --command="
    mkdir -p ~/app && \
    sudo chown -R jake-user:jake-user ~/app
"


# 1. Copy files to VM (Delta sync)
echo "Getting instance IP..."
IP=$(gcloud compute instances describe $INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "Instance IP: $IP"

echo "Ensuring rsync is installed on remote..."
gcloud compute ssh jake-user@$INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --command="
    if ! command -v rsync &> /dev/null; then
        echo 'rsync not found, installing...'
        sudo apt-get update && sudo apt-get install -y rsync
    else
        echo 'rsync is already installed.'
    fi
"

echo "Syncing project files (rsync)..."
# We exclude things that are large, ignored, or platform-specific
rsync -avz --info=progress2 \
    --exclude '.git' \
    --exclude 'target' \
    --exclude 'node_modules' \
    --exclude '.postgres_local' \
    --exclude 'postgres.log' \
    --exclude '.env' \
    --exclude '.DS_Store' \
    -e "ssh -i ~/.ssh/google_compute_engine -o StrictHostKeyChecking=no" \
    ./ \
    jake-user@$IP:~/app/

# 2. SSH and Deploy
echo "Starting remote configuration and build..."
gcloud compute ssh jake-user@$INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --command="
    cd ~/app &&
    chmod +x scripts/*.sh &&
    ./scripts/remote_setup.sh &&
    ./scripts/remote_build.sh $TARGET
"

echo "Deployment of $TARGET complete!"
