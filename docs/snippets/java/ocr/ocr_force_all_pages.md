```java title="Java"
import io.xberg.Xberg;
import io.xberg.ExtractionResult;
import io.xberg.ExtractionConfig;
import io.xberg.OcrConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .ocr(OcrConfig.builder()
        .backend("tesseract")
        .build())
    .forceOcr(true)
    .build();

ExtractionResult result = Xberg.extractFile("document.pdf", config);
System.out.println(result.getContent());
```
