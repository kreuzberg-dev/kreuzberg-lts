```go title="Go"
package main

import (
	"fmt"
	"log"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	result, err := kreuzberg.ExtractFileSync("document.pdf", nil)
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}

	fmt.Println(result.Content)
	fmt.Printf("Tables: %d\n", len(result.Tables))
	fmt.Printf("Metadata: %+v\n", result.Metadata)
}
```
