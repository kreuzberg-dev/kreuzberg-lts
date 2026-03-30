```go title="Go"
package main

import (
	"fmt"
	"log"
	"os"

	"github.com/kreuzberg-dev/kreuzberg/packages/go/v4"
)

func main() {
	// Iterate all pages (memory-efficient, one page at a time)
	iter, err := kreuzberg.NewPdfPageIterator("document.pdf", 150)
	if err != nil {
		log.Fatalf("failed to create iterator: %v", err)
	}
	defer iter.Close()

	for {
		pageIndex, png, ok, err := iter.Next()
		if err != nil {
			log.Fatalf("render error: %v", err)
		}
		if !ok {
			break
		}
		fmt.Printf("Page %d: %d bytes\n", pageIndex, len(png))
		os.WriteFile(fmt.Sprintf("page_%d.png", pageIndex), png, 0644)
	}
}
```
