
import pdfplumber
import sys

PDF_PATH = "imports/Burns Lake Lakes District News Burns Lake Lakes District News 05_21_2025 1.pdf"

def inspect_layout(pdf_path, page_num=1):
    with pdfplumber.open(pdf_path) as pdf:
        if page_num >= len(pdf.pages):
            print(f"Page {page_num} not found")
            return
            
        page = pdf.pages[page_num]
        
        print(f"--- Page {page_num+1} Layout Analysis ---")
        print(f"Size: {page.width}x{page.height}")
        
        # Group words into lines for context analysis
        words = page.extract_words()
        lines = []
        current_line = []
        last_top = 0
        
        for word in words:
            if abs(word['top'] - last_top) > 5: # New line detection
                if current_line:
                    lines.append(current_line)
                current_line = []
                last_top = word['top']
            current_line.append(word)
        if current_line: lines.append(current_line)

        # Search lines for "Jake Wray"
        print("\n--- Jake Wray Context Analysis ---")
        for i, line in enumerate(lines):
            line_text = " ".join([w['text'] for w in line])
            if "Jake" in line_text and "Wray" in line_text:
                print(f"Match found on line {i}:")
                # Print surrounding lines
                start = max(0, i-2)
                end = min(len(lines), i+3)
                for j in range(start, end):
                    print(f"  Line {j}: {' '.join([w['text'] for w in lines[j]])}")
                print("-" * 30)

        # Dump all words with coordinates to visualize structure
        # (limiting output for readability)
        print("\n--- First 50 Words ---")
        for i, word in enumerate(words[:50]):
             print(f"Word: {word['text']:<20} | Top: {word['top']:.2f} | Size: {word['bottom']-word['top']:.2f} | x0: {word['x0']:.2f}")

        # Check for images
        print(f"\n--- Images: {len(page.images)} ---")
        for img in page.images:
            print(f"Image: x0={img['x0']}, top={img['top']}, width={img['width']}, height={img['height']}")

if __name__ == "__main__":
    inspect_layout(PDF_PATH)
