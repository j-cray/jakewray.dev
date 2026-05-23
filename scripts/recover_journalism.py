#!/usr/bin/env python3
"""
Recover historical journalism articles from the pre-cleanup Git history
and import them directly into the local SQLite database.

This script uses only standard library modules (no pip dependencies required).
"""

import json
import os
import sqlite3
import urllib.request
import uuid

def main():
    # The parent commit before we cleared journalism.json
    commit_sha = "878e3b88466e70572434fb837bfa26b8944d7a02"
    url = f"https://raw.githubusercontent.com/j-cray/jakewray.dev/{commit_sha}/frontend/src/data/journalism.json"
    
    print("⏳ Downloading original journalism.json from Git history...")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req) as response:
            data = response.read().decode("utf-8")
        articles = json.loads(data)
    except Exception as e:
        print(f"❌ Error downloading or parsing JSON: {e}")
        return

    print(f"✅ Found {len(articles)} articles. Connecting to SQLite...")
    
    db_path = "sqlite.db"
    if not os.path.exists(db_path):
        # Create an empty sqlite.db if it is missing
        print(f"⚠️  {db_path} not found. Creating a fresh database file...")
        open(db_path, "w").close()

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Ensure the articles table exists
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS articles (
            id TEXT PRIMARY KEY,
            wp_id BIGINT UNIQUE,
            slug TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            subtitle TEXT,
            excerpt TEXT,
            content TEXT NOT NULL,
            cover_image_url TEXT,
            author TEXT NOT NULL,
            published_at DATETIME NOT NULL,
            origin TEXT NOT NULL DEFAULT 'local',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    imported_count = 0
    for a in articles:
        uid = str(uuid.uuid4())
        
        # Check both potential date keys (older entries had 'date', newer had 'iso_date')
        date_str = a.get("iso_date") or a.get("date")
        if not date_str:
            print(f"⚠️  Skipping article without date: {a.get('title')}")
            continue
            
        published_at = f"{date_str}T00:00:00.000Z"
        cover_image = a.get("images")[0] if a.get("images") else None
        byline = a.get("byline", "Jake Wray")
        
        cursor.execute("""
            INSERT INTO articles (id, slug, title, content, excerpt, cover_image_url, author, published_at, origin)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'imported')
            ON CONFLICT(slug) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                excerpt = excluded.excerpt,
                cover_image_url = excluded.cover_image_url,
                author = excluded.author,
                published_at = excluded.published_at,
                updated_at = CURRENT_TIMESTAMP
        """, (
            uid,
            a["slug"],
            a["title"],
            a["content_html"],
            a["excerpt"],
            cover_image,
            byline,
            published_at
        ))
        imported_count += 1
        
    conn.commit()
    conn.close()
    print(f"🎉 Successfully imported {imported_count} articles directly into {db_path}!")

if __name__ == "__main__":
    main()
