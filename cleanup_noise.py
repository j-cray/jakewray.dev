
import json
import os

DATA_FILE = "frontend/src/data/journalism.json"

def cleanup():
    if not os.path.exists(DATA_FILE):
        return

    with open(DATA_FILE, 'r') as f:
        data = json.load(f)

    original_count = len(data)
    
    # Filter out noise
    clean_data = []
    for item in data:
        title = item.get('title', '')
        if "PUBLISHED BY" in title:
            continue
        if "w L ww a" in title:
            continue
        if len(title) < 5: # Too short
            continue
            
        clean_data.append(item)
    
    removed_count = original_count - len(clean_data)
    
    with open(DATA_FILE, 'w') as f:
        json.dump(clean_data, f, indent=2)

    print(f"Removed {removed_count} noise entries. Remaining: {len(clean_data)}")

if __name__ == "__main__":
    cleanup()
