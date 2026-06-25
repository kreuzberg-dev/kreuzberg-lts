```java title="Java"
import io.xberg.ExtractionConfig;
import io.xberg.PostProcessorConfig;
import java.util.Arrays;

ExtractionConfig config = ExtractionConfig.builder()
    .postprocessor(PostProcessorConfig.builder()
        .enabled(true)
        .enabledProcessors(Arrays.asList("deduplication", "whitespace_normalization"))
        .disabledProcessors(Arrays.asList("mojibake_fix"))
        .build())
    .build();
```
