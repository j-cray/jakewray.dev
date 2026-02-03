
import json
import os

DATA_FILE = "frontend/src/data/journalism.json"

def cleanup():
    if not os.path.exists(DATA_FILE):
        print(f"File {DATA_FILE} not found.")
        return

    with open(DATA_FILE, 'r') as f:
        data = json.load(f)

    # Filter out entries that look like they came from the local file import
    # The import script used "source_url": "file://..."
    original_count = len(data)
    clean_data = [item for item in data if not item.get('source_url', '').startswith('file://')]
    
    removed_count = original_count - len(clean_data)
    
    with open(DATA_FILE, 'w') as f:
        json.dump(clean_data, f, indent=2)

    print(f"Removed {removed_count} imported entries. Remaining: {len(clean_data)}")

if __name__ == "__main__":
    cleanup()
