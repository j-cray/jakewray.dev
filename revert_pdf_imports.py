
import json
import os

DATA_FILE = "frontend/src/data/journalism.json"

def revert_imports():
    if not os.path.exists(DATA_FILE):
        print("Data file not found.")
        return

    with open(DATA_FILE, 'r') as f:
        data = json.load(f)

    original_count = len(data)
    
    # Filter out entries where source_url starts with "file://"
    # These mark the locally imported PDF articles
    clean_data = [
        item for item in data 
        if not item.get('source_url', '').startswith("file://")
    ]
    
    removed_count = original_count - len(clean_data)
    
    with open(DATA_FILE, 'w') as f:
        json.dump(clean_data, f, indent=2)

    print(f"Reverted imports. Removed {removed_count} items. Remaining: {len(clean_data)}")

if __name__ == "__main__":
    revert_imports()
