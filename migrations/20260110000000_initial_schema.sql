-- MACRO: UUID_V4_GENERATOR
-- Expression: (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6))))
-- Note: (random() & 3) + 1 provides perfectly uniform UUID variant bits.

-- Users (Admin)
CREATE TABLE users (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Articles (Journalism - Imported/Synced)
CREATE TABLE articles (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
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
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Personal Blog Posts
CREATE TABLE blog_posts (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content TEXT NOT NULL, -- Markdown/Rich Text
    published_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    tags TEXT, -- JSON Array
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Creative Writing (Stories, Novels, Poetry)
CREATE TABLE creative_works (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
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
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
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
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
    title TEXT NOT NULL,
    description TEXT,
    audio_url TEXT,
    embed_code TEXT, -- For Soundcloud/Spotify iframe
    published_at DATETIME DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Programming Projects
CREATE TABLE projects (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
    name TEXT NOT NULL,
    description TEXT,
    github_url TEXT,
    demo_url TEXT,
    technologies TEXT, -- JSON Array
    stars INT DEFAULT 0,
    is_featured BOOLEAN DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
