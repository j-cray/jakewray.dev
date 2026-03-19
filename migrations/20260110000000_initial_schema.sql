-- Users (Admin)
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Articles (Journalism - Imported/Synced)
CREATE TABLE articles (
    id UUID PRIMARY KEY,
    wp_id BIGINT UNIQUE, -- External ID from WordPress
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    subtitle TEXT,
    excerpt TEXT,
    content TEXT NOT NULL, -- HTML content
    cover_image_url TEXT,
    author TEXT NOT NULL,
    published_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    origin TEXT NOT NULL DEFAULT 'local', -- 'imported', 'synced', 'local'
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Personal Blog Posts
CREATE TABLE blog_posts (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content TEXT NOT NULL, -- Markdown/Rich Text
    published_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    tags TEXT, -- JSON Array
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Creative Writing (Stories, Novels, Poetry)
CREATE TABLE creative_works (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    work_type TEXT NOT NULL, -- 'story', 'novel', 'poetry'
    synopsis TEXT,
    content TEXT, -- Full text or chapters (can be JSON if complex)
    status TEXT NOT NULL DEFAULT 'published', -- 'draft', 'published'
    published_at DATETIME DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Media Items (Photography, Visual Art, J-School Video, Videography)


CREATE TABLE media_items (
    id UUID PRIMARY KEY,
    title TEXT,
    description TEXT,
    url TEXT NOT NULL, -- S3 URL or local path
    thumbnail_url TEXT,
    category TEXT NOT NULL, -- 'photography', 'visual_art', 'video', 'j_school'
    context TEXT NOT NULL DEFAULT 'personal', -- To distinguish Photojournalism (prof) vs Personal
    taken_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Music
CREATE TABLE music_tracks (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    audio_url TEXT,
    embed_code TEXT, -- For Soundcloud/Spotify iframe
    published_at DATETIME DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Programming Projects
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    github_url TEXT,
    demo_url TEXT,
    technologies TEXT, -- JSON Array
    stars INT DEFAULT 0,
    is_featured BOOLEAN DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Triggers for UUID generation on primary keys
CREATE TRIGGER set_users_id AFTER INSERT ON users
WHEN NEW.id IS NULL
BEGIN UPDATE users SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_articles_id AFTER INSERT ON articles
WHEN NEW.id IS NULL
BEGIN UPDATE articles SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_blog_posts_id AFTER INSERT ON blog_posts
WHEN NEW.id IS NULL
BEGIN UPDATE blog_posts SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_creative_works_id AFTER INSERT ON creative_works
WHEN NEW.id IS NULL
BEGIN UPDATE creative_works SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_media_items_id AFTER INSERT ON media_items
WHEN NEW.id IS NULL
BEGIN UPDATE media_items SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_music_tracks_id AFTER INSERT ON music_tracks
WHEN NEW.id IS NULL
BEGIN UPDATE music_tracks SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;

CREATE TRIGGER set_projects_id AFTER INSERT ON projects
WHEN NEW.id IS NULL
BEGIN UPDATE projects SET id = lower(hex(randomblob(16))) WHERE rowid = NEW.rowid; END;
