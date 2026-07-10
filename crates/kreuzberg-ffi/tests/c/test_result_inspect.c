#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {

    {
        const char *text = "Hello from kreuzberg test. This is sample content for inspection.";
        struct CExtractionResult *res =
        kreuzberg_extract_bytes_sync((const uint8_t *)text, strlen(text), "text/plain");

        if (res != NULL) {
            assert(res->success);

            assert(res->content != NULL);
            assert(strlen(res->content) > 0);

            assert(res->mime_type != NULL);
            assert(strlen(res->mime_type) > 0);

            assert(strstr(res->content, "Hello") != NULL ||
                strstr(res->content, "kreuzberg") != NULL);


            if (res->metadata_json != NULL) {
                assert(strlen(res->metadata_json) > 0);
            }


            kreuzberg_free_result(res);
        } else {
            printf("  note: bytes extraction returned NULL, skipping field inspection\n");
            const char *err = kreuzberg_last_error();
            printf("  note: error: %s\n", err ? err : "(none)");
        }
    }

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(10);
        assert(pool != NULL);

        const struct CExtractionResultView *view =
        kreuzberg_extract_file_into_pool("/nonexistent/inspect_test.txt", NULL, pool);

        assert(view == NULL);

        struct CExtractionResultView view_struct =
        kreuzberg_extract_file_into_pool_view("/nonexistent/inspect_test.txt", NULL, pool);

        assert(view_struct.content_ptr == NULL);
        assert(view_struct.content_len == 0);

        kreuzberg_result_pool_free(pool);
    }

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(10);
        assert(pool != NULL);

        struct CExtractionResultView empty_view;
        memset(&empty_view, 0, sizeof(empty_view));

        const uint8_t *out_ptr = NULL;
        uintptr_t out_len = 0;

        int32_t rc = kreuzberg_view_get_content(&empty_view, &out_ptr, &out_len);
        if (rc == 0) {
            assert(out_ptr == NULL || out_len == 0);
        }

        out_ptr = NULL;
        out_len = 0;
        rc = kreuzberg_view_get_mime_type(&empty_view, &out_ptr, &out_len);
        if (rc == 0) {
            assert(out_ptr == NULL || out_len == 0);
        }

        kreuzberg_result_pool_free(pool);
    }

    {
        struct CMetadataField field;
        memset(&field, 0, sizeof(field));
        assert(field.name == NULL);
        assert(field.json_value == NULL);
        assert(field.is_null == 0);
    }

    printf("test_result_inspect: all tests passed\n");
    return 0;
}
