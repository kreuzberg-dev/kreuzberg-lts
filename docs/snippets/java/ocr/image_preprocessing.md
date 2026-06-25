```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.ImagePreprocessingConfig;
import io.xberg.OcrConfig;
import io.xberg.TesseractConfig;

ExtractionConfig config = ExtractionConfig.builder()
    .ocr(OcrConfig.builder()
        .tesseractConfig(TesseractConfig.builder()
            .preprocessing(ImagePreprocessingConfig.builder()
                .targetDpi(300)
                .denoise(true)
                .deskew(true)
                .contrastEnhance(true)
                .binarizationMethod("otsu")
                .build())
            .build())
        .build())
    .build();
```
