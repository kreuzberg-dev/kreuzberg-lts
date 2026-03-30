```typescript title="TypeScript"
import { renderPdfPageSync } from "@kreuzberg/node";
import { writeFileSync } from "node:fs";

// Render a single page (zero-based index)
const pngBytes = renderPdfPageSync("document.pdf", 0, 150);

writeFileSync("first_page.png", pngBytes);
```
