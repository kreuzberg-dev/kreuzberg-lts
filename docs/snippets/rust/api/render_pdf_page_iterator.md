```rust title="Rust"
use kreuzberg::pdf::PdfPageIterator;

// Iterate all pages (memory-efficient, one page at a time)
for result in PdfPageIterator::from_file("document.pdf", Some(150), None)? {
    let (page_index, png_bytes) = result?;
    println!("Page {}: {} bytes", page_index, png_bytes.len());
}
```
