import pdfplumber
import sys

def inspect(path):
    with pdfplumber.open(path) as pdf:
        page = pdf.pages[0]
        words = page.extract_words(keep_blank_chars=True, extra_attrs=['fontname', 'size'])
        
        
        print(f"Total words found: {len(words)}")
        # Sort by size descending
        words.sort(key=lambda x: x['size'], reverse=True)
        

        
        print(f"--- All words in {path} ---")
        for w in words:
            print(f"Text: {w['text']}")


            
if __name__ == "__main__":
    if len(sys.argv) > 1:
        inspect(sys.argv[1])
    else:
        # Default to one of the problem files
        inspect("imports/Terrace Standard Terrace Standard 11_06_2025 1.pdf")
