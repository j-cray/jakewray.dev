#!/usr/bin/env python3
"""
Scrape journalism articles from the Jake Wray WordPress site, download images,
upload them to GCS, and insert them directly into the SQLite database.

Usage:
  python scripts/import_journalism.py

Prereqs:
  - `requests` and `beautifulsoup4` Python packages (install with pip if needed)
  - `gsutil` installed and authenticated
  - Active gcloud project set to `jakewray-portfolio`
"""

import json
import os
import re
import subprocess
import sys
import sqlite3
import uuid
from datetime import datetime
from pathlib import Path
from urllib.parse import urlparse

try:
    import requests
    from bs4 import BeautifulSoup
except ImportError as exc:
    sys.stderr.write("Missing dependency: {}\n".format(exc))
    sys.stderr.write("Install with: python -m pip install --user requests beautifulsoup4\n")
    raise

BASE_URLS = [
    "https://jakewrayportfolio.wordpress.com/category/news-articles/",
    "https://jakewrayportfolio.wordpress.com/category/news-articles/page/2/",
]
BUCKET = "jakewray-portfolio"
PREFIX = "media/journalism"
DATA_PATH = Path("frontend/src/data/journalism.json")
TMP_DIR = Path(".tmp/journalism")
TMP_DIR.mkdir(parents=True, exist_ok=True)

HEADERS = {"User-Agent": "Mozilla/5.0 (compatible; ingest-script/1.0)"}

def fetch(url: str) -> str:
    resp = requests.get(url, headers=HEADERS, timeout=30)
    resp.raise_for_status()
    return resp.text


def slug_from_url(url: str) -> str:
    path = urlparse(url).path.rstrip("/")
    return path.split("/")[-1]


def parse_listing(url: str) -> list[str]:
    html = fetch(url)
    soup = BeautifulSoup(html, "html.parser")
    links = []
    for a in soup.select("h2.entry-title a"):
        href = a.get("href")
        if href and href not in links:
            links.append(href)
    return links


def parse_date(text: str) -> tuple[str, str]:
    """Return (iso_date, display_date)."""
    text = text.strip()
    for fmt in ["%B %d, %Y", "%b. %d, %Y", "%b %d, %Y"]:
        try:
            dt = datetime.strptime(text, fmt)
            return dt.strftime("%Y-%m-%d"), dt.strftime("%B %d, %Y")
        except ValueError:
            continue
    # Sometimes date appears as "Jan. 8, 2021" in body text.
    match = re.search(r"([A-Za-z]{3,}\.?)\s+\d{1,2},\s+\d{4}", text)
    if match:
        raw = match.group(0)
        for fmt in ["%b. %d, %Y", "%B %d, %Y", "%b %d, %Y"]:
            try:
                dt = datetime.strptime(raw, fmt)
                return dt.strftime("%Y-%m-%d"), dt.strftime("%B %d, %Y")
            except ValueError:
                continue
    raise ValueError(f"Could not parse date from '{text}'")


def clean_content(content):
    for tag in content.select("script, style, noscript"):
        tag.decompose()
    # Drop WP-specific share/footer bits if present
    for tag in content.select("div.sharedaddy, div.wp-block-buttons"):
        tag.decompose()
    return content


def download_image(url: str, dest: Path):
    resp = requests.get(url, headers=HEADERS, timeout=30)
    resp.raise_for_status()
    dest.write_bytes(resp.content)


def upload_to_gcs(local_path: Path, bucket: str, object_path: str) -> str:
    gcs_uri = f"gs://{bucket}/{object_path}"
    subprocess.run([
        "gsutil",
        "cp",
        "-a",
        "public-read",
        str(local_path),
        gcs_uri,
    ], check=True)
    return f"https://storage.googleapis.com/{bucket}/{object_path}"


