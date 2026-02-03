
import json
import os
import subprocess
import fitz  # PyMuPDF
import sys
import hashlib

DATA_FILE = "frontend/src/data/journalism.json"
GCS_BUCKET = "gs://jakewray-portfolio"
GCS_BASE_URL = "https://storage.googleapis.com/jakewray-portfolio"
MEDIA_PREFIX = "media/journalism"

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
        # Use -r to exclude from public if needed? Previous script didn't use flags.
        # Assuming bucket defaults or manual set.
        subprocess.run(["gsutil", "cp", local_path, gcs_uri], check=True, capture_output=True)
        return f"{GCS_BASE_URL}/{destination_blob_name}"
    except subprocess.CalledProcessError as e:
        print(f"Error uploading {local_path} to {gcs_uri}: {e}")
        return None

def extract_and_upload_images():
    data = get_data()
    updated_count = 0
    
    # Create temp dir
    temp_dir = "temp_extracted_images"
    os.makedirs(temp_dir, exist_ok=True)

    for article in data:
        source_url = article.get('source_url', '')
        # Check if local PDF and no images
        if source_url.startswith("file://") and not article.get('images'):
            pdf_path = source_url.replace("file://", "")
            if not os.path.exists(pdf_path):
                # Try relative to imports if full path fails (bulk_extract might have saved rel or abs)
                if os.path.exists(os.path.join("imports", os.path.basename(pdf_path))):
                     pdf_path = os.path.join("imports", os.path.basename(pdf_path))
                else:
                    print(f"PDF not found: {pdf_path}")
                    continue

            slug = article.get('slug')
            title = article.get('title')
            
            print(f"Processing: {slug} from {os.path.basename(pdf_path)}")
            
            try:
                doc = fitz.open(pdf_path)
                
                # Find page with title
                target_page = None
                
                # First try to find title text
                for page_num, page in enumerate(doc):
                    text = page.get_text()
                    # Fuzzy match title? 
                    # Titles in PDF might be uppercase or split lines.
                    # Let's simple check if a significant part of title is in text.
                    title_snippet = title[:20] 
                    if title_snippet in text or title_snippet.upper() in text:
                         target_page = page
                         break
                
                # Fallback: if bulk_extract used filename date and title, maybe it's just page-by-page.
                # If we can't find title, maybe scan all pages for images? No, safer to skip or use first page if single page logic?
                # Bulk extract didn't save page num. 
                # Let's try to match by content snippet if title fails (excerpt might be better).
                if not target_page:
                    excerpt = article.get('excerpt', '')
                    if excerpt:
                        excerpt_snippet = excerpt[:30]
                        for page in doc:
                            if excerpt_snippet in page.get_text():
                                target_page = page
                                break

                if not target_page:
                    print(f"  -> Could not locate page for title: {title}")
                    continue

                # Extract images from target page
                image_list = target_page.get_images(full=True)
                
                new_images = []
                
                for img_index, img in enumerate(image_list):
                    xref = img[0]
                    base_image = doc.extract_image(xref)
                    image_bytes = base_image["image"]
                    ext = base_image["ext"]
                    
                    # Filter small images (logos, icons)
                    if len(image_bytes) < 15000: # < 15KB
                        continue
                    
                    # Filename: slug-imgIndex.ext
                    filename = f"{slug}-{img_index+1}.{ext}"
                    local_filepath = os.path.join(temp_dir, filename)
                    
                    with open(local_filepath, "wb") as img_file:
                        img_file.write(image_bytes)
                    
                    # Upload
                    destination = f"{MEDIA_PREFIX}/{slug}/{filename}"
                    print(f"  -> Uploading {filename}...")
                    public_url = upload_to_gcs(local_filepath, destination)
                    
                    if public_url:
                        new_images.append(public_url)
                        # Optionally insert into content_html if we want inline images
                    
                    # Cleanup local
                    os.remove(local_filepath)

                if new_images:
                    article['images'] = new_images
                    updated_count += 1
                    print(f"  -> Added {len(new_images)} images.")
                
            except Exception as e:
                print(f"  -> Error processing PDF: {e}")

    # Save
    if updated_count > 0:
        save_data(data)
        print(f"Updated {updated_count} articles with images.")
    else:
        print("No articles updated.")
        
    # Remove temp dir
    try:
        os.rmdir(temp_dir)
    except:
        pass

if __name__ == "__main__":
    extract_and_upload_images()
