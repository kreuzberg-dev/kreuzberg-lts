```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.PdfConfig;
import io.xberg.HierarchyConfig;
import java.util.Arrays;

ExtractionConfig config = ExtractionConfig.builder()
    .pdfOptions(PdfConfig.builder()
        .extractImages(true)
        .extractMetadata(true)
        .passwords(Arrays.asList("password1", "password2"))
        .hierarchyConfig(HierarchyConfig.builder().build())
        .build())
    .build();
```
