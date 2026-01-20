-- Create default admin user (should be changed immediately in production)
INSERT INTO users (username, password_hash) 
VALUES ('admin', 'demo-admin-2026!')
ON CONFLICT (username) DO NOTHING;

-- You can also run this from the command line:
-- docker compose -f docker-compose.prod.yml exec -T db psql -U $POSTGRES_USER -d $POSTGRES_DB -c "INSERT INTO users (username, password_hash) VALUES ('admin', 'admin123') ON CONFLICT (username) DO NOTHING;"
