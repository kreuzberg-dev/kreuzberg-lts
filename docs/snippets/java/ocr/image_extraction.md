```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.ImageExtractionConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .imageExtraction(ImageExtractionConfig.builder()
        .extractImages(true)
        .targetDpi(200)
        .maxImageDimension(2048)
        .injectPlaceholders(true) // set to false to extract images without markdown references
        .autoAdjustDpi(true)
        .build())
    .build();
```
