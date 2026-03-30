```rust title="Rust"
use kreuzberg::pdf::render_pdf_page_to_png;

let pdf_bytes = std::fs::read("document.pdf")?;

// Render a single page (zero-based index)
let png = render_pdf_page_to_png(&pdf_bytes, 0, Some(150), None)?;

std::fs::write("first_page.png", &png)?;
```
