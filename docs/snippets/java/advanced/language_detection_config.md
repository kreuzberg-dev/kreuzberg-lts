```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.LanguageDetectionConfig;
import java.math.BigDecimal;

ExtractionConfig config = ExtractionConfig.builder()
    .languageDetection(LanguageDetectionConfig.builder()
        .enabled(true)
        .minConfidence(new BigDecimal("0.8"))
        .detectMultiple(false)
        .build())
    .build();
```
