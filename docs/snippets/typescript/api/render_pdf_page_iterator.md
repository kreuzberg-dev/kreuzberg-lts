```typescript title="TypeScript"
import { PdfPageIterator } from "@kreuzberg/node";
import { writeFileSync } from "node:fs";

// Iterate all pages (memory-efficient, one page at a time)
const iter = new PdfPageIterator("document.pdf", 150);
let result;
while ((result = iter.next()) !== null) {
    const { pageIndex, data } = result;
    console.log(`Page ${pageIndex}: ${data.length} bytes`);
    writeFileSync(`page_${pageIndex}.png`, data);
}
iter.close();
```
