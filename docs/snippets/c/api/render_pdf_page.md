```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    /* Render a single page (zero-based index) */
    CRenderPageResult *page = kreuzberg_render_pdf_page("document.pdf", 0, 150);
    if (!page) {
        fprintf(stderr, "Error: %s\n", kreuzberg_last_error());
        return 1;
    }

    FILE *f = fopen("first_page.png", "wb");
    fwrite(page->data, 1, page->data_len, f);
    fclose(f);

    kreuzberg_free_render_page_result(page);
    return 0;
}
```
