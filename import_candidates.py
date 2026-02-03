
import json
import os
import fitz # PyMuPDF
import subprocess
import hashlib
from datetime import datetime

CANDIDATES_FILE = "smart_candidates.json"
DATA_FILE = "frontend/src/data/journalism.json"
MEDIA_PREFIX = "media/journalism"
GCS_BUCKET = "gs://jakewray-portfolio"
GCS_BASE_URL = "https://storage.googleapis.com/jakewray-portfolio"

def get_data():
    if not os.path.exists(DATA_FILE):
        return []
    with open(DATA_FILE, 'r') as f:
        return json.load(f)

def save_data(data):
    with open(DATA_FILE, 'w') as f:
        json.dump(data, f, indent=2)

def upload_to_gcs(local_path, destination_blob_name):
    gcs_uri = f"{GCS_BUCKET}/{destination_blob_name}"
    try:
        subprocess.run(["gsutil", "cp", local_path, gcs_uri], check=True, capture_output=True, timeout=60)
        return f"{GCS_BASE_URL}/{destination_blob_name}"
    except subprocess.TimeoutExpired:
        print(f"Timeout uploading {local_path}")
        return None
    except subprocess.CalledProcessError as e:
        print(f"Error uploading {local_path}: {e}")
        return None

def import_candidates():
    if not os.path.exists(CANDIDATES_FILE):
        print("Candidates file not found.")
        return

    with open(CANDIDATES_FILE, 'r') as f:
        candidates = json.load(f)

    existing_data = get_data()
    existing_slugs = {item.get('slug') for item in existing_data}
    
    new_articles = []
    
    # Temp dir for images
    os.makedirs("temp_import_images", exist_ok=True)
    
    print(f"Processing {len(candidates)} candidates...")
    
    for c in candidates:
        headline = c.get('headline', '').strip()
        slug = c['slug']
        
        # Filters
        if len(headline) < 10:
            print(f"Skipping short headline: {headline}")
            continue
        if "www." in headline or ".com" in headline:
             print(f"Skipping URL-like headline: {headline}")
             continue
        if headline.upper() in ["TERRACE STANDARD", "LAKES DISTRICT NEWS", "TANDARD"]:
             print(f"Skipping masthead headline: {headline}")
             continue

        if slug in existing_slugs:
            print(f"Skipping duplicate slug: {slug}")
            continue
            
        print(f"Importing: {headline}...")
        
        # 1. Extract Images
        # We need to re-open PDF with fitz to get images
        pdf_path = os.path.join("imports", c['filename'])
        images = []
        
        try:
            doc = fitz.open(pdf_path)
            # page_num is 1-indexed in json
            page = doc.load_page(c['page'] - 1)
            
            img_list = page.get_images(full=True)
            for i, img in enumerate(img_list):
                xref = img[0]
                base = doc.extract_image(xref)
                image_bytes = base["image"]
                ext = base["ext"]
                
                # Filter small
                if len(image_bytes) < 15000: continue
                
                filename = f"{slug}-{i+1}.{ext}"
                local_path = os.path.join("temp_import_images", filename)
                
                with open(local_path, "wb") as f_img:
                    f_img.write(image_bytes)
                
                # Upload
                # public_url = upload_to_gcs(local_path, f"{MEDIA_PREFIX}/{slug}/{filename}")
                # For now, let's just use local path placeholder or skip upload to speed up logic testing?
                # User wants "1 (minimum) image".
                # I will uncomment upload for production.
                # But to avoid previous hang, I'll print the command first.
                
                print(f"  -> Uploading {filename}...")
                public_url = upload_to_gcs(local_path, f"{MEDIA_PREFIX}/{slug}/{filename}")
                
                if public_url:
                    images.append(public_url)
                
                os.remove(local_path)
                
        except Exception as e:
            print(f"  -> Error extracting images: {e}")

        # 2. Build Article Object
        article = {
            "slug": slug,
            "title": c['headline'],
            "date": c['date'],
            "display_date": datetime.strptime(c['date'], "%Y-%m-%d").strftime("%B %d, %Y") if c['date'] != "Unknown" else "Unknown",
            "source_url": f"file://imports/{c['filename']}",
            "excerpt": c['excerpt'],
            "content_html": "<p>" + c['full_text'].replace("\n\n", "</p><p>") + "</p>", # Simple formatting
            "images": images,
            "byline": "Jake Wray", # Verified by strict check
            "tags": ["journalism", "archive"]
        }
        
        new_articles.append(article)
        existing_slugs.add(slug)

    # Save
    if new_articles:
        existing_data.extend(new_articles)
        # Sort by date desc
        try:
            existing_data.sort(key=lambda x: x.get('date', '0000-00-00'), reverse=True)
        except:
            pass
            
        save_data(existing_data)
        print(f"\nSuccessfully imported {len(new_articles)} articles.")
    else:
        print("\nNo new articles imported.")
        
    try:
        os.rmdir("temp_import_images")
    except:
        pass

if __name__ == "__main__":
    import_candidates()
