```go title="Go"
package main

import (
	"fmt"
	"log"
	"os"

	"github.com/kreuzberg-dev/kreuzberg/packages/go/v4"
)

func main() {
	// Render a single page (zero-based index)
	iter, err := kreuzberg.NewPdfPageIterator("document.pdf", 150)
	if err != nil {
		log.Fatalf("failed to open PDF: %v", err)
	}
	defer iter.Close()

	pageIndex, png, ok, err := iter.Next()
	if err != nil {
		log.Fatalf("render error: %v", err)
	}
	if ok {
		os.WriteFile(fmt.Sprintf("page_%d.png", pageIndex), png, 0644)
	}
}
```
