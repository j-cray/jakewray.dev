
import json
import os
import re

SOURCE_FILE = "frontend/src/data/journalism.json"
TARGET_DIR = "data/articles"

def migrate():
    if not os.path.exists(SOURCE_FILE):
        print(f"Source file {SOURCE_FILE} not found.")
        return

    os.makedirs(TARGET_DIR, exist_ok=True)

    with open(SOURCE_FILE, 'r') as f:
        articles = json.load(f)

    print(f"Found {len(articles)} articles.")

    for article in articles:
        slug = article.get('slug')
        if not slug:
            print("Skipping article without slug")
            continue
        
        # Ensure safe filenames
        safe_slug = re.sub(r'[^a-z0-9-]', '', slug.lower())
        target_path = os.path.join(TARGET_DIR, f"{safe_slug}.json")
        
        with open(target_path, 'w') as f:
            json.dump(article, f, indent=2)
            
    print(f"Migrated {len(articles)} articles to {TARGET_DIR}")

if __name__ == "__main__":
    migrate()
