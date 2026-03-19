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

### Known Limitations
- **Database Concurrency**: The application uses embedded SQLite in WAL mode with a small connection pool (`max_connections(5)`). SQLite only allows one concurrent writer. Concurrent write bursts will queue (up to a 5s busy timeout) and could fail under heavy write load. This is acceptable for a personal blog/portfolio, but must be accounted for if write traffic scales.

## Development

### Quick Start with Nix (Recommended)
```bash
direnv allow          # Load development environment  
./scripts/setup-dev.sh # Setup database
cargo leptos watch    # Start dev server
```

See [docs/LOCAL_DEV.md](docs/LOCAL_DEV.md) for detailed setup.

### Without Nix
```bash
cargo install cargo-leptos
./scripts/setup-dev.sh
cargo leptos watch
```

## Project Structure
- `backend/` - Server-side Rust code
- `frontend/` - Client-side Leptos components  
- `shared/` - Shared types and utilities
- `flake.nix` - Nix development environment
- `.envrc` - direnv configuration


## Development Roadmap
- [x] **HTTPS/SSL** - Let's Encrypt certificates
- [x] **Authentication** - Password-protected admin panel
- [x] **Theme** - Modern indigo design
- [ ] **Admin features** - Post creation, sync manager
- [ ] **Content sync** - Import from terracestandard.com
- [ ] **Media library** - Photo/video management
- [x] **Password hashing** - Argon2 implementation
- [ ] **Password reset** - Email-based recovery
