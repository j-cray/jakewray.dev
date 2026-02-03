import os
import re
import json
import decimal
import shutil
import fitz  # PyMuPDF
import pdfplumber
from datetime import datetime

# Configuration
IMPORTS_DIR = "imports"
ASSETS_DIR = "assets/journalism/imported"
DATA_FILE = "frontend/src/data/journalism.json"
DRY_RUN = False  # Set to True to test without writing data

def get_existing_data():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE, 'r') as f:
            try:
                return json.load(f)
            except:
                return []
    return []

def article_exists(slug, data):
    return any(item.get('slug') == slug for item in data)

def extract_date_from_filename(filename):
    """
    Tries to extract date from filename like "Terrace Standard 11_06_2025 1.pdf"
    Returns iso_date string "YYYY-MM-DD" or None.
    """
    # Look for MM_DD_YYYY
    match = re.search(r'(\d{2})_(\d{2})_(\d{4})', filename)
    if match:
        month, day, year = match.groups()
        return f"{year}-{month}-{day}"
    return None

def extract_date_from_text(text):
    """
    Tries to find a date pattern in the first page text.
    Improvement: Could be expanded with more patterns.
    """
    # Very basic check for full date strings if needed
    return None

def format_cp_date(iso_date):
    if not iso_date:
        return ""
    try:
        dt = datetime.strptime(iso_date, '%Y-%m-%d')
        display_date = dt.strftime('%B %d, %Y')
        cp_replacements = {
            "January": "Jan.", "February": "Feb.", "August": "Aug.", 
            "September": "Sept.", "October": "Oct.", "November": "Nov.", "December": "Dec."
        }
        for full, abbrev in cp_replacements.items():
            display_date = display_date.replace(full, abbrev)
        return display_date
    except:
        return iso_date

def slugify(text):
    text = text.lower()
    text = re.sub(r'[^a-z0-9\s-]', '', text)
    text = re.sub(r'\s+', '-', text).strip('-')
    return text

def extract_images(pdf_path, slug):
    """
    Extracts images from PDF using PyMuPDF (fitz).
    Returns list of saved image filenames (relative to assets root ideally, but here just filenames).
    """
    doc = fitz.open(pdf_path)
    image_paths = []
    
    # We only care about the first page for the main visual usually, but let's grab all valid ones
    # Limit to reasonable number to avoid icon spam
    count = 0 
    
    for i in range(len(doc)):
        for img in doc.get_page_images(i):
            xref = img[0]
            base_image = doc.extract_image(xref)
            image_bytes = base_image["image"]
            ext = base_image["ext"]
            width = base_image["width"]
            height = base_image["height"]
            
            # Filter out small icons/logos
            if width < 200 or height < 200:
                continue
                
            image_filename = f"{slug}-{count}.{ext}"
            abs_path = os.path.join(ASSETS_DIR, image_filename)
            
            with open(abs_path, "wb") as f:
                f.write(image_bytes)
            
            # Construct public URL path - assuming assets/ is served or migrated
            # For now, we store the local path or a placeholder for the migration script to pick up
            # The prompt implies "import to jakewray.dev", usually meaning migrating to cloud or serving locally.
            # Let's stick to the convention: if it's in assets/, it might need migration. 
            # But the schema expects full URLs.
            # We will return the local relative path for now, enabling a later migration step.
            image_paths.append(image_filename)
            count += 1
            if count >= 5: # Limit images per article
                break
        if count >= 5:
            break
            
    return image_paths

def clean_text(text):
    """
    Fixes double-character encoding issues (e.g., 'BBLLAACCKK' -> 'BLACK').
    And normalizes whitespace.
    """
    if not text:
        return ""
        
    # Heuristic: if > 50% of words look like double-encoded, apply fix.
    # Or just fix specific words.
    # Actually, simpler aggression: if extracting specific key phrases fails, try deduplication.
    
    # Let's try a regex for double chars: aa bb cc...
    # But 'book' has double o. 'keeper' has double e.
    # 'BBLLAACCKK' -> B L A C K.  Indices: 0, 2, 4, 6, 8.
    # If text is consistently double chars, len is even.
    
    # We can try to see if the string equals itself sliced [::2] and [1::2] ?
    # i.e. text[0] == text[1], text[2] == text[3]...
    
def is_double_encoded(s):
    if len(s) < 4: return False
    if len(s) % 2 != 0: return False
    # Check if every pair is identical
    for i in range(0, len(s), 2):
        if s[i] != s[i+1]:
            return False
    return True

def clean_text(text):
    """
    Fixes double-character encoding issues (e.g., 'BBLLAACCKK' -> 'BLACK').
    And normalizes whitespace.
    """
    if not text:
        return ""
        
    words = text.split()
    cleaned_words = []
    for w in words:
        if is_double_encoded(w):
            cleaned_words.append(w[::2])
        else:
            cleaned_words.append(w)
            
    return " ".join(cleaned_words)