def process_article(url: str) -> dict:
    html = fetch(url)
    soup = BeautifulSoup(html, "html.parser")

    title_tag = soup.select_one("h1.entry-title")
    if not title_tag:
        raise RuntimeError(f"No title found for {url}")
    title = title_tag.get_text(strip=True)

    date_tag = soup.select_one("time.entry-date") or soup.select_one("time")
    if not date_tag:
        raise RuntimeError(f"No date found for {url}")
    iso_date, display_date = parse_date(date_tag.get_text(strip=True))

    content = soup.select_one("div.entry-content")
    if not content:
        raise RuntimeError(f"No content found for {url}")
    content = clean_content(content)

    slug = slug_from_url(url)
    images = []
    for idx, img in enumerate(content.find_all("img"), start=1):
        src = img.get("src")
        if not src:
            continue
        parsed = urlparse(src)
        ext = os.path.splitext(parsed.path)[1] or ".jpg"
        filename = f"{slug}-{idx}{ext}"
        local_path = TMP_DIR / filename
        try:
            download_image(src, local_path)
        except Exception as exc:  # noqa: BLE001
            sys.stderr.write(f"Failed to download {src}: {exc}\n")
            continue
        object_path = f"{PREFIX}/{slug}/{filename}"
        public_url = upload_to_gcs(local_path, BUCKET, object_path)
        images.append(public_url)
        img["src"] = public_url
        if img.has_attr("srcset"):
            del img["srcset"]
        if img.has_attr("sizes"):
            del img["sizes"]

    content_html = str(content)
    excerpt = content.get_text(" ", strip=True)
    if len(excerpt) > 320:
        excerpt = excerpt[:317] + "..."

    return {
        "slug": slug,
        "title": title,
        "iso_date": iso_date,
        "display_date": display_date,
        "source_url": url,
        "content_html": content_html,
        "images": images,
        "excerpt": excerpt,
    }


def main():
    all_links = []
    for url in BASE_URLS:
        links = parse_listing(url)
        all_links.extend(links)
    # De-dup and keep order
    seen = set()
    deduped = []
    for link in all_links:
        if link not in seen:
            seen.add(link)
            deduped.append(link)

    articles = []
    for link in deduped:
        sys.stderr.write(f"Processing {link}\n")
        article = process_article(link)
        articles.append(article)

    articles.sort(key=lambda a: a["iso_date"], reverse=True)

    # Insert articles directly into the SQLite database
    db_path = "sqlite.db"
    print(f"Connecting to database to import {len(articles)} articles...")
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Ensure table exists
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS articles (
            id TEXT PRIMARY KEY,
            wp_id BIGINT UNIQUE,
            slug TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            subtitle TEXT,
            excerpt TEXT,
            content TEXT NOT NULL,
            cover_image_url TEXT,
            author TEXT NOT NULL,
            published_at DATETIME NOT NULL,
            origin TEXT NOT NULL DEFAULT 'local',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    """)

    for a in articles:
        uid = str(uuid.uuid4())
        published_at = f"{a['iso_date']}T00:00:00.000Z"
        cover_image = a["images"][0] if a["images"] else None
        byline = a.get("byline", "Jake Wray")

        cursor.execute("""
            INSERT INTO articles (id, slug, title, content, excerpt, cover_image_url, author, published_at, origin)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'imported')
            ON CONFLICT(slug) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                excerpt = excluded.excerpt,
                cover_image_url = excluded.cover_image_url,
                author = excluded.author,
                published_at = excluded.published_at,
                updated_at = CURRENT_TIMESTAMP
        """, (
            uid,
            a["slug"],
            a["title"],
            a["content_html"],
            a["excerpt"],
            cover_image,
            byline,
            published_at
        ))

    conn.commit()
    conn.close()
    print("Database sync complete!")

    # Write an empty list to journalism.json to prevent compile failures but remove binary bloat
    DATA_PATH.parent.mkdir(parents=True, exist_ok=True)
    DATA_PATH.write_text("[]", encoding="utf-8")
    print(f"Cleared local static asset file: {DATA_PATH}")


if __name__ == "__main__":
    main()
