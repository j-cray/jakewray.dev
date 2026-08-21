-- Pages (Static / Standalone content pages like About Me)
CREATE TABLE pages (
    -- Uses MACRO: UUID_V4_GENERATOR
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6)))),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Trigger to auto-update updated_at on modification
CREATE TRIGGER update_pages_updated_at
AFTER UPDATE ON pages
FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
    UPDATE pages SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

-- Seed default About Me page content
INSERT INTO pages (slug, title, content)
VALUES (
    'about',
    'About Me',
    '<p class="mb-6">I am a journalist, developer, and photographer based in Northern British Columbia. I have a passion for uncovering stories that matter and documenting the world around me through both words and images.</p><p class="mb-6">Currently, I am expanding my horizons into software development, building tools and applications that bridge the gap between storytelling and technology. This website itself is a testament to that journey—a work in progress where I explore new ideas and showcase my evolving portfolio.</p><h3 class="text-2xl font-semibold mt-8 mb-4 text-gray-800">Journalism</h3><p class="mb-4">My reporting focuses on community issues, Indigenous culture, and public interest stories in the Terrace and Kitimat regions. I believe in the power of local journalism to inform communities and hold power to account.</p><h3 class="text-2xl font-semibold mt-8 mb-4 text-gray-800">Development</h3><p class="mb-4">As a developer, I am interested in Rust, web technologies, and building efficient, user-focused applications. I am currently working on several projects that integrate my diverse interests.</p>'
)
ON CONFLICT(slug) DO NOTHING;
