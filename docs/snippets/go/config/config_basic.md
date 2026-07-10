```go title="Go"
package main

import (
	"log"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	useCache := true
	enableQP := true

	result, err := kreuzberg.ExtractFileSync("document.pdf", &kreuzberg.ExtractionConfig{
		UseCache:                &useCache,
		EnableQualityProcessing: &enableQP,
	})
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}

	log.Println("content length:", len(result.Content))
}
```
