```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.KeywordConfig;
import io.xberg.KeywordAlgorithm;

ExtractionConfig config = ExtractionConfig.builder()
    .keywords(KeywordConfig.builder()
        .algorithm(KeywordAlgorithm.YAKE)
        .maxKeywords(10)
        .minScore(0.3)
        .ngramRange(1, 3)
        .language("en")
        .build())
    .build();
```
