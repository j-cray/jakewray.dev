
import json
import os
import re
import sys
import pdfplumber
from datetime import datetime

DATA_FILE = "frontend/src/data/journalism.json"
MATCH_REPORT = "matches.txt"

def get_existing_data():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE, 'r') as f:
            try:
                return json.load(f)
            except:
                return []
    return []

def save_data(data):
    # Sort by date
    data.sort(key=lambda x: x.get('iso_date', ''), reverse=True)
    with open(DATA_FILE, 'w') as f:
        json.dump(data, f, indent=2)
    print(f"Saved {len(data)} articles to {DATA_FILE}")

def parse_matches():
    matches = []
    if not os.path.exists(MATCH_REPORT):
        print(f"Report {MATCH_REPORT} not found.")
        return matches

    with open(MATCH_REPORT, 'r') as f:
        for line in f:
            if line.startswith("MATCH:"):
                # Format: MATCH: <filename> | Page <num> | Context: ...
                parts = line.split("|")
                if len(parts) >= 2:
                    filename_part = parts[0].replace("MATCH:", "").strip()
                    page_part = parts[1].strip()
                    # Extract page number
                    try:
                        page_num = int(page_part.replace("Page", "").strip())
                        # Construct full path (filename includes path relative to extraction root usually, 
                        # but matches.txt has "imports/..." usually or just filename?)
                        # The match report output was: MATCH: imports/Burns... 5.pdf | ...
                        # So filename_part is likely "imports/..." 
                        
                        # Let's clean the path. 
                        # The report line: MATCH: Burns Lake ... 
                        # Wait, looking at the view_file output: 
                        # MATCH: Burns Lake ... 03_19_2025 5.pdf | Page 5 ...
                        # It seems scan_report.py printed os.path.basename(path).
                        # So we need to reconstruct the full path.
                        
                        full_path = os.path.join("imports", filename_part)
                        matches.append((full_path, page_num, filename_part))
                    except ValueError:
                        continue
    return matches

def extract_date_from_filename(filename):
    # Format: ... 03_19_2025 5.pdf
    # Regex for MM_DD_YYYY
    match = re.search(r"(\d{2})_(\d{2})_(\d{4})", filename)
    if match:
        return f"{match.group(3)}-{match.group(1)}-{match.group(2)}"
    return "2025-01-01" # Fallback

def format_display_date(iso_date):
    try:
        dt = datetime.strptime(iso_date, "%Y-%m-%d")
        return dt.strftime("%b. %d, %Y")
    except:
        return iso_date

def process_page(pdf_path, page_num_1_indexed, filename):
    try:
        with pdfplumber.open(pdf_path) as pdf:
            page_idx = page_num_1_indexed - 1
            if page_idx >= len(pdf.pages):
                return None
            
            page = pdf.pages[page_idx]
            text = page.extract_text()
            if not text:
                return None

            # Heuristics
            # 1. Skip Masthead pages (often Page 4) if content is sparse or just listing staff
            if "MULTI-MEDIA JOURNALIST: JAKE WRAY" in text and len(text) < 500:
                print(f"Skipping potential masthead: {filename} p{page_num_1_indexed}")
                return None

            # 2. Extract Title (First non-empty line, simplistic)
            lines = [l.strip() for l in text.split('\n') if l.strip()]
            title = lines[0] if lines else "Untitled"
            
            # Simple cleanup for title
            if len(title) < 5 or "Burns Lake Lakes District News" in title:
                # Try next line
                if len(lines) > 1:
                    title = lines[1]

            # 3. Content
            # Wrap as parsing
            content_html = ""
            for line in lines[2:]: # Skip first couple potentially
                content_html += f"<p>{line}</p>"

            slug = title.lower().replace(' ', '-').replace("'", "").replace(",", "")[:50]
            iso_date = extract_date_from_filename(filename)
            
            # Uniquify slug
            slug = f"{slug}-{iso_date}"

            return {
                "slug": slug,
                "title": title,
                "iso_date": iso_date,
                "display_date": format_display_date(iso_date),
                "source_url": f"file://{pdf_path}", # Marker
                "content_html": content_html,
                "images": [],
                "captions": [],
                "byline": "By Jake Wray",
                "excerpt": lines[2] if len(lines) > 2 else "..."
            }

    except Exception as e:
        print(f"Error processing {filename} p{page_num_1_indexed}: {e}")
        return None

def main():
    print("Reading match report...")
    matches = parse_matches()
    print(f"Found {len(matches)} candidate pages.")

    existing_data = get_existing_data()
    existing_slugs = {item['slug'] for item in existing_data}
    
    new_articles = []
    
    for full_path, page_num, filename in matches:
        # Check if file exists
        if not os.path.exists(full_path):
            # Try to match fuzzy? scan_report printed basename.
            # If valid path not found, skip or try to find in imports dir
            # scan_report output: MATCH: imports/Burns... (lines 1-94 showed "imports/" prefix in `view_file` output previously?
            # Let's re-read matches.txt logic.
            # The view_file output showed: 
            # 2: MATCH: Burns Lake ... 03_19_2025 5.pdf | Page 5 ...
            # Wait, line 2 DOES NOT have "imports/".
            # BUT line 94 in view_file showed:
            # 1: Generating Match Report...
            # 2: MATCH: Burns Lake ...
            # Wait, I previously ran `run_command` with `scraper_venv/bin/python3 scan_report.py imports/*.pdf`.
            # `scan_report.py` code: `print(f"MATCH: {os.path.basename(path)} | ...`
            # SO it prints BASENAME.
            # My logic `full_path = os.path.join("imports", filename_part)` should be correct.
            pass

        article = process_page(full_path, page_num, filename)
        if article:
            # Check for duplicates or very similar slugs
            if article['slug'] not in existing_slugs:
                new_articles.append(article)
                existing_slugs.add(article['slug'])
                print(f"Importer: {article['title']} ({article['iso_date']})")

    if new_articles:
        existing_data.extend(new_articles)
        save_data(existing_data)
        print(f"Successfully imported {len(new_articles)} new articles.")
    else:
        print("No new articles found.")

if __name__ == "__main__":
    main()
