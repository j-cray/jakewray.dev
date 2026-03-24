-- Normalize datetime precision to match the default SQLite %f format (milliseconds)
-- This ensures consistent precision for cursor-based pagination.

UPDATE articles
SET published_at = strftime('%Y-%m-%dT%H:%M:%fZ', published_at)
WHERE published_at IS NOT NULL;

UPDATE blog_posts
SET published_at = strftime('%Y-%m-%dT%H:%M:%fZ', published_at)
WHERE published_at IS NOT NULL;
