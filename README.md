# jakewray.ca

My personal portfolio website built with Rust, Leptos, and SQLite.

## Live Site

- [jakewray.dev](https://jakewray.dev)

## Architecture

- **Backend**: Rust with Leptos (SSR)
- **Database**: SQLite

### Known Limitations

- **Database Concurrency**: The application uses embedded SQLite in WAL mode with a small connection pool (`max_connections(5)`). SQLite only allows one concurrent writer. Concurrent write bursts will queue (up to a 5s busy timeout) and could fail under heavy write load. This is acceptable for a personal blog/portfolio, but must be accounted for if write traffic scales.
- **Reverse Proxy Setup**: When deploying behind a reverse proxy (such as Nginx), you **MUST** configure the `TRUSTED_PROXY_IPS` environment variable with the proxy's IP address. If left unset, all client requests will appear to come from the proxy's IP, effectively disabling per-client rate limiting and causing all users to share the same rate limit bucket.

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
- [ ] **Admin features** - Post creation
- [ ] **Media library** - Photo/video management
- [x] **Password hashing** - Argon2 implementation
- [ ] **Password reset** - Email-based recovery
