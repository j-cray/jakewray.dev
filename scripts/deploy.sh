#!/usr/bin/env bash
set -e

TARGET=${1:-all}
PROJECT_ID="jakewray-portfolio"
INSTANCE_NAME="jakewray-portfolio"
ZONE="us-west1-a"

echo "Deploying target: $TARGET"



# 0. Clean remote directory (preserving persistent data)
echo "Cleaning remote directory and Docker artifacts..."
gcloud compute ssh jake-user@$INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --command="
    sudo docker system prune --force && \
    mkdir -p ~/app && \
    cd ~/app && \
    # Remove everything EXCEPT persistent data (using sudo for root-owned files like target/)
    sudo find . -maxdepth 1 ! -name '.' ! -name 'certbot' ! -name 'media_mount' -exec rm -rf {} + && \
    sudo chown -R jake-user:jake-user ~/app
"


# 1. Copy files to VM
echo "Copying project files..."
gcloud compute scp --recurse \
    ./Dockerfile \
    ./nginx \
    ./docker-compose.prod.yml \
    ./migrations \
    ./Cargo.toml \
    ./backend \
    ./frontend \
    ./shared \
    ./migration \
    ./style \
    ./assets \
    ./scripts \
    ./Cargo.lock \
    jake-user@$INSTANCE_NAME:~/app \
    --project=$PROJECT_ID \
    --zone=$ZONE

# 2. SSH and Deploy
echo "Starting remote configuration and build..."
gcloud compute ssh jake-user@$INSTANCE_NAME --project=$PROJECT_ID --zone=$ZONE --command="
    cd ~/app &&
    chmod +x scripts/*.sh &&
    ./scripts/remote_setup.sh &&
    ./scripts/remote_build.sh $TARGET
"

echo "Deployment of $TARGET complete!"
