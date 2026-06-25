```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.TokenReductionConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .tokenReduction(TokenReductionConfig.builder()
        .mode("moderate")
        .preserveMarkdown(true)
        .preserveCode(true)
        .languageHint("eng")
        .build())
    .build();
```
