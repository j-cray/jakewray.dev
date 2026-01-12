#!/bin/bash
set -e

# This script initializes SSL certificates for the first deployment
# Run this on the server after the first deployment

DOMAIN="jakewray.dev"
EMAIL="admin@jakewray.dev"

echo "Initializing SSL certificates for $DOMAIN..."
cd ~/app

# Ensure services are up (except proxy which needs certs)
echo "Starting backend services..."
sudo docker compose -f docker-compose.prod.yml up -d db portfolio

# Wait for backend to be ready
echo "Waiting for backend to be ready..."
sleep 10

# Stop proxy if running
sudo docker compose -f docker-compose.prod.yml stop proxy 2>/dev/null || true

# Get certificates using certbot standalone mode (since nginx isn't running yet)
echo "Requesting Let's Encrypt certificates using standalone mode..."
sudo docker compose -f docker-compose.prod.yml run --rm -p 80:80 certbot certonly \
    --standalone \
    --preferred-challenges http \
    --email $EMAIL \
    --agree-tos \
    --no-eff-email \
    -d $DOMAIN \
    -d www.$DOMAIN

# Start proxy with the new certificates
echo "Starting nginx with SSL certificates..."
sudo docker compose -f docker-compose.prod.yml up -d proxy

# Start certbot renewal service
echo "Starting certbot renewal service..."
sudo docker compose -f docker-compose.prod.yml up -d certbot

echo "SSL certificates initialized successfully!"
echo "Nginx is now running with HTTPS enabled."
echo "Certificates will auto-renew via certbot service (checks twice daily)."
