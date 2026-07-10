```go title="Go"
package main

import (
	"log"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	preserve := true
	result, err := kreuzberg.ExtractFileSync("document.pdf", &kreuzberg.ExtractionConfig{
		TokenReduction: &kreuzberg.TokenReductionConfig{
			Mode:                  "moderate",
			PreserveImportantWords: &preserve,
		},
	})
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}

	log.Println("content length:", len(result.Content))
}
```
