```java title="Java"
import io.xberg.ChunkingConfig;
import io.xberg.EmbeddingConfig;
import io.xberg.EmbeddingModelType;
import io.xberg.ExtractionConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .chunking(ChunkingConfig.builder()
        .maxChars(1500)
        .maxOverlap(200)
        .embedding(EmbeddingConfig.builder()
            .model(EmbeddingModelType.builder()
                .type("preset")
                .name("text-embedding-all-minilm-l6-v2")
                .build())
            .build())
        .build())
    .build();
```
