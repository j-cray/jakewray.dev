import requests
from bs4 import BeautifulSoup
import json
import os
import time
from datetime import datetime
import re

# Configuration
TARGET_URLS = [
    "https://terracestandard.com/2020/07/09/mother-grizzly-bear-with-two-cubs-spotted-on-gruchys-beach-trail-near-terrace/",
    "https://terracestandard.com/2020/07/09/conservation-officers-relocate-spirit-bear-known-to-roam-northwestern-b-c/",
    "https://terracestandard.com/2020/07/09/terrace-first-responders-honour-late-colleague-with-large-procession/",
    "https://terracestandard.com/2020/07/08/longtime-terrace-resident-celebrates-100th-birthday/",
    "https://terracestandard.com/2020/07/07/man-found-injured-in-thornhill/",
    "https://terracestandard.com/2020/07/07/northwest-indigenous-governments-form-new-alliance/",
    "https://terracestandard.com/2020/07/04/terrace-couple-awarded-by-governor-general-for-volunteer-work/",
    "https://terracestandard.com/2020/07/03/group-rescued-unharmed-after-attempting-to-tube-lakelse-river/",
    "https://terracestandard.com/2020/06/29/conservation-officers-kill-black-bear-near-terrace/",
    "https://terracestandard.com/2020/06/26/five-more-murals-to-be-painted-in-downtown-terrace-this-summer/",
    "https://terracestandard.com/2020/06/25/terrace-rcmp-searching-for-nass-valley-man-wanted-on-arrest-warrant/",
    "https://terracestandard.com/2020/06/24/skeena-voices-im-happy-schools-out-at-least/",
    "https://terracestandard.com/2020/06/23/two-black-bears-near-terrace-conservation-officers-say/",
    "https://terracestandard.com/2020/06/23/some-coastal-gaslink-workers-could-be-housed-in-terrace-hotels/",
    "https://terracestandard.com/2020/06/18/nearly-1000-free-meals-distributed-from-outreach-food-truck/",
    "https://terracestandard.com/2020/06/18/modified-grad-events-happening-this-weekend/",
    "https://terracestandard.com/2020/06/15/company-nearly-ready-to-tap-geothermal-energy-near-terrace/",
    "https://terracestandard.com/2020/06/11/developer-says-city-of-terrace-overcharging-for-building-permits/",
    "https://terracestandard.com/2020/06/11/king-of-the-mountain-foot-race-switches-to-virtual-format/",
    "https://terracestandard.com/2020/06/10/terrace-rcmp-arrest-man-near-recycling-depot/",
    "https://terracestandard.com/2020/06/09/anti-racism-demonstration-held-in-terrace/",
    "https://terracestandard.com/2020/06/08/heritage-park-museum-open-for-summer-season/",
    "https://terracestandard.com/2020/06/05/skeena-valley-farmers-market-looks-to-reopen/",
    "https://terracestandard.com/2020/06/04/city-of-terrace-considers-reopening-playgrounds-day-camps/",
    "https://terracestandard.com/2020/06/03/skeena-mla-advocates-for-small-lng-project-in-terrace-2/",
    "https://terracestandard.com/2020/06/03/skeena-mla-advocates-for-small-lng-project-in-terrace/",
    "https://terracestandard.com/2020/06/03/scientists-produce-extensive-map-of-tseax-volcano-lava-flow/",
    "https://terracestandard.com/2020/06/01/terrace-rcmp-respond-to-firearms-call-on-south-side/",
    "https://terracestandard.com/2020/06/01/city-increasing-maximum-business-license-fine/",
    "https://terracestandard.com/2020/05/29/terrace-womans-truck-vandalized-with-go-home-message/",
    "https://terracestandard.com/2020/05/29/city-council-considers-easing-food-truck-restriction/",
    "https://terracestandard.com/2020/05/26/local-runners-complete-terrace-to-rosswood-marathon/",
    "https://terracestandard.com/2020/05/26/firefighter-drops-eggs-65-ft-for-kids-science-project/",
    "https://terracestandard.com/2020/05/22/parklets-returning-to-downtown-terrace/",
    "https://terracestandard.com/2020/05/21/terrace-couples-dogs-battle-wolves-while-camping-2/",
    "https://terracestandard.com/2020/05/21/terrace-couples-dogs-battle-wolves-while-camping/",
    "https://terracestandard.com/2020/05/20/police-chase-suspect-denied-bail/",
    "https://terracestandard.com/2020/05/19/terrace-rcmp-investigating-homicide/",
    "https://terracestandard.com/2020/05/15/dominos-customer-allegedly-finds-spit-in-pizza/",
    "https://terracestandard.com/2020/05/15/small-wildfire-damages-classic-truck/",
    "https://terracestandard.com/2020/05/14/council-poses-questions-to-rcmp-inspector/",
    "https://terracestandard.com/2020/05/14/ferry-island-camground-to-reopen-for-locals/",
    "https://terracestandard.com/2020/05/14/no-one-hurt-after-report-of-shots-fired-in-thornhill/",
    "https://terracestandard.com/2020/05/13/happy-gang-centre-closed-but-surviving/",
    "https://terracestandard.com/2020/05/12/small-wildfire-handled-on-copper-mountain/",
    "https://terracestandard.com/2020/05/12/rcmp-ready-for-may-long-weekend-as-parks-reopen/",
    "https://terracestandard.com/2020/05/11/small-fire-extinguished-on-lazelle-ave/",
    "https://terracestandard.com/2020/05/08/effects-of-closed-rcmp-training-centre-felt-in-terrace/",
    "https://terracestandard.com/2020/05/05/terrace-youth-soccer-goes-online/",
    "https://terracestandard.com/2020/05/04/plenty-of-work-still-to-be-done-at-museum-this-summer/",
    "https://burnslakelakesdistrictnews.com/2025/11/13/ksi-lisims-lng-north-coast-transmission-line-to-be-fast-tracked-pm/",
    "https://burnslakelakesdistrictnews.com/2025/09/19/copperview-seniors-housing-project-in-granisle-remains-on-schedule-says-bc-housing/",
    "https://burnslakelakesdistrictnews.com/2025/06/10/prince-rupert-has-2nd-wettest-may-on-record-more-than-double-its-average/",
    "https://burnslakelakesdistrictnews.com/2024/03/18/terrace-river-kings-bring-home-cameron-kerr-cup-in-memory-of-teammate/",
    "https://burnslakelakesdistrictnews.com/2020/07/22/highway-of-tears-memorial-totem-pole-to-be-raised-on-kitsumkalum-territory-west-of-terrace/",
    "https://burnslakelakesdistrictnews.com/2020/09/03/three-covid-19-cases-confirmed-in-the-nass-valley/",
    "https://burnslakelakesdistrictnews.com/2020/09/03/highway-of-tears-memorial-totem-pole-to-be-raised-tomorrow/",
    "https://burnslakelakesdistrictnews.com/2020/08/27/tow-truck-impounded-after-attempting-to-impound-street-racers/",
    "https://burnslakelakesdistrictnews.com/2020/07/10/four-air-ambulance-flights-out-of-terrace-delayed-or-cancelled/",
    "https://burnslakelakesdistrictnews.com/2020/07/09/conservation-officers-relocate-spirit-bear-known-to-roam-northwestern-b-c/",
    "https://burnslakelakesdistrictnews.com/2020/05/21/terrace-couples-dogs-battle-wolves-while-camping/"
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
