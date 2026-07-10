```go title="Go"
package main

import (
	"fmt"

	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

func main() {
	config := &kreuzberg.ExtractionConfig{
		EnableQualityProcessing: true,  // Default
	}

	fmt.Printf("Quality processing enabled: %v\n", config.EnableQualityProcessing)
}
```
