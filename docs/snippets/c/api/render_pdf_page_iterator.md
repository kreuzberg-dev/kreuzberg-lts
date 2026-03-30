```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    /* Iterate all pages (memory-efficient, one page at a time) */
    CRenderPageResult *page;
    for (size_t i = 0; ; i++) {
        page = kreuzberg_render_pdf_page("document.pdf", i, 150);
        if (!page) {
            break; /* No more pages or error */
        }

        printf("Page %zu: %zu bytes\n", i, page->data_len);

        char filename[64];
        snprintf(filename, sizeof(filename), "page_%zu.png", i);
        FILE *f = fopen(filename, "wb");
        fwrite(page->data, 1, page->data_len, f);
        fclose(f);

        kreuzberg_free_render_page_result(page);
    }

    return 0;
}
```
