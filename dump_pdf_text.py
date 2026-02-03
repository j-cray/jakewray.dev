
import pdfplumber
import sys
import os

def dump_text(pdf_paths):
    for path in pdf_paths:
        try:
            with pdfplumber.open(path) as pdf:
                for i, page in enumerate(pdf.pages):
                    text = page.extract_text()
                    if not text: 
                        continue
                        
                    # Strict check: only pages with the byline
                    if "jake wray" in text.lower():
                        print(f"\n========== MATCH: {path} (Page {i+1}) ==========")
                        print(text)
                        print("==================================================\n")
        except Exception as e:
            print(f"Error reading {path}: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python dump_pdf_text.py <pdf_files...>")
    else:
        # Sort files to keep output chronological/orderly
        files = sorted(sys.argv[1:])
        dump_text(files)
