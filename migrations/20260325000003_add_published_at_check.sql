-- Add triggers to validate published_at datetime format on insert and update for articles and blog_posts

CREATE TRIGGER check_articles_published_at_insert
BEFORE INSERT ON articles
WHEN NEW.published_at IS NOT NULL AND NEW.published_at != strftime('%Y-%m-%dT%H:%M:%fZ', NEW.published_at)
BEGIN
    SELECT RAISE(ABORT, 'published_at must be in %Y-%m-%dT%H:%M:%fZ format');
END;

CREATE TRIGGER check_articles_published_at_update
BEFORE UPDATE ON articles
WHEN NEW.published_at IS NOT NULL AND NEW.published_at IS NOT OLD.published_at AND NEW.published_at != strftime('%Y-%m-%dT%H:%M:%fZ', NEW.published_at)
BEGIN
    SELECT RAISE(ABORT, 'published_at must be in %Y-%m-%dT%H:%M:%fZ format');
END;

CREATE TRIGGER check_blog_posts_published_at_insert
BEFORE INSERT ON blog_posts
WHEN NEW.published_at IS NOT NULL AND NEW.published_at != strftime('%Y-%m-%dT%H:%M:%fZ', NEW.published_at)
BEGIN
    SELECT RAISE(ABORT, 'published_at must be in %Y-%m-%dT%H:%M:%fZ format');
END;

CREATE TRIGGER check_blog_posts_published_at_update
BEFORE UPDATE ON blog_posts
WHEN NEW.published_at IS NOT NULL AND NEW.published_at IS NOT OLD.published_at AND NEW.published_at != strftime('%Y-%m-%dT%H:%M:%fZ', NEW.published_at)
BEGIN
    SELECT RAISE(ABORT, 'published_at must be in %Y-%m-%dT%H:%M:%fZ format');
END;
