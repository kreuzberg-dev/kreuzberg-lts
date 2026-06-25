```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.OcrConfig;
import io.xberg.TesseractConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .ocr(OcrConfig.builder()
        .backend("tesseract")
        .language("eng+fra")
        .tesseractConfig(TesseractConfig.builder()
            .psm(3)
            .build())
        .build())
    .build();
```
