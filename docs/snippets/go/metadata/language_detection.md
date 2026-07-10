```go title="Go"
package main

import (
	"log"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	minConfidence := 0.9
	detectMultiple := true
	result, err := kreuzberg.ExtractFileSync("document.pdf", &kreuzberg.ExtractionConfig{
		LanguageDetection: &kreuzberg.LanguageDetectionConfig{
			Enabled:        kreuzberg.BoolPtr(true),
			MinConfidence:  &minConfidence,
			DetectMultiple: &detectMultiple,
		},
	})
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}

	log.Println("content length:", len(result.Content))
}
```
