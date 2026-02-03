
import json
import os
from datetime import datetime

DATA_FILE = "frontend/src/data/journalism.json"

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

def main():
    existing = get_existing_data()
    slugs = {item['slug'] for item in existing}
    
    new_articles = [
        # From previous manual import
        {
            "slug": "team-searching-for-missing-thomas-kelly-raises-funds",
            "title": "Team searching for missing Thomas Kelly raises funds for final Skeena River search",
            "iso_date": "2025-10-16",
            "display_date": "Oct. 16, 2025",
            "source_url": "file://imports/Terrace Standard Terrace Standard 10_16_2025 1.pdf",
            "content_html": "<p>Family and community volunteers searching for Thomas Kelly have launched a fundraising effort to bring a professional K9 search team back to the Skeena River later this month as part of a final push before winter conditions set in.</p><p>Kelly, a husband and father, disappeared while fishing with family on July 12. Although official search efforts ended within days, volunteers have continued to comb the Skeena River for months, supported by local operators, drone crews and the Canadian Search Dogs and Disaster Dogs Association (CASDDA).</p><p>Organizer Jahaziel Trudell said in a Facebook post on Oct. 3 that the team now hopes to raise enough money to fund one last large-scale search, targeting the lower Skeena estuary between Lower Skeena River Provincial Park and the Haysport–Port Essington area — a section she described as 'tidal, complex and high priority.'</p><p>'Fundraising will help offset costs for the CASDDA K9 team and the volunteer fishing charters and guides assisting us,' Trudell explained in the post. 'At this time of year, with fluctuating water levels and challenging weather, it is critical to have professional and highly experienced boat operators to ensure safety and best support the K9 teams.'</p><p>Tentative dates for the three-day K9 operation are Oct. 17 to 19, Oct. 24 to 26, or Oct. 31 to Nov. 2, depending on tides and weather.</p>",
            "images": [],
            "captions": [],
            "byline": "By Jake Wray",
            "excerpt": "Family and community volunteers searching for Thomas Kelly have launched a fundraising effort to bring a professional K9 search team back to the Skeena River later this month as part of a final push before winter conditions set in."
        },
        {
            "slug": "iskut-loses-patience-with-drug-dealers",
            "title": "Iskut loses patience with drug dealers",
            "iso_date": "2025-10-16",
            "display_date": "Oct. 16, 2025",
            "source_url": "file://imports/Terrace Standard Terrace Standard 10_16_2025 1.pdf",
            "content_html": "<p>Dealers warned to cease and desist.</p><p>(Content reconstructed from page scan)</p>", 
            "images": [],
            "captions": [],
            "byline": "By Jake Wray",
            "excerpt": "Iskut First Nation leadership has issued a warning to local drug dealers to cease operations immediately or face consequences."
        },
        {
             "slug": "police-identify-suspect-lakelse-lake",
             "title": "Police identify suspect after releasing photo",
             "iso_date": "2025-10-16",
             "display_date": "Oct. 16, 2025",
             "source_url": "file://imports/Terrace Standard Terrace Standard 10_16_2025 1.pdf",
             "content_html": "<p>Terrace RCMP have identified a suspect following a report of suspicious behaviour at Lakelse Lake.</p>",
             "images": [],
             "captions": [],
             "byline": "By Jake Wray",
             "excerpt": "Terrace RCMP have identified a suspect following a report of suspicious behaviour at Lakelse Lake."
        },
        # NEW ARTICLES FROM FILTERED SCAN
        {
            "slug": "braving-cold-to-build-community",
            "title": "Braving the cold to build for the community",
            "iso_date": "2026-01-22", # Corrected date from file name: Terrace Standard 01_22_2026
            "display_date": "Jan. 22, 2026",
            "source_url": "file://imports/Terrace Standard Terrace Standard 01_22_2026 1.pdf", # Page 3
            "content_html": "<p>Construction workers braved the cold and wind Jan. 23 as they worked on a new apartment building, which stands at the corner of Kenney Avenue and Agar Street.</p><p>It is a 39-unit, four-storey building with 16 of the units intended for larger families and other units are designed for seniors or people with mobility challenges.</p><p>The building will be ready for occupancy in fall 2026, according to a Facebook post from Acadia Northwest Mechanical, one of the primary contractors on the project.</p>",
            "images": [],
            "captions": ["Construction workers braved the cold and wind Jan. 23 as they worked on a new apartment building... (Jake Wray/Terrace Standard)"],
            "byline": "Jake Wray/Terrace Standard (Photo)",
            "excerpt": "Construction workers braved the cold and wind Jan. 23 as they worked on a new apartment building, which stands at the corner of Kenney Avenue and Agar Street."
        },
        {
             "slug": "bc-conservative-leader-visits-terrace",
             "title": "B.C. Conservative leader visits Terrace",
             "iso_date": "2025-10-23", 
             "display_date": "Oct. 23, 2025",
             "source_url": "file://imports/Terrace Standard Terrace Standard 10_23_2025 1.pdf", # Page 3
             "content_html": "<p>Bailey said continuing to visit northern communities will remain a priority as major projects reshape the province's economic and environmental landscape.</p><p>'Doing this work in conjunction with building the economy, ensuring partnership with nations, and doing it in the least environmentally harmful way possible is the way forward,' she said.</p>",
              "images": [],
              "captions": [],
              "byline": "With files from Jake Wray", 
              "excerpt": "B.C. conservative leader visits Terrace and tours LNG Canada facility, emphasizing partnership with nations and environmental responsibility."
        },
        # Newly Discovered Matches (May 2025)
        {
            "slug": "man-arrested-after-firearms-incident",
            "title": "Man arrested after firearms incident",
            "iso_date": "2025-03-19",
            "display_date": "Mar. 19, 2025",
            "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 03_19_2025 5.pdf", # Page 5
            "content_html": "<p>(Content pending full extraction)</p>",
            "images": [],
            "captions": [],
            "byline": "Jake Wray", 
            "excerpt": "Local police respond to firearms incident leading to arrest."
        },
        {
            "slug": "treat-mom-tune-up",
            "title": "Treat Mom to a Tune-Up?",
            "iso_date": "2025-05-07",
            "display_date": "May 07, 2025",
             "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_07_2025 1.pdf", # Page 10
            "content_html": "<p>(Content pending full extraction)</p>",
            "images": [],
            "captions": [],
            "byline": "Jake Wray",
            "excerpt": "Automotive feature for Mother's Day."
        },
        {
            "slug": "lakes-district-museum-launches", 
            "title": "Lakes District Museum launches new season",
            "iso_date": "2025-05-07",
            "display_date": "May 07, 2025",
            "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_07_2025 1.pdf", # Page 13
            "content_html": "<p>(Content pending full extraction)</p>",
             "images": [],
            "captions": [],
            "byline": "Jake Wray",
            "excerpt": "Museum opening season coverage."
        },
        {
            "slug": "bags-labelled-crystal-and-opioids",
            "title": "Bags labelled 'crystal' and opioids found",
            "iso_date": "2025-05-14",
            "display_date": "May 14, 2025",
            "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_14_2025 1.pdf", # Page 2
            "content_html": "<p>“These bags do not include pre- The harm-reduction supplies may be confusing to some residents,” said Jake Wray.</p>",
             "images": [],
            "captions": [],
            "byline": "Jake Wray",
            "excerpt": "Discovery of labelled drug bags raises community concerns regarding harm reduction supplies."
        },
        {
             "slug": "cut-and-wrap-facility",
             "title": "Cut & Wrap Facility Opens",
             "iso_date": "2025-05-14",
             "display_date": "May 14, 2025",
              "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_14_2025 1.pdf", # Page 3
             "content_html": "<p>(Content pending full extraction)</p>",
              "images": [],
             "captions": [],
             "byline": "Jake Wray",
             "excerpt": "New meat processing facility coverage."
        },
         {
             "slug": "energy-corridors-partnership",
             "title": "Energy corridors working in partnership",
             "iso_date": "2025-05-14",
             "display_date": "May 14, 2025",
             "source_url": "file://imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_14_2025 1.pdf", # Page 4
             "content_html": "<p>(Content pending full extraction)</p>",
              "images": [],
             "captions": [],
             "byline": "Jake Wray",
             "excerpt": "Editorial analysis on energy corridor partnerships."
        }
    ]
    
    count = 0
    for article in new_articles:
        if article['slug'] not in slugs:
            existing.append(article)
            count += 1
            
    save_data(existing)
    print(f"Added {count} new manually recovered articles.")

if __name__ == "__main__":
    main()
