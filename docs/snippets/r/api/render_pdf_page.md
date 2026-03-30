```r title="R"
library(kreuzberg)

# Render a single page (zero-based index)
png <- render_pdf_page("document.pdf", 0L, dpi = 150L)

writeBin(png, "first_page.png")
```
