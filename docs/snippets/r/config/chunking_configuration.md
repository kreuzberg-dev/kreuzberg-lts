```r
library(kreuzberg)

# Configure text chunking for RAG pipelines
config <- extraction_config(
  chunking = chunking_config(
    max_characters = 1000L,
    overlap = 200L
  )
)

result <- extract_file_sync("large_document.pdf", config = config)
cat("Number of chunks:", length(result$chunks), "\n")
for (chunk in result$chunks) {
  cat("Chunk:", substr(chunk$content, 1, 50), "...\n")
}
```
