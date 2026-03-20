#!/bin/bash
# Local development setup script

set -e

echo "🚀 Setting up local development environment..."

if [ "$APP_ENV" = "production" ] || [[ "$DATABASE_URL" == *"/app/data"* ]]; then
  echo "❌ Error: Production environment detected. Setup script aborted."
  exit 1
fi

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

# Try to use existing tools if possible, but no background service is needed for sqlite.
echo ""
echo "⏳ Running database migrations..."
# create an empty sqlite database file if it doesn't exist
touch sqlite.db
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="sqlite://sqlite.db"
fi

cargo sqlx migrate run -D "$DATABASE_URL" || true

echo ""
echo "👤 Creating default admin user..."
# Fallback to python UUID or kernel uuid if uuidgen missing
ADMIN_UUID=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || python3 -c 'import uuid; print(uuid.uuid4())' 2>/dev/null || { echo "❌ Could not generate a UUID. Please install uuidgen."; exit 1; })
SAFE_UUID=$(printf '%q' "$ADMIN_UUID" | tr -cd 'a-fA-F0-9-')

if ! [[ "$SAFE_UUID" =~ ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$ ]]; then
  echo "❌ Invalid Admin UUID format generated: $SAFE_UUID"
  exit 1
fi

sqlite3 sqlite.db <<EOF || echo "⚠️ Could not create user (may already exist)"
INSERT INTO users (id, username, password_hash) VALUES ('${SAFE_UUID}', 'admin', '\$argon2id\$v=19\$m=19456,t=2,p=1\$eZjB8IC9MeFUwBfPULedVA\$INlXdgAcRKilnu3//TUQ3ds00iBb5rMTw39vBfOwK30') ON CONFLICT (username) DO NOTHING;
EOF

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
echo "🛑 Setup complete."
