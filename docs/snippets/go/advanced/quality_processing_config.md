```go title="Go"
package main

import (
	"github.com/kreuzberg-dev/kreuzberg-lts/v4"
)

enableQualityProcessing := true

config := &kreuzberg.ExtractionConfig{
	EnableQualityProcessing: &enableQualityProcessing,
}
```
