
import pdfplumber
import os
import re
import json
from datetime import datetime

IMPORTS_DIR = "imports"
OUTPUT_JSON = "smart_candidates.json"

def get_files():
    return [f for f in os.listdir(IMPORTS_DIR) if f.endswith(".pdf")]

def clean_text(text):
    if not text: return ""
    # Remove hyphenation at end of lines
    text = re.sub(r'-\n', '', text)
    # Replace newlines with spaces
    text = re.sub(r'\n', ' ', text)
    return text.strip()

def analyze_page(pdf_path, filename, page, page_num):
    candidates = []
    width, height = page.width, page.height
    words = page.extract_words()
    
    # 1. Find Anchor (Jake Wray)
    anchors = []
    for w in words:
        if "Jake" in w['text'] and "Wray" in w['text']:
            anchors.append(w)
        elif "Jake" == w['text']:
            # Check next word
            idx = words.index(w)
            if idx + 1 < len(words) and "Wray" in words[idx+1]['text']:
                # Merge
                w['text'] = "Jake Wray"
                w['x1'] = words[idx+1]['x1']
                anchors.append(w)

    for anchor in anchors:
        # Strict Byline Check
        # 1. Must be on the left (or reasonably aligned, typically < 40% of page width)
        # Actually, some bylines are centered?
        # Let's check Context: Is the line just "Jake Wray" or "By Jake Wray"?
        
        # Get line context (words with same/similar top)
        line_words = [w for w in words if abs(w['top'] - anchor['top']) < 3]
        line_text = " ".join([w['text'] for w in line_words])
        
        # Filter exclusions
        if "with files from" in line_text.lower():
            continue
        if "photo" in line_text.lower() or "credit" in line_text.lower():
            # Likely photo credit
            continue
            
        # If text is very long, it's likely a paragraph where his name appears.
        # Bylines are short.
        if len(line_text) > 50: 
             continue

        # 2. Find Headline (Above anchor, Large Text)
        headline = None
        headline_top = 0
        
        # Scan words above anchor
        # Look for the largest font size in the 200pts above
        search_zone_top = max(0, anchor['top'] - 300)
        above_words = [w for w in words if search_zone_top < w['top'] < anchor['top']]
        
        if not above_words:
            continue
            
        # Group by line/block to find headline
        # Heuristic: Find max font size
        max_size = 0
        for w in above_words:
            if (w['bottom'] - w['top']) > max_size:
                max_size = (w['bottom'] - w['top'])
        
        # Threshold: Headline should be significantly larger than byline (usually ~8pt)
        # Let's say Headline > 14pt
        if max_size < 14:
            continue # No headline found
            
        # Get all words with that max size (or close to it) in the zone
        headline_words = [w for w in above_words if (w['bottom'] - w['top']) > (max_size * 0.9)]
        # Sort by top/left
        headline_words.sort(key=lambda x: (x['top'], x['x0']))
        
        headline_text = " ".join([w['text'] for w in headline_words])
        headline_top = headline_words[0]['top']
        
        # 3. Extract Body (Below anchor)
        # We need to crop the page to get text *below* the byline
        # And stop where? Maybe the whole rest of page? 
        # Or simplistic: Get all text below anchor['bottom'] + 5
        
        # Use crop to extract text
        crop_box = (0, anchor['bottom'] + 5, width, height) 
        # Note: pdfplumber crop uses (x0, top, x1, bottom)
        try:
            cropped = page.crop(crop_box)
            body_text = cropped.extract_text()
        except:
             body_text = ""
        
        clean_body = clean_text(body_text)
        word_count = len(clean_body.split())
        
        if word_count < 200:
            continue
            
        if word_count > 2000: # Too long? Maybe capturing whole page ads?
             pass # Let's accept for now, better to extract too much than too little body
             
        # 4. Images
        # Check for images between headline top and some reasonable bottom
        # Or just any image on page? User said "1 (minimum) image".
        # Let's grab images in the "Article Zone" (Headline Top -> Page Bottom)
        article_images = [
            img for img in page.images 
            if img['top'] > headline_top - 50 # Allow image slightly above headline
        ]
        
        if not article_images:
            # Maybe strict "1 image" is too harsh? User said "essential... if template incomplete then don't have article"
            # But also "images lowest priority". 
            # I'll flag it but maybe keep it?
            pass

        # 5. Extract Date from filename
        # Pattern: ...YYYY_MM_DD... or ...MM_DD_YYYY...
        # My files: Terrace Standard 01_29_2026.pdf
        date_str = "Unknown"
        match = re.search(r'(\d{2})_(\d{2})_(\d{4})', filename)
        if match:
            m, d, y = match.groups()
            date_str = f"{y}-{m}-{d}"
            
        candidates.append({
            "headline": headline_text,
            "byline_context": line_text,
            "filename": filename,
            "page": page_num + 1,
            "word_count": word_count,
            "image_count": len(article_images),
            "date": date_str,
            "excerpt": clean_body[:200],
            "full_text": clean_body,
            "slug": headline_text.lower().replace(" ", "-")[:50] # Check existing later
        })

    return candidates

def main():
    files = get_files()
    all_candidates = []
    
    print(f"Scanning {len(files)} files...")
    
    for filename in files:
        path = os.path.join(IMPORTS_DIR, filename)
        try:
            with pdfplumber.open(path) as pdf:
                for i, page in enumerate(pdf.pages):
                     # Search fast for string first?
                     text = page.extract_text() or ""
                     if "Jake" in text and "Wray" in text:
                         print(f"  -> Analyzing {filename} Page {i+1}...")
                         matches = analyze_page(path, filename, page, i)
                         if matches:
                             print(f"     Found {len(matches)} candidates.")
                             all_candidates.extend(matches)
        except Exception as e:
            print(f"Error reading {filename}: {e}")

    # Remove duplicates (same headline)
    unique_candidates = {}
    for c in all_candidates:
        unique_candidates[c['headline']] = c
        
    final_list = list(unique_candidates.values())
    
    print(f"\nFound {len(final_list)} valid candidates.")
    
    with open(OUTPUT_JSON, 'w') as f:
        json.dump(final_list, f, indent=2)
        
    print(f"Saved candidates to {OUTPUT_JSON}")

if __name__ == "__main__":
    main()
