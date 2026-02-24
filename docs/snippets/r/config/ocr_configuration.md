```r
library(kreuzberg)

# Configure OCR with Tesseract
config <- extraction_config(
  force_ocr = TRUE,
  ocr = ocr_config(
    backend = "tesseract",
    language = "eng+deu",
    dpi = 300L
  )
)

result <- extract_file_sync("scanned_document.pdf", config = config)
cat(result$content)
```
