```java title="Java"
import io.xberg.ChunkingConfig;
import io.xberg.EmbeddingConfig;
import io.xberg.EmbeddingModelType;
import io.xberg.ExtractionConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .chunking(ChunkingConfig.builder()
        .maxChars(1000)
        .embedding(EmbeddingConfig.builder()
            .model(EmbeddingModelType.builder()
                .type("preset")
                .name("all-mpnet-base-v2")
                .build())
            .batchSize(16)
            .normalize(true)
            .showDownloadProgress(true)
            .build())
        .build())
    .build();
```
