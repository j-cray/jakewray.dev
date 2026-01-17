#!/bin/bash
set -e

# Configuration
DOCKER_CONFIG_DIR="$HOME/.docker/cli-plugins"
PLUGIN_URL="https://github.com/docker/compose/releases/download/v2.24.6/docker-compose-linux-x86_64"
PLUGIN_PATH="$DOCKER_CONFIG_DIR/docker-compose"

echo "Installing Docker Compose Plugin..."

# 1. Create directory
mkdir -p "$DOCKER_CONFIG_DIR"

# 2. Download plugin
echo "Downloading binary from $PLUGIN_URL..."
curl -SL "$PLUGIN_URL" -o "$PLUGIN_PATH"

# 3. Make executable
chmod +x "$PLUGIN_PATH"

echo "Docker Compose Plugin installed successfully at $PLUGIN_PATH"
docker compose version
