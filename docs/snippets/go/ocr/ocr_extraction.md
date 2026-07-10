```go title="Go"
package main

import (
	"log"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	lang := "eng"
	cfg := &kreuzberg.ExtractionConfig{
		OCR: &kreuzberg.OCRConfig{
			Backend:  "tesseract",
			Language: &lang,
		},
	}

	result, err := kreuzberg.ExtractFileSync("scanned.pdf", cfg)
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}
	log.Println(len(result.Content))
}
```
