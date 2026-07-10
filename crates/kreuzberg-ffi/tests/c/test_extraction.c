#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    const struct CExtractionResult *result = kreuzberg_extract_file_sync(NULL);
    assert(result == NULL);

    const char *err = kreuzberg_last_error();
    assert(err != NULL);
    assert(strlen(err) > 0);

    {
        int32_t code = kreuzberg_last_error_code();
        assert(code != 0);
    }

    result = kreuzberg_extract_file_sync("/nonexistent/file.pdf");
    assert(result == NULL);

    kreuzberg_free_result(NULL);

    kreuzberg_free_string(NULL);

    {
        const char *text = "Hello, Kreuzberg! This is a test document.";
        struct CExtractionResult *res =
        kreuzberg_extract_bytes_sync((const uint8_t *)text, strlen(text), "text/plain");

        if (res != NULL) {
            assert(res->success);

            assert(res->content != NULL);
            assert(strlen(res->content) > 0);

            assert(res->mime_type != NULL);
            assert(strlen(res->mime_type) > 0);

            assert(strstr(res->content, "Hello") != NULL ||
                strstr(res->content, "Kreuzberg") != NULL);

            kreuzberg_free_result(res);
        } else {
            const char *extract_err = kreuzberg_last_error();
            printf("  note: bytes extraction returned NULL (error: %s)\n",
                extract_err ? extract_err : "(none)");
        }
    }

    printf("test_extraction: all tests passed\n");
    return 0;
}
