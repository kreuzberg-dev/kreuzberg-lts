```java title="Java"
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

try (var iter = Kreuzberg.PdfPageIterator.open(Path.of("document.pdf"), 150)) {
    // Render a single page (first page)
    if (iter.hasNext()) {
        Kreuzberg.PageResult page = iter.next();
        Files.write(Path.of("first_page.png"), page.data());
    }
}
```
