
import pdfplumber
import sys

def debug_pdf(path):
    print(f"--- Debugging {path} ---")
    with pdfplumber.open(path) as pdf:
        if not pdf.pages:
            print("No pages found.")
            return

        page = pdf.pages[0]
        text = page.extract_text()
        print("--- EXTRACTED TEXT ---")
        print(text)
        print("----------------------")
        
        # Check specific keywords
        if "Jake Wray" in text:
            print("FOUND: 'Jake Wray' (exact match)")
        elif "JAKE WRAY" in text:
            print("FOUND: 'JAKE WRAY' (exact match)")
        else:
             print("NOT FOUND: 'Jake Wray' or 'JAKE WRAY'")
             
        # Check for common malformations
        normalized = text.replace(" ", "").lower()
        if "jakewray" in normalized:
             print("FOUND: 'jakewray' (ignoring spaces)")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python debug_extraction.py <pdf_path>")
    else:
        debug_pdf(sys.argv[1])
