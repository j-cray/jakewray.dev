-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users (Admin)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Articles (Journalism - Imported/Synced)
CREATE TABLE articles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wp_id BIGINT UNIQUE, -- External ID from WordPress
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    subtitle TEXT,
    excerpt TEXT,
    content TEXT NOT NULL, -- HTML content
    cover_image_url TEXT,
    author TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    origin TEXT NOT NULL DEFAULT 'local', -- 'imported', 'synced', 'local'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Personal Blog Posts
CREATE TABLE blog_posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content TEXT NOT NULL, -- Markdown/Rich Text
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Creative Writing (Stories, Novels, Poetry)
CREATE TYPE creative_type AS ENUM ('story', 'novel', 'poetry');
CREATE TABLE creative_works (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    work_type creative_type NOT NULL,
    synopsis TEXT,
    content TEXT, -- Full text or chapters (can be JSON if complex)
    status TEXT NOT NULL DEFAULT 'published', -- 'draft', 'published'
    published_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Media Items (Photography, Visual Art, J-School Video, Videography)
CREATE TYPE media_category AS ENUM ('photography', 'visual_art', 'video', 'j_school');
CREATE TYPE media_context AS ENUM ('personal', 'professional');

CREATE TABLE media_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT,
    description TEXT,
    url TEXT NOT NULL, -- S3 URL or local path
    thumbnail_url TEXT,
    category media_category NOT NULL,
    context media_context NOT NULL DEFAULT 'personal', -- To distinguish Photojournalism (prof) vs Personal
    taken_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Music
CREATE TABLE music_tracks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    description TEXT,
    audio_url TEXT,
    embed_code TEXT, -- For Soundcloud/Spotify iframe
    published_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Programming Projects
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    github_url TEXT,
    demo_url TEXT,
    technologies TEXT[],
    stars INT DEFAULT 0,
    is_featured BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
