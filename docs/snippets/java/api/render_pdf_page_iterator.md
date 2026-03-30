```java title="Java"
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

// Iterate all pages (memory-efficient, one page at a time)
try (var iter = Kreuzberg.PdfPageIterator.open(Path.of("document.pdf"), 150)) {
    while (iter.hasNext()) {
        Kreuzberg.PageResult page = iter.next();
        System.out.printf("Page %d: %d bytes%n", page.pageIndex(), page.data().length);
        Files.write(Path.of("page_" + page.pageIndex() + ".png"), page.data());
    }
}
```