def parse_pdf_page(pdf_path, filename, page_num, pdf_obj):
    # pdf_obj is the pdfplumber object
    page = pdf_obj.pages[page_num]
    text = page.extract_text()
    
    if not text:
        return None
        
    cleaned_text = clean_text(text)
    
    # 2. Author Check
    has_byline = "jake wray" in cleaned_text.lower() or "jake wray" in text.lower()
    
    if not has_byline:
        return None
        
    # Extract words with style for Headline detection
    words = page.extract_words(extra_attrs=['fontname', 'size'])
    words_sorted = sorted(words, key=lambda x: x['size'], reverse=True)

    # 3. Metadata Extraction
    title = "Untitled"
    candidate_titles = []
    
    for w in words_sorted:
        w_text = w['text']
        if is_double_encoded(w_text):
            w_text = w_text[::2]
            
        # Filter junk
        if len(w_text) < 3: continue
        # Filter masthead noise
        if "STANDARD" in w_text or "LAKES" in w_text or "DISTRICT" in w_text or "NEWS" in w_text or "TANDARD" in w_text:
            continue
        if "SALE" in w_text or "BLACK" in w_text: # Ad filtering
             continue
        if "FURNITURE" in w_text or "MATTRESSES" in w_text or "APPLIANCES" in w_text:
             continue
        if "News" in w_text or "Updates" in w_text: # Common headers
            continue
             
        # Construct line from this word's vertical position
        line_words = [word for word in words if abs(word['top'] - w['top']) < 5]
        line_words.sort(key=lambda x: x['x0'])
        
        line_str_parts = []
        for lw in line_words:
             lw_text = lw['text']
             if is_double_encoded(lw_text):
                 lw_text = lw_text[::2]
             line_str_parts.append(lw_text)
        
        line_str = " ".join(line_str_parts)
        
        if line_str not in candidate_titles:
             candidate_titles.append(line_str)
             
        if len(candidate_titles) >= 3:
            break
            
    if candidate_titles:
        title = candidate_titles[0]
        
    # Heuristic: Valid headlines usually have multiple words.
    if len(title.split()) < 3:
        # "Village of" -> 2 words. "STEAK" -> 1 word.
        return None
        
    iso_date = extract_date_from_filename(filename)
    if not iso_date:
        iso_date = datetime.now().strftime('%Y-%m-%d')
        
    display_date = format_cp_date(iso_date)
    # Add page number to slug to differentiate multiple articles in one issue
    slug = slugify(f"{title}-{iso_date}-{page_num+1}")
    
    # 4. Content Formatting
    lines = cleaned_text.split('\n')
    content_html = ""
    for line in lines:
        if not line.strip(): 
            continue
        content_html += f"<p>{line.strip()}</p>"
        
    # 5. Image Extraction (for specific page)
    # Note: We need fitz for image extraction, separate from pdfplumber
    saved_images = extract_images_from_page(pdf_path, slug, page_num)
    image_entries = [f"LOCAL:{img}" for img in saved_images]
    
    return {
        "slug": slug,
        "title": title,
        "iso_date": iso_date,
        "display_date": display_date,
        "source_url": f"file://{filename}#page={page_num+1}", 
        "content_html": content_html,
        "images": image_entries,
        "captions": [],
        "byline": "By Jake Wray",
        "excerpt": cleaned_text[:200] + "..."
    }

def extract_images_from_page(pdf_path, slug, page_num):
    doc = fitz.open(pdf_path)
    if page_num >= len(doc):
        return []
        
    image_paths = []
    count = 0
    
    for img in doc.get_page_images(page_num):
        xref = img[0]
        base_image = doc.extract_image(xref)
        image_bytes = base_image["image"]
        ext = base_image["ext"]
        width = base_image["width"]
        height = base_image["height"]
        
        if width < 200 or height < 200:
            continue
            
        image_filename = f"{slug}-{count}.{ext}"
        abs_path = os.path.join(ASSETS_DIR, image_filename)
        
        with open(abs_path, "wb") as f:
            f.write(image_bytes)
        
        image_paths.append(image_filename)
        count += 1
        if count >= 5:
            break
            
    return image_paths

def main():
    if not os.path.exists(ASSETS_DIR):
        os.makedirs(ASSETS_DIR)
        
    all_articles = get_existing_data()
    existing_slugs = {item.get('slug') for item in all_articles}
    
    new_articles = []
    
    files = sorted([f for f in os.listdir(IMPORTS_DIR) if f.lower().endswith('.pdf')])
    
    for filename in files:
        pdf_path = os.path.join(IMPORTS_DIR, filename)
        print(f"Scanning {filename}...")
        
        try:
            with pdfplumber.open(pdf_path) as pdf:
                for req_page_num, page in enumerate(pdf.pages):
                    try:
                        article = parse_pdf_page(pdf_path, filename, req_page_num, pdf)
                        
                        if not article:
                            continue
                            
                        # If title is suspiciously generic, maybe skip?
                        if len(article['title']) < 4:
                            continue

                        if article['slug'] in existing_slugs:
                            print(f"  -> [Page {req_page_num+1}] Skipping duplicate slug: {article['slug']}")
                            continue
                            
                        print(f"  -> [Page {req_page_num+1}] Found: {article['title']}")
                        new_articles.append(article)
                        existing_slugs.add(article['slug'])
                        
                    except Exception as e_page:
                        print(f"  -> Error on page {req_page_num+1}: {e_page}")

        except Exception as e:
            print(f"  -> Error processing file {filename}: {e}")
            
            
    if not new_articles:
        print("No new articles found.")
        return

    print(f"Found {len(new_articles)} new articles.")
    
    if DRY_RUN:
        print("Dry run finished. No files written.")
    else:
        # Append and sort
        all_articles.extend(new_articles)
        all_articles.sort(key=lambda x: x.get('iso_date', ''), reverse=True)
        
        with open(DATA_FILE, 'w') as f:
            json.dump(all_articles, f, indent=2)
        print(f"Updated {DATA_FILE}")

if __name__ == "__main__":
    main()
