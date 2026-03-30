```python title="Python"
from kreuzberg import render_pdf_page

# Iterate all pages by index (memory-efficient, one page at a time)
from kreuzberg import render_pdf_page

for page_index in range(total_pages):
    png_bytes = render_pdf_page("document.pdf", page_index=page_index, dpi=150)
    print(f"Page {page_index}: {len(png_bytes)} bytes")
```
