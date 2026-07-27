#!/usr/bin/env python3
"""
Restore missing article captions from data/articles/*.json into SQLite database.
Only updates cover_image_caption if it is currently NULL or empty in the database,
ensuring existing captions are never overwritten.
"""

import glob
import json
import sqlite3

def restore_captions():
    db_path = 'sqlite.db'
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    json_files = glob.glob('data/articles/*.json')
    restored_count = 0
    skipped_existing = 0

    for filepath in json_files:
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                data = json.load(f)
        except Exception as e:
            print(f"Error reading {filepath}: {e}")
            continue

        slug = data.get('slug')
        captions = data.get('captions', [])

        if not slug or not captions or not any(captions):
            continue

        caption_text = captions[0].strip()
        if not caption_text:
            continue

        cursor.execute('SELECT cover_image_caption FROM articles WHERE slug = ?', (slug,))
        row = cursor.fetchone()

        if row:
            existing_caption = row[0]
            if existing_caption and existing_caption.strip():
                skipped_existing += 1
            else:
                cursor.execute(
                    'UPDATE articles SET cover_image_caption = ? WHERE slug = ?',
                    (caption_text, slug)
                )
                restored_count += 1

    conn.commit()
    conn.close()
    print(f"Successfully restored captions for {restored_count} articles.")
    print(f"Skipped {skipped_existing} articles with existing database captions.")

if __name__ == '__main__':
    restore_captions()
