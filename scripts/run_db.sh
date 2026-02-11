#!/bin/bash
# Run the database container for jakewray.dev
# Uses bitnami/postgresql and keep-id for rootless Podman compatibility.

# Ensure the old container is gone
podman rm -f db 2>/dev/null || true

# Run the container
podman run -d --name db \
  --restart always \
  --userns=keep-id \
  -e POSTGRESQL_USERNAME=admin \
  -e POSTGRESQL_PASSWORD=password \
  -e POSTGRESQL_DATABASE=portfolio \
  -p 5432:5432 \
  -v db_data_v2:/bitnami/postgresql \
  docker.io/bitnami/postgresql:latest

echo "Database container 'db' started on port 5432."
echo "Use 'podman logs -f db' to view logs."
