# Deployment Guide

## Overview

This application uses Docker Compose for production deployment with Nginx as a reverse proxy and Let's Encrypt for SSL certificates.

## Architecture

- **Backend**: Rust/Leptos application (port 3000)
- **Database**: PostgreSQL 15
- **Reverse Proxy**: Nginx (with automatic HTTPS via Let's Encrypt)
- **Certificate Management**: Certbot (automatic renewal)

## Domain Configuration

The application is configured to serve traffic for `jakewray.dev` through Nginx, which handles:
- HTTPS certificate provisioning via Let's Encrypt/Certbot
- HTTP to HTTPS redirection
- Reverse proxy to the backend service
- Security headers and rate limiting

### Nginx Configuration

The `nginx.conf` configures:
- `jakewray.dev` - Main domain, proxies to the backend on port 3000
- `www.jakewray.dev` - Redirects to the main domain (non-www)
- SSL/TLS with modern security settings
- Gzip compression for performance
- Rate limiting (10 requests/second with burst of 20)

## First-Time Deployment

1. Ensure DNS A records point to your server IP:
   - `jakewray.dev` → Server IP
   - `www.jakewray.dev` → Server IP

2. Run the deployment script:
   ```bash
   ./scripts/deploy.sh all
   ```

3. Initialize SSL certificates (first time only):
   ```bash
   # SSH into the server, then run:
   cd ~/app
   ./scripts/init_ssl.sh
   ```

   This script will:
   - Create temporary self-signed certificates
   - Start Nginx
   - Request real certificates from Let's Encrypt
   - Reload Nginx with the new certificates

## Subsequent Deployments

For code updates, simply run:
```bash
./scripts/deploy.sh all
```

Certificates will be automatically renewed by the certbot service (checks twice daily).

## Services

- `portfolio` - Main application (internal port 3000)
- `db` - PostgreSQL database
- `proxy` - Nginx reverse proxy (ports 80/443)
- `certbot` - Certificate renewal service
- `migration` - Database migration service (runs once)

## Troubleshooting

### Site Not Loading

1. Check if all services are running:
   ```bash
   docker compose -f docker-compose.prod.yml ps
   ```

2. View Nginx logs:
   ```bash
   docker compose -f docker-compose.prod.yml logs proxy
   ```

3. View backend logs:
   ```bash
   docker compose -f docker-compose.prod.yml logs portfolio
   ```

4. Verify DNS records:
   ```bash
   dig jakewray.dev
   ```

### Certificate Issues

1. Check certbot logs:
   ```bash
   docker compose -f docker-compose.prod.yml logs certbot
   ```

2. Manually renew certificates:
   ```bash
   docker compose -f docker-compose.prod.yml run --rm certbot renew
   docker compose -f docker-compose.prod.yml exec proxy nginx -s reload
   ```

3. Ensure ports 80 and 443 are accessible from the internet

### Connection Timeout

If you're experiencing timeouts:

1. Check if the portfolio service is running and healthy:
   ```bash
   docker compose -f docker-compose.prod.yml exec portfolio curl http://localhost:3000/health
   ```

2. Check Nginx can reach the backend:
   ```bash
   docker compose -f docker-compose.prod.yml exec proxy wget -O- http://portfolio:3000/health
   ```

3. Verify firewall rules allow traffic on ports 80 and 443

## Migration from nginx-proxy-manager

This deployment previously used nginx-proxy-manager. The migration to Nginx + Certbot provides:
- Configuration as code (no manual UI setup required)
- Automatic certificate renewal
- Better performance and control
- Industry-standard setup

If upgrading from the old setup:
1. The old volumes (`npm_data`, `npm_letsencrypt`) can be safely removed after verifying the new setup works
2. Run `init_ssl.sh` to set up new certificates
3. The migration is seamless - no data loss

## Security Features

The Nginx configuration includes:
- TLS 1.2 and 1.3 only
- Strong cipher suites
- Security headers (X-Frame-Options, X-Content-Type-Options, etc.)
- Rate limiting to prevent abuse
- HTTP to HTTPS redirection
