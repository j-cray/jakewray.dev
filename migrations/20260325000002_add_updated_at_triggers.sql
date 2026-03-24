-- Add AFTER UPDATE triggers for core tables to auto-update the updated_at column

CREATE TRIGGER update_articles_updated_at
AFTER UPDATE ON articles
FOR EACH ROW
BEGIN
    UPDATE articles SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

CREATE TRIGGER update_blog_posts_updated_at
AFTER UPDATE ON blog_posts
FOR EACH ROW
BEGIN
    UPDATE blog_posts SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

CREATE TRIGGER update_creative_works_updated_at
AFTER UPDATE ON creative_works
FOR EACH ROW
BEGIN
    UPDATE creative_works SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

CREATE TRIGGER update_media_items_updated_at
AFTER UPDATE ON media_items
FOR EACH ROW
BEGIN
    UPDATE media_items SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

CREATE TRIGGER update_music_tracks_updated_at
AFTER UPDATE ON music_tracks
FOR EACH ROW
BEGIN
    UPDATE music_tracks SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;

CREATE TRIGGER update_projects_updated_at
AFTER UPDATE ON projects
FOR EACH ROW
BEGIN
    UPDATE projects SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = NEW.id;
END;
