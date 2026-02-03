import json
import re

DATA_FILE = "frontend/src/data/journalism.json"

def main():
    with open(DATA_FILE, 'r') as f:
        data = json.load(f)

    print(f"Total articles before cleanup: {len(data)}")

    # content_html often has html tags, so we search strictly for the phrase
    # "files from Jake Wray" which seems to be the common denominator
    # variants seen: "With files from Jake Wray", "-With files from Jake Wray", "– With files from Jake Wray"
    
    clean_data = []
    removed_slugs = []

    for article in data:
        content = article.get('content_html', '')
        if re.search(r'files from.*Jake Wray', content, re.IGNORECASE):
            removed_slugs.append(article['slug'])
        else:
            clean_data.append(article)

    with open(DATA_FILE, 'w') as f:
        json.dump(clean_data, f, indent=2)

    print(f"Removed {len(removed_slugs)} articles:")
    for slug in removed_slugs:
        print(f" - {slug}")
        
    print(f"Total articles after cleanup: {len(clean_data)}")

if __name__ == "__main__":
    main()
