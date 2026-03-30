```python title="Python"
from kreuzberg import render_pdf_page

# Render a single page (zero-based index)
png_bytes = render_pdf_page("document.pdf", page_index=0, dpi=150)

# Write to disk
with open("first_page.png", "wb") as f:
    f.write(png_bytes)
```
