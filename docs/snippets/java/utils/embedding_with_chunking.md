```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.ChunkingConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .chunking(ChunkingConfig.builder()
        .maxChars(1024)
        .maxOverlap(100)
        .embedding("balanced")
        .build())
    .build();
```
