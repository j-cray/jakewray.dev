# Fixing jakewray.dev Timeout Issue - Summary

## Problem
The website was timing out when accessing `jakewray.dev`. The root cause was that nginx-proxy-manager required manual UI configuration (via web interface on port 81) to create proxy host entries, which was not automated or version-controlled.

## Solution
Replaced nginx-proxy-manager with Nginx + Certbot for a fully automated, configuration-as-code solution.

## Changes Made

### 1. New Files Created
- **`nginx.conf`** - Complete Nginx configuration with:
  - Reverse proxy to portfolio backend (port 3000)
  - HTTP → HTTPS redirect
  - SSL/TLS settings (TLS 1.2/1.3)
  - Security headers
  - Rate limiting (10 req/s, burst 20)
  - Gzip compression
  - www → non-www redirect
  
- **`scripts/init_ssl.sh`** - First-time SSL setup script:
  - Uses certbot standalone mode to get initial certificates
  - Starts nginx after certificates are obtained
  - Enables automatic renewal service

- **`docs/DEPLOYMENT.md`** - Comprehensive deployment documentation:
  - Architecture overview
  - First-time deployment steps
  - Troubleshooting guide
  - Migration instructions

### 2. Modified Files
- **`docker-compose.prod.yml`**:
  - Removed `nginx-proxy-manager` service
  - Added `nginx:alpine` as proxy service
  - Added `certbot` service for certificate renewal (runs every 6h)
  - Updated volumes (certbot_conf, certbot_www)

- **`scripts/deploy.sh`**:
  - Updated to copy `nginx.conf` instead of Caddyfile

- **`.gitignore`**:
  - Added npm_data/, npm_letsencrypt/, media_mount/

### 3. Deleted Files
- **`Caddyfile`** - Replaced with nginx.conf per user requirement

## Key Benefits

1. **No Manual Configuration**: Everything is defined in code
2. **Automatic HTTPS**: Certbot handles certificate provisioning and renewal
3. **Version Controlled**: All configuration is in git
4. **Production Ready**: Industry-standard Nginx setup
5. **Troubleshooting**: Clear documentation and logging

## How It Works

### Normal Operation
```
Internet → Port 80/443 → Nginx → portfolio:3000 → Leptos App
```

### Certificate Renewal
- Certbot service checks for renewal every 6 hours
- Automatically renews certificates when needed (30 days before expiry)
- Nginx reloads to use new certificates

### First-Time Setup Flow
1. Deploy code with `./scripts/deploy.sh all`
2. SSH to server and run `./scripts/init_ssl.sh`
3. Script obtains certificates using standalone mode
4. Nginx starts with HTTPS enabled
5. Certbot renewal service begins monitoring

## Testing Recommendations

After deployment:

1. **Check HTTP redirect**:
   ```bash
   curl -I http://jakewray.dev
   # Should return 301 redirect to https://
   ```

2. **Check HTTPS**:
   ```bash
   curl -I https://jakewray.dev
   # Should return 200 OK
   ```

3. **Verify backend health**:
   ```bash
   curl https://jakewray.dev/health
   # Should return "OK"
   ```

4. **Check certificate**:
   ```bash
   echo | openssl s_client -connect jakewray.dev:443 -servername jakewray.dev 2>/dev/null | openssl x509 -noout -dates
   ```

## Rollback Plan

If issues occur:

1. Services can be restarted individually:
   ```bash
   docker compose -f docker-compose.prod.yml restart proxy
   docker compose -f docker-compose.prod.yml restart portfolio
   ```

2. To revert to nginx-proxy-manager:
   ```bash
   git revert <this-commit>
   # Then manually configure NPM via UI
   ```

## Security Considerations

✅ TLS 1.2 and 1.3 only
✅ Strong cipher suites
✅ Security headers enabled
✅ Rate limiting configured
✅ Automatic certificate renewal
✅ No sensitive data in configuration
