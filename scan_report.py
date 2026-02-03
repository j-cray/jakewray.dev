
import pdfplumber
import sys
import os

def generate_report(pdf_paths):
    print("Generating Match Report...")
    match_count = 0
    
    for path in pdf_paths:
        try:
            with pdfplumber.open(path) as pdf:
                for i, page in enumerate(pdf.pages):
                    text = page.extract_text()
                    if not text: 
                        continue
                        
                    if "jake wray" in text.lower():
                        # Extract the specific line or context
                        lines = text.split('\n')
                        context = ""
                        for line in lines:
                            if "jake wray" in line.lower():
                                context = line.strip()
                                break
                        
                        print(f"MATCH: {os.path.basename(path)} | Page {i+1} | Context: {context}")
                        match_count += 1
                        
        except Exception as e:
            print(f"Error reading {path}: {e}")
            
    print(f"\nTotal Matches Found: {match_count}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python scan_report.py <pdf_files...>")
    else:
        files = sorted(sys.argv[1:])
        generate_report(files)
