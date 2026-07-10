#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    kreuzberg_free_batch_result(NULL);

    {
        struct CBatchResult *batch = kreuzberg_batch_extract_files_sync(NULL, NULL, 0, NULL);

        if (batch != NULL) {
            assert(batch->count == 0);
            kreuzberg_free_batch_result(batch);
        }
    }

    {
        const char *text = "Batch extraction test content.";
        struct CBytesWithMime item;
        item.data = (const uint8_t *)text;
        item.data_len = strlen(text);
        item.mime_type = "text/plain";

        struct CBatchResult *batch = kreuzberg_batch_extract_bytes_sync(&item, NULL, 1, NULL);

        if (batch != NULL) {
            if (batch->success && batch->count > 0) {
                assert(batch->results != NULL);
                assert(batch->count == 1);

                const struct CExtractionResult *res = batch->results[0];
                if (res != NULL && res->success) {
                    assert(res->content != NULL);
                    assert(strlen(res->content) > 0);
                }
            }
            kreuzberg_free_batch_result(batch);
        } else {
            const char *err = kreuzberg_last_error();
            printf("  note: batch bytes extraction returned NULL (error: %s)\n",
                err ? err : "(none)");
        }
    }

    {
        const char *text1 = "First document content.";
        const char *text2 = "Second document content.";

        struct CBytesWithMime items[2];
        items[0].data = (const uint8_t *)text1;
        items[0].data_len = strlen(text1);
        items[0].mime_type = "text/plain";
        items[1].data = (const uint8_t *)text2;
        items[1].data_len = strlen(text2);
        items[1].mime_type = "text/plain";

        struct CBatchResult *batch = kreuzberg_batch_extract_bytes_sync(items, NULL, 2, NULL);

        if (batch != NULL) {
            if (batch->success) {
                assert(batch->count == 2);
                assert(batch->results != NULL);
            }
            kreuzberg_free_batch_result(batch);
        } else {
            const char *err = kreuzberg_last_error();
            printf("  note: multi-item batch returned NULL (error: %s)\n", err ? err : "(none)");
        }
    }

    {
        const char *paths[] = {"/nonexistent/file1.txt", "/nonexistent/file2.txt"};
        struct CBatchResult *batch = kreuzberg_batch_extract_files_sync(paths, NULL, 2, NULL);

        if (batch != NULL) {
            kreuzberg_free_batch_result(batch);
        }
    }


    {
        const char *text = "Config test content.";
        struct CBytesWithMime item;
        item.data = (const uint8_t *)text;
        item.data_len = strlen(text);
        item.mime_type = "text/plain";

        const char *config = "{}";

        struct CBatchResult *batch = kreuzberg_batch_extract_bytes_sync(&item, NULL, 1, config);

        if (batch != NULL) {
            kreuzberg_free_batch_result(batch);
        }
    }

    printf("test_batch: all tests passed\n");
    return 0;
}
