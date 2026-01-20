#!/bin/bash
# Local development setup script

set -e

echo "🚀 Setting up local development environment..."

# Check dependencies
command -v cargo &> /dev/null || { echo "❌ cargo not found. Install Rust from https://rustup.rs/"; exit 1; }

# Check for container runtime
CONTAINER_CMD=""
if command -v docker &> /dev/null; then
  # Check if docker daemon is running
  if docker ps &> /dev/null; then
    CONTAINER_CMD="docker"
    echo "✅ Docker found and running"
  else
    echo "⚠️  Docker daemon not running. Trying podman..."
  fi
fi

if [ -z "$CONTAINER_CMD" ] && command -v podman &> /dev/null; then
  CONTAINER_CMD="podman"
  echo "✅ Using podman for containers"
fi

if [ -z "$CONTAINER_CMD" ]; then
  echo "❌ No container runtime found (docker/podman)"
  echo ""
  echo "On macOS, you can use colima (lightweight Docker):"
  echo "  brew install colima"
  echo "  colima start"
  echo ""
  echo "Or use podman:"
  echo "  brew install podman"
  echo "  podman machine init && podman machine start"
  exit 1
fi

echo "✅ All dependencies found"
echo ""

# Start container runtime if using colima
if [ "$CONTAINER_CMD" = "docker" ] && ! docker ps &> /dev/null; then
  echo "🐳 Starting colima (Docker daemon)..."
  if command -v colima &> /dev/null; then
    colima start
  fi
fi

# Start database
echo "📦 Starting PostgreSQL database..."
COMPOSE_CMD="docker-compose"
if [ "$CONTAINER_CMD" = "podman" ]; then
  COMPOSE_CMD="podman-compose"
fi

$COMPOSE_CMD up -d db
sleep 3

echo ""
echo "⏳ Running database migrations..."
cargo sqlx database create || true
cargo sqlx migrate run -D "postgres://admin:password@127.0.0.1:5432/portfolio" || true

echo ""
echo "👤 Creating default admin user..."
PGPASSWORD=password psql -U admin -h 127.0.0.1 -d portfolio -c "INSERT INTO users (username, password_hash) VALUES ('admin', 'demo-admin-2026!') ON CONFLICT (username) DO NOTHING;" || echo "⚠️ Could not create user (may already exist)"

echo ""
echo "✅ Setup complete!"
echo ""
echo "🎯 To run the development server:"
echo ""
echo "   cargo leptos watch"
echo ""
echo "📍 Access at:"
echo "   - Frontend: http://localhost:3000"
echo "   - Admin login: http://localhost:3000/admin/login"
echo ""
echo "🔐 Default credentials:"
echo "   Username: admin"
echo "   Password: demo-admin-2026!"
echo ""
echo "🛑 To stop the database:"
echo "   $COMPOSE_CMD down"
