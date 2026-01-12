# jakewray.ca

My personal portfolio website built with Rust, Leptos, and PostgreSQL.

## Live Site

- [jakewray.dev](https://jakewray.dev)

## Deployment

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for complete deployment instructions.

Quick start:
```bash
./scripts/deploy.sh all
```

For first-time SSL setup on the server:
```bash
./scripts/init_ssl.sh
```

## Architecture

- **Backend**: Rust with Leptos (SSR)
- **Database**: PostgreSQL
- **Reverse Proxy**: Nginx with Let's Encrypt SSL
- **Deployment**: Docker Compose

## Development

See the workspace structure:
- `backend/` - Server-side Rust code
- `frontend/` - Client-side Leptos components  
- `shared/` - Shared types and utilities
- `migration/` - Database migration tools

