# Local Development Guide

## Prerequisites

- **Nix with Flakes** (recommended): [Install Nix](https://nixos.org/download)
- **direnv** (optional but recommended): `brew install direnv`
- OR **Rust**: [Install from rustup.rs](https://rustup.rs/)
- **Docker & Docker Compose**: [Install Docker Desktop](https://www.docker.com/products/docker-desktop)

## Setup with Nix + direnv (Recommended)

### 1. Install direnv
```bash
brew install direnv
```

### 2. Hook direnv into your shell
Add to your `~/.zshrc` or `~/.bashrc`:
```bash
eval "$(direnv hook zsh)"  # or bash
```

### 3. Set up container runtime (choose one)

**Option A: colima (Lightweight Docker - Recommended)**
```bash
brew install colima
colima start
```

**Option B: podman (No Docker needed)**
```bash
brew install podman
podman machine init
podman machine start
```

**Option C: Docker Desktop**
Just install and run Docker Desktop normally.

### 4. Allow direnv in project
```bash
cd /path/to/jakewray.dev
direnv allow
```

This will:
- ✅ Load Nix flake automatically
- ✅ Set environment variables from `.envrc`
- ✅ Add dev tools to PATH (including docker/podman)
- ✅ Set database connection string

### 5. Verify environment loaded
```bash
cargo --version
rustc --version
docker --version  # or podman
psql --version
```

All should show versions without errors.

## Setup with Nix only (no direnv)

If you don't want direnv, manually enter the dev environment:
```bash
nix flake update
nix develop
```

Then you can run commands as normal.

## Quick Start (Any Setup Method)

### 1. Run setup script
```bash
./scripts/setup-dev.sh
```

This will:
- ✅ Start PostgreSQL database in Docker
- ✅ Run database migrations
- ✅ Create default admin user (`admin` / `admin123`)

### 2. Start the development server
```bash
cargo leptos watch
```

The server will:
- Compile Rust backend code
- Build WASM frontend components
- Watch for file changes and rebuild automatically
- Serve on `http://localhost:3000`

## Testing the Login Flow

### 1. Access the site
Open `http://localhost:3000` in your browser

### 2. Try accessing the admin panel
Navigate to `http://localhost:3000/admin/login`

### 3. Login with default credentials
- **Username**: `admin`
- **Password**: `admin123`

### 4. Test the features
- ✅ Verify login redirects to dashboard
- ✅ Check that theme is applied (indigo colors, modern styling)
- ✅ Click logout and verify redirect to login
- ✅ Try accessing dashboard without login - should redirect to login

### 5. Check browser console
- Open DevTools (F12)
- Check Network tab to see API requests to `/api/admin/login`
- Check Console for any errors
- Check Application tab to see localStorage with `admin_token`

## Database Access

### View database directly
```bash
PGPASSWORD=password psql -U admin -h 127.0.0.1 -d portfolio
```

### Query users table
```sql
SELECT id, username FROM users;
```

## Common Development Tasks

### Change admin password
```bash
PGPASSWORD=password psql -U admin -h 127.0.0.1 -d portfolio -c "UPDATE users SET password_hash = 'newpassword' WHERE username = 'admin';"
```

### Reset database
```bash
docker-compose down -v
docker-compose up -d db
# Re-run migrations
./scripts/setup-dev.sh
```

### Stop development server
Press `Ctrl+C` in the terminal

### Stop database
```bash
docker-compose down
```

### View database logs
```bash
docker-compose logs db
```

### View application logs
They appear in the terminal where `cargo leptos watch` is running

## Debugging

### Enable more verbose logging
```bash
RUST_LOG=debug cargo leptos watch
```

### Check which port the app is running on
The output of `cargo leptos watch` will show:
```
Listening on http://0.0.0.0:3000
```

### If port 3000 is already in use
Edit `Cargo.toml` under `[workspace.metadata.leptos]`:
```toml
site-addr = "0.0.0.0:3001"  # Change port here
```

### Database connection issues
Check if postgres is running:
```bash
docker ps | grep postgres
```

If not running:
```bash
docker-compose up -d db
```

## Project Structure

```
.
├── backend/           # Rust backend (Axum server)
├── frontend/          # Leptos frontend (React-like components)
├── shared/           # Shared types between frontend/backend
├── migration/        # Database migration tools
├── style/            # SCSS stylesheet
├── docker-compose.yml # Local dev database
└── Cargo.toml        # Workspace configuration
```

## File Changes During Development

When you modify:
- **Rust files** (`src/*.rs`) - Backend/frontend recompiles automatically
- **Style files** (`style/main.scss`) - CSS recompiles automatically
- **HTML** - Page refreshes automatically

The watch command handles hot-reloading!

## Making Changes to Authentication

### Backend changes
Edit files in `backend/src/api/admin.rs`
- Login logic updates automatically
- Restart not needed (watch mode recompiles)

### Frontend changes
Edit files in `frontend/src/pages/admin/login.rs`
- UI updates automatically
- Refresh browser to see changes

### Adding new admin routes
1. Add route to `backend/src/api/admin.rs`
2. Add frontend page in `frontend/src/pages/admin/`
3. The watch system handles recompilation

## Testing Tips

1. **Test with empty localStorage**: Open DevTools → Application → Clear site data
2. **Test network conditions**: DevTools → Network tab → set throttling
3. **Test mobile**: DevTools → Toggle device toolbar (Ctrl+Shift+M)
4. **Check form validation**: Try submitting without credentials
5. **Test token expiration**: Manually edit localStorage token to test 

## Production Deployment

When ready to deploy:
```bash
cargo leptos build --release
./scripts/deploy.sh all
```

See [docs/DEPLOYMENT.md](../DEPLOYMENT.md) for full deployment guide.

## Troubleshooting

### "Cannot find leptos_config in workspace"
Run: `cargo leptos build` once first

### "Port 5432 already in use"
Stop other postgres instances: `docker ps` and `docker kill <container>`

### "Node.js not found" warning
This is optional - only needed if using JS-based asset tools

### "Module not found" errors
Run: `cargo clean && cargo build`

### Browser shows blank page
- Check browser console (F12) for errors
- Check terminal output for compilation errors
- Make sure `cargo leptos watch` is running

## Getting Help

1. Check `cargo leptos --help` for command options
2. Review [Leptos documentation](https://leptos.dev)
3. Check [Axum documentation](https://docs.rs/axum)
4. Review project issue comments in source files
