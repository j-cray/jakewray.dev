import requests
from bs4 import BeautifulSoup
import json
import os
import time
from datetime import datetime
import re

# Configuration
BASE_URLS = [
    "https://terracestandard.com/author/jakewray/",
    "https://terracestandard.com/author/jakewray/page/2/"
]
DATA_FILE = "frontend/src/data/journalism.json"

def get_existing_slugs(data):
    return {item['slug'] for item in data}

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

def extract_article_urls(soup):
    links = []
    # Identify article links
    for article in soup.find_all('article'):
        link = article.find('a', href=True)
        if link:
            links.append(link['href'])
    
    # Fallback
    if not links:
        main = soup.find('main') or soup.find('body')
        for link in main.find_all('a', href=True):
            href = link['href']
            # Basic filter: must have year and be on domain
            if '/20' in href and 'terracestandard.com' in href: 
                links.append(href)
    
    return list(set(links))

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
    else:
        h2_headline = soup.select_one('h2.headline')
        if h2_headline:
            title = h2_headline.get_text(strip=True)
        else:
            meta_title = soup.find('meta', property='og:title')
            if meta_title:
                title = meta_title.get('content', '').split(' - ')[0]

    # Date
    iso_date = ""
    display_date = ""
    
    meta_date = soup.find('meta', property='article:published_time')
    if meta_date:
        dt_str = meta_date.get('content', '')
        if 'T' in dt_str:
            dt_str = dt_str.split('T')[0]
        iso_date = dt_str
    
    if not iso_date:
        match = re.search(r'/(\d{4})/(\d{2})/(\d{2})/', url)
        if match:
            iso_date = f"{match.group(1)}-{match.group(2)}-{match.group(3)}"

    if iso_date:
        try:
            dt = datetime.strptime(iso_date, '%Y-%m-%d')
            display_date = dt.strftime('%B %d, %Y')
        except ValueError:
            display_date = iso_date
    else:
        now = datetime.now()
        iso_date = now.strftime('%Y-%m-%d')
        display_date = now.strftime('%B %d, %Y')

    # Content
    content_div = soup.select_one('#article_content')
    if not content_div:
        content_div = soup.find('div', class_='entry-content')
    if not content_div:
        content_div = soup.find('div', class_='article-body')
    if not content_div:
        content_div = soup.find('article')

    images = []
    captions = []
    
    # Extract images from carousel
    # Terrace Standard uses .slide which contains img and optionally .caption
    slides = soup.select('.slides-container .slide')
    for slide in slides:
        img = slide.find('img')
        if img:
            src = img.get('src') or img.get('data-src')
            if src:
                if '?' in src:
                    src = src.split('?')[0]
                
                # Check for duplicate images
                if src not in images:
                    images.append(src)
                    
                    # Try to find caption in this slide
                    caption_text = ""
                    caption_div = slide.find(class_='caption')
                    if caption_div:
                        caption_text = caption_div.get_text(strip=True)
                    else:
                        # Fallback: check data-image-caption attr on img
                        caption_text = img.get('data-image-caption', '').strip()
                    
                    captions.append(caption_text)

    content_html = ""
    excerpt = ""

    if content_div:
        # Cleanup
        for tag in content_div(['script', 'style', 'iframe', 'noscript', 'button', 'input', 'form']):
            tag.decompose()
        
        junk_selectors = [
            '.ad', '.mobile-ad-contain', '.marfeel-inline-recommender-first', 
            '.marfeel-inline-recommender-second', '.instory_widget',
            '.share_buttons_group', '.related-posts', '.author-bio',
            '#cedato-unit', '.marfeel-recommender-container', '.mrf-recommender-container',
            'div[data-mrf-recirculation]'
        ]
        for selector in junk_selectors:
            for tag in content_div.select(selector):
                tag.decompose()

        # Extract images from content
        for img in content_div.find_all('img'):
            src = img.get('src') or img.get('data-src')
            if src:
                if '?' in src:
                     src = src.split('?')[0]
                if src not in images:
                    images.append(src)

        content_html = "".join(str(c) for c in content_div.contents).strip()
        
        text = content_div.get_text(separator=' ', strip=True)
        excerpt = text[:300] + "..." if len(text) > 300 else text

    # Fallback image
    if not images:
        og_image = soup.find('meta', property='og:image')
        if og_image:
            src = og_image.get('content')
            if src:
                if '?' in src:
                    src = src.split('?')[0]
                images.append(src)

    slug = url.strip('/').split('/')[-1]

    return {
        "slug": slug,
        "title": title,
        "iso_date": iso_date,
        "display_date": display_date,
        "source_url": url,
        "content_html": content_html,
        "images": images,
        "captions": captions,
        "excerpt": excerpt
    }

def main():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE, 'r') as f:
            try:
                data = json.load(f)
            except:
                data = []
    else:
        data = []
        
    existing_slugs = get_existing_slugs(data)
    
    all_article_urls = set()
    
    for base_url in BASE_URLS:
        html = fetch_page(base_url)
        if html:
            soup = BeautifulSoup(html, 'html.parser')
            urls = extract_article_urls(soup)
            print(f"Found {len(urls)} articles on {base_url}")
            all_article_urls.update(urls)
            
    print(f"Total unique articles found: {len(all_article_urls)}")
    
    new_count = 0
    for url in all_article_urls:
         slug = url.strip('/').split('/')[-1]
         if slug in existing_slugs:
             print(f"Skipping existing: {slug}")
             continue
             
         print(f"Scraping {slug}...")
         article_data = parse_article(url)
         if article_data:
             data.append(article_data)
             new_count += 1
             time.sleep(1)
    
    # Sort data by iso_date descending
    data.sort(key=lambda x: x['iso_date'], reverse=True)
    
    with open(DATA_FILE, 'w') as f:
        json.dump(data, f, indent=2)
        
    print(f"Saved {new_count} new articles to {DATA_FILE}")

if __name__ == "__main__":
    main()
