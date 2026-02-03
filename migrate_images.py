import json
import os
import urllib.request
import subprocess
import shutil
import time
from urllib.parse import urlparse

# Configuration
DATA_FILE = "frontend/src/data/journalism.json"
GCS_BUCKET = "gs://jakewray-portfolio"
GCS_BASE_URL = "https://storage.googleapis.com/jakewray-portfolio"
MEDIA_PREFIX = "media/journalism"

def get_filename_from_url(url):
    parsed = urlparse(url)
    return os.path.basename(parsed.path)

def upload_to_gcs(local_path, destination_blob_name):
    # Use gsutil to interact with GCS
    # gsutil cp <local_path> gs://<bucket>/<destination_blob_name>
    gcs_uri = f"{GCS_BUCKET}/{destination_blob_name}"
    try:
        subprocess.run(["gsutil", "cp", local_path, gcs_uri], check=True, capture_output=True)
        return f"{GCS_BASE_URL}/{destination_blob_name}"
    except subprocess.CalledProcessError as e:
        print(f"Error uploading {local_path} to {gcs_uri}: {e}")
        return None

def main():
    if not os.path.exists(DATA_FILE):
        print(f"Data file {DATA_FILE} not found.")
        return

    with open(DATA_FILE, 'r') as f:
        data = json.load(f)

    updated_count = 0
    temp_dir = "temp_images"
    os.makedirs(temp_dir, exist_ok=True)

    for article in data:
        slug = article.get('slug', 'unknown')
        images = article.get('images', [])
        content_html = article.get('content_html', '')
        
        new_images = []
        modified = False

        # Process the 'images' list
        for img_url in images:
            if "storage.googleapis.com" in img_url:
                new_images.append(img_url)
                continue
            
            print(f"Migrating image for {slug}: {img_url}")
            
            # ... (migration logic) ...
            try:
                # Use urllib instead of requests
                with urllib.request.urlopen(img_url, timeout=10) as response:
                    filename = get_filename_from_url(img_url)
                    local_path = os.path.join(temp_dir, filename)
                    
                    with open(local_path, 'wb') as f:
                        shutil.copyfileobj(response, f)
                
                destination = f"{MEDIA_PREFIX}/{slug}/{filename}"
                new_url = upload_to_gcs(local_path, destination)
                
                if new_url:
                    new_images.append(new_url)
                    # Also replace in content_html
                    if img_url in content_html:
                        content_html = content_html.replace(img_url, new_url)
                    modified = True
                    updated_count += 1
                else:
                    new_images.append(img_url) # Keep original if upload fails
                    
                os.remove(local_path)
                
            except Exception as e:
                print(f"Failed to migrate {img_url}: {e}")
                new_images.append(img_url)

        # Scan content_html for any remaining legacy URLs (e.g. in data attributes)
        # Regex to find wp-content/uploads URLs
        import re
        legacy_urls = re.findall(r'(https://jakewrayportfolio\.wordpress\.com/wp-content/uploads/[^"\']+\.(?:jpg|jpeg|png|gif))', content_html)
        
        # Deduplicate
        legacy_urls = list(set(legacy_urls))
        
        for legacy_url in legacy_urls:
             # Skip if we just migrated it (though replace would have handled it, this covers cases not in 'images')
             if "storage.googleapis.com" in legacy_url: 
                 continue
                 
             print(f"Migrating embedded image for {slug}: {legacy_url}")
             try:
                with urllib.request.urlopen(legacy_url, timeout=10) as response:
                    filename = get_filename_from_url(legacy_url)
                    # Avoid overwriting processing
                    local_path = os.path.join(temp_dir, f"embedded_{filename}")
                    
                    with open(local_path, 'wb') as f:
                        shutil.copyfileobj(response, f)
                
                destination = f"{MEDIA_PREFIX}/{slug}/{filename}"
                new_url = upload_to_gcs(local_path, destination)
                
                if new_url:
                    content_html = content_html.replace(legacy_url, new_url)
                    modified = True
                    updated_count += 1
                
                if os.path.exists(local_path):
                    os.remove(local_path)
             except Exception as e:
                 print(f"Failed to migrate embedded {legacy_url}: {e}")

        if modified:
            article['images'] = new_images
            article['content_html'] = content_html

    # Clean up temp dir
    try:
        shutil.rmtree(temp_dir)
    except:
        pass

    if updated_count > 0:
        with open(DATA_FILE, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"Successfully migrated {updated_count} images.")
    else:
        print("No images needed migration.")

if __name__ == "__main__":
    main()
