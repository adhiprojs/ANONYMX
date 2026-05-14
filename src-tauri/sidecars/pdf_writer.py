import sys
import argparse
from fpdf import FPDF

def write_pdf(file_path, text):
    try:
        pdf = FPDF()
        pdf.add_page()
        pdf.set_font("Helvetica", size=12)
        
        # Replace non-latin-1 chars with '?' to avoid fpdf2 errors with standard fonts
        for line in text.split('\n'):
            clean_line = line.encode('latin-1', 'replace').decode('latin-1')
            pdf.multi_cell(0, 7, clean_line)
            
        pdf.output(file_path)
    except Exception as e:
        print(f"Error writing PDF: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--path", required=True)
    args = parser.parse_args()
    text = sys.stdin.read()
    write_pdf(args.path, text)
