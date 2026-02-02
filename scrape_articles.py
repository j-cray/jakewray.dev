import requests
from bs4 import BeautifulSoup
import json
import os
import time
from datetime import datetime
import re

# Configuration
TARGET_URLS = [
    "https://terracestandard.com/2025/11/18/council-supports-proposed-nisgaa-health-clinic-in-terrace/",
    "https://terracestandard.com/2025/11/13/ksi-lisims-bc-hydros-north-coast-transmission-line-to-be-fast-tracked-pm/",
    "https://terracestandard.com/2025/06/10/prince-rupert-has-2nd-wettest-may-on-record-more-than-double-its-average/",
    "https://terracestandard.com/2025/06/09/may-in-terrace-was-wetter-colder-than-usual/",
    "https://terracestandard.com/2024/03/18/terrace-river-kings-bring-home-cameron-kerr-cup-in-memory-of-teammate/",
    "https://terracestandard.com/2021/09/30/council-finalizing-new-rules-restricting-fireworks-in-terrace/",
    "https://terracestandard.com/2021/09/18/city-supports-bid-to-use-geothermal-energy/",
    "https://terracestandard.com/2021/06/22/former-terrace-man-on-parole-granted-authorization-to-travel-outside-of-canada/",
    "https://terracestandard.com/2020/12/18/covid-19-exposures-reported-at-thornhill-primary-school-mountain-view-christian-academy/",
    "https://terracestandard.com/2020/12/09/covid-19-exposure-reported-at-caledonia-secondary-school-in-terrace/",
    "https://terracestandard.com/2020/12/07/covid-19-exposure-reported-at-suwilaawks-community-school-in-terrace/",
    "https://terracestandard.com/2020/11/26/students-at-nisgaa-school-test-positive-for-covid-19/",
    "https://terracestandard.com/2020/11/18/foster-family-loses-old-remo-home-to-fire/",
    "https://terracestandard.com/2020/10/24/skeena-candidates-talk-inland-port-resource-development-at-second-all-candidates-forum/",
    "https://terracestandard.com/2020/10/09/2020-virtual-all-candidates-forum-to-take-place-wednesday-oct-14/",
    "https://terracestandard.com/2020/07/22/highway-of-tears-memorial-totem-pole-to-be-raised-on-kitsumkalum-territory-west-of-terrace/",
    "https://terracestandard.com/2020/04/21/help-the-terrace-standard-continue-its-mission-to-provide-trusted-local-news/",
    "https://terracestandard.com/2020/10/13/skeena-voices-living-the-simple-life/",
    "https://terracestandard.com/2020/10/06/inland-port-to-be-discussed-at-oct-9-council-meeting/",
    "https://terracestandard.com/2020/09/24/gas-station-on-kalum-st-in-terrace-might-become-craft-liqour-distillery/",
    "https://terracestandard.com/2020/09/23/heres-the-latest-on-the-proposed-inland-port-development-process/",
    "https://terracestandard.com/2020/09/17/save-our-children-demonstration-held-in-terrace/",
    "https://terracestandard.com/2020/09/16/film-to-premier-at-tillicum-twin-theatres/",
    "https://terracestandard.com/2020/09/16/mla-to-ride-bike-from-terrace-to-kitimat/",
    "https://terracestandard.com/2020/09/16/skeena-voices-witchcraft-nothing-like-in-hollywood-movies/",
    "https://terracestandard.com/2020/09/16/nisgaa-nation-lifts-local-state-of-emergency/",
    "https://terracestandard.com/2020/09/09/video-fire-smolders-in-downtown-terrace/",
    "https://terracestandard.com/2020/09/09/fire-smolders-in-downtown-terrace-storefront/",
    "https://terracestandard.com/2020/09/09/agriculture-expert-talks-northwest-food-security/",
    "https://terracestandard.com/2020/09/08/mp-taylor-bachrach-shares-labour-day-thoughts/",
    "https://terracestandard.com/2020/09/03/three-covid-19-cases-confirmed-in-the-nass-valley/",
    "https://terracestandard.com/2020/09/03/highway-of-tears-memorial-totem-pole-to-be-raised-tomorrow/",
    "https://terracestandard.com/2020/09/02/overdose-calls-increasing-significantly-in-terrace/",
    "https://terracestandard.com/2020/08/31/nisgaa-nation-enacts-emergency-measures-after-possible-covid-19-exposure/",
    "https://terracestandard.com/2020/08/27/tow-truck-impounded-after-attempting-to-impound-street-racers-2/",
    "https://terracestandard.com/2020/08/27/tow-truck-impounded-after-attempting-to-impound-street-racers/",
    "https://terracestandard.com/2020/08/21/terrace-city-council-chambers-to-get-new-audio-video-equipment/",
    "https://terracestandard.com/2020/08/20/council-briefs-court-watch-loses-momentum/",
    "https://terracestandard.com/2020/08/19/terrace-rcmp-searching-for-suspect-in-armed-robbery/",
    "https://terracestandard.com/2020/08/19/former-terrace-man-gets-full-parole-on-drug-weapon-convictions/",
    "https://terracestandard.com/2020/08/17/vehicle-incident-knocks-out-power-in-terrace/",
    "https://terracestandard.com/2020/08/11/terrace-fire-department-gets-new-rescue-truck/",
    "https://terracestandard.com/2020/08/10/survivors-of-abuse-at-indian-day-schools-could-use-help-accessing-compensation-advocate-says/",
    "https://terracestandard.com/2020/08/05/terrace-property-tax-payments-steadily-coming-in/",
    "https://terracestandard.com/2020/07/23/terrace-rcmp-taser-pipe-wielding-woman/",
    "https://terracestandard.com/2020/07/21/drugs-cash-weapons-seized-after-traffic-stop-in-thornhill/",
    "https://terracestandard.com/2020/07/15/skeena-voices-resting-reflecting-writing-after-long-career/",
    "https://terracestandard.com/2020/07/15/skeena-voices-you-help-where-you-can/",
    "https://terracestandard.com/2020/07/10/four-air-ambulance-flights-out-of-terrace-delayed-or-cancelled/",
    "https://terracestandard.com/2020/07/10/12-year-old-terrace-boy-missing/"
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
        
        # Check for redirect to home page (soft 404)
        if response.url.rstrip('/') == 'https://terracestandard.com':
            print(f"  -> Redirected to home page. Skipping.")
            return None
            
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
    now = datetime.now()
    
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
                    caption_div = slide.find(class_='slide-caption')
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

    # Validation: Skip if filtered content is empty or contains "files from Jake Wray" attribution
    if not content_html or not title or title == "Untitled":
        print(f"  -> Failed validation: Empty content or no title.")
        return None
        
    # Check if article is only "with files from" Jake Wray
    # Case insensitive check
    lower_content = content_html.lower()
    if 'files from jake wray' in lower_content:
        print(f"  -> Failed validation: Article is only contributed to by Jake Wray (not primary author).")
        return None

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
    
    new_count = 0
    for url in TARGET_URLS:
         slug = url.strip('/').split('/')[-1]
         if slug in existing_slugs:
             print(f"Skipping existing: {slug}")
             continue
             
         print(f"Scraping {slug}...")
         article_data = parse_article(url)
         if article_data:
             data.append(article_data)
             new_count += 1
         else:
             print(f"Failed to scrape {url}")
         
         time.sleep(1)
    
    # Sort data by iso_date descending
    data.sort(key=lambda x: x['iso_date'], reverse=True)
    
    with open(DATA_FILE, 'w') as f:
        json.dump(data, f, indent=2)
        
    print(f"Saved {new_count} new articles to {DATA_FILE}")

if __name__ == "__main__":
    main()
