import requests
from bs4 import BeautifulSoup
import json
import os
import re
from datetime import datetime

TARGET_URL = "https://terracestandard.com/2025/11/13/ksi-lisims-bc-hydros-north-coast-transmission-line-to-be-fast-tracked-pm/"
DATA_FILE = "frontend/src/data/journalism.json"

def fetch_page(url):
    print(f"Fetching {url}...")
    headers = {'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36'}
    try:
        response = requests.get(url, headers=headers)
        response.raise_for_status()
        return response.text
    except Exception as e:
        print(f"Error fetching {url}: {e}")
        return None

def parse_article(url):
    html = fetch_page(url)
    if not html:
        return None

    soup = BeautifulSoup(html, 'html.parser')

    # Title
    title = "Untitled"
    h1 = soup.find('h1')
    if h1:
        title = h1.get_text(strip=True)
    
    # Date logic (reused from scraper)
    iso_date = ""
    display_date = ""
    now = datetime.now()
    
    meta_date = soup.find('meta', property='article:published_time')
    if meta_date:
        dt_str = meta_date.get('content', '')
        if 'T' in dt_str:
            dt_str = dt_str.split('T')[0]
        iso_date = dt_str
    
    if iso_date:
        try:
             dt = datetime.strptime(iso_date, '%Y-%m-%d')
             display_date = dt.strftime('%B %d, %Y')
        except ValueError:
             display_date = iso_date
    else:
        iso_date = now.strftime('%Y-%m-%d')
        display_date = now.strftime('%B %d, %Y')

    # CP Style replacements
    cp_replacements = {
        "January": "Jan.", "February": "Feb.", "August": "Aug.", 
        "September": "Sept.", "October": "Oct.", "November": "Nov.", "December": "Dec."
    }
    for full, abbrev in cp_replacements.items():
        display_date = display_date.replace(full, abbrev)

    # Content
    content_div = soup.select_one('#article_content') or soup.find('div', class_='entry-content') or soup.find('div', class_='article-body')

    images = []
    captions = []
    
    # Extract images from carousel (if any)
    slides = soup.select('.slides-container .slide')
    for slide in slides:
        img = slide.find('img')
        if img:
            src = img.get('src') or img.get('data-src')
            if src:
                if '?' in src: src = src.split('?')[0]
                if src not in images:
                    images.append(src)
                    caption_text = ""
                    caption_div = slide.find(class_='slide-caption')
                    if caption_div:
                         caption_text = caption_div.get_text(strip=True)
                    else:
                         caption_text = img.get('data-image-caption', '').strip()
                    captions.append(caption_text)

    content_html = ""
    excerpt = ""

    if content_div:
        # Cleanup
        for tag in content_div(['script', 'style', 'iframe', 'noscript', 'button', 'input', 'form']):
            tag.decompose()
        junk_selectors = ['.ad', '.share_buttons_group', '.related-posts', '.author-bio', '#cedato-unit']
        for selector in junk_selectors:
            for tag in content_div.select(selector):
                tag.decompose()
        
        # Remove the "With files from..." paragraph if it exists
        for p in content_div.find_all('p'):
            if "files from Jake Wray" in p.get_text():
                p.decompose()

        for img in content_div.find_all('img'):
            src = img.get('src') or img.get('data-src')
            if src:
                if '?' in src: src = src.split('?')[0]
                if src not in images:
                    images.append(src)

        content_html = "".join(str(c) for c in content_div.contents).strip()
        text = content_div.get_text(separator=' ', strip=True)
        excerpt = text[:300] + "..." if len(text) > 300 else text

    slug = url.strip('/').split('/')[-1]

    # Custom byline
    byline = "By Thom Barker and Jake Wray"

    return {
        "slug": slug,
        "title": title,
        "iso_date": iso_date,
        "display_date": display_date,
        "source_url": url,
        "content_html": content_html,
        "images": images,
        "captions": captions,
        "excerpt": excerpt,
        "byline": byline
    }

def main():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE, 'r') as f:
            data = json.load(f)
    else:
        data = []

    print(f"Restoring {TARGET_URL}...")
    article = parse_article(TARGET_URL)
    
    if article:
        # Remove if exists (to avoid duplicate if I ran it twice)
        data = [a for a in data if a['slug'] != article['slug']]
        data.append(article)
        
        # Sort
        data.sort(key=lambda x: x['iso_date'], reverse=True)
        
        with open(DATA_FILE, 'w') as f:
            json.dump(data, f, indent=2)
            
        print("Restored article with correct byline.")
    else:
        print("Failed to scrape article.")

if __name__ == "__main__":
    main()
