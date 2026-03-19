#!/bin/bash
set -e

# This script initializes SSL certificates for the first deployment
# Run this on the server after the first deployment

DOMAIN="jakewray.dev"
EMAIL="admin@jakewray.dev"

echo "Initializing SSL certificates for $DOMAIN..."
cd ~/app

# Ensure services are up (except nginx which needs certs)
echo "Starting backend services..."
sudo docker compose -f compose.prod.yaml up -d db portfolio

# Wait for backend to be ready
echo "Waiting for backend to be ready..."
sleep 10

# Stop nginx if running
sudo docker compose -f compose.prod.yaml stop nginx 2>/dev/null || true

# Get certificates using certbot standalone mode (since nginx isn't running yet)
echo "Requesting Let's Encrypt certificates using standalone mode..."
mkdir -p ~/app/certbot/conf ~/app/certbot/www
sudo docker run --rm -p 80:80 -v ~/app/certbot/conf:/etc/letsencrypt -v ~/app/certbot/www:/var/www/certbot certbot/certbot certonly \
    --standalone \
    --preferred-challenges http \
    --email $EMAIL \
    --agree-tos \
    --no-eff-email \
    -d $DOMAIN \
    -d www.$DOMAIN

# Start nginx with the new certificates
echo "Starting nginx with SSL certificates..."
sudo docker compose -f compose.prod.yaml up -d nginx

# Start certbot renewal service
echo "Starting certbot renewal service..."
sudo docker compose -f compose.prod.yaml up -d certbot

echo "SSL certificates initialized successfully!"
echo "Nginx is now running with HTTPS enabled."
echo "Certificates will auto-renew via certbot service (checks twice daily)."
