-- Migration to add status column and indexes for post scheduling and drafts

ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'published';

CREATE INDEX IF NOT EXISTS idx_articles_status_published_at ON articles(status, published_at DESC);
