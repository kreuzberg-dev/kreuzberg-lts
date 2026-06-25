```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.ChunkingConfig;
import io.xberg.EmbeddingConfig;
import io.xberg.EmbeddingModelType;

ExtractionConfig config = ExtractionConfig.builder()
    .chunking(ChunkingConfig.builder()
        .maxChars(1000)
        .maxOverlap(200)
        .embedding(EmbeddingConfig.builder()
            .model(EmbeddingModelType.preset("all-minilm-l6-v2"))
            .normalize(true)
            .batchSize(32)
            .build())
        .build())
    .build();
```
