```r title="R"
library(kreuzberg)

result <- extract_file_sync("document.pdf")

cat("Detected Language:", result$detected_language, "\n")
cat("Quality Score:", result$quality_score, "\n")
cat("Keywords:", paste(result$keywords, collapse=", "), "\n\n")

cat("Metadata fields:\n")
created_by <- metadata_field(result, "created_by")
if (!is.null(created_by)) {
  cat("Author:", created_by, "\n")
}

created <- metadata_field(result, "created_date")
if (!is.null(created)) {
  cat("Created Date:", created, "\n")
}

pages_meta <- metadata_field(result, "page_count")
if (!is.null(pages_meta)) {
  cat("Pages:", pages_meta, "\n")
}
```
