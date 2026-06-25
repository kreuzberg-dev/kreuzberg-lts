```java title="Java"
import io.xberg.Xberg;
import io.xberg.ExtractionResult;
import io.xberg.ExtractionConfig;
import java.nio.file.Paths;

ExtractionConfig config = ExtractionConfig.builder().build();
ExtractionResult result = Xberg.extractFile(Paths.get("document.pdf"), config);

System.out.println(result.content());
System.out.println(result.mimeType());
```
