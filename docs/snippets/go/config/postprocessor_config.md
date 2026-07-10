```go title="Go"
package main

import "github.com/kreuzberg-dev/kreuzberg-lts/v4"

func main() {
	enabled := true
	cfg := &kreuzberg.ExtractionConfig{
		Postprocessor: &kreuzberg.PostProcessorConfig{
			Enabled:            &enabled,
			EnabledProcessors:  []string{"deduplication", "whitespace_normalization"},
			DisabledProcessors: []string{"mojibake_fix"},
		},
	}

	_ = cfg
}
```
