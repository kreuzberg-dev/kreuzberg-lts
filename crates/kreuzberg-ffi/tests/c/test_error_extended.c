#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    {
        uint32_t code = kreuzberg_classify_error("Failed to open file: permission denied");
        assert(code == kreuzberg_error_code_io());

        code = kreuzberg_classify_error("validation failed: invalid input");
        assert(code == kreuzberg_error_code_validation());

        code = kreuzberg_classify_error("parse error: unexpected token");
        assert(code == kreuzberg_error_code_parsing());

        code = kreuzberg_classify_error("unsupported type: x-custom");
        assert(code == kreuzberg_error_code_unsupported_format());

        code = kreuzberg_classify_error("something happened");
        (void)code;

        code = kreuzberg_classify_error(NULL);
        (void)code;
    }

    {
        char *context = kreuzberg_last_panic_context();
        if (context != NULL) {
            printf("  note: panic context unexpectedly non-NULL: %s\n", context);
            kreuzberg_free_string(context);
        }
    }

    {
        const char *original = "Hello, kreuzberg clone test!";
        char *cloned = kreuzberg_clone_string(original);
        assert(cloned != NULL);
        assert(strcmp(cloned, original) == 0);
        assert(cloned != original);
        kreuzberg_free_string(cloned);
    }

    {
        const char *original = "";
        char *cloned = kreuzberg_clone_string(original);
        assert(cloned != NULL);
        assert(strcmp(cloned, original) == 0);
        assert(strlen(cloned) == 0);
        kreuzberg_free_string(cloned);
    }

    {
        const char *cloned = kreuzberg_clone_string(NULL);
        assert(cloned == NULL);
    }

    kreuzberg_free_error_details(NULL);

    {
        struct CErrorDetails *details = kreuzberg_get_error_details_ptr();
        if (details != NULL) {
            kreuzberg_free_error_details(details);
        }
    }

    {
        const struct CExtractionResult *result = kreuzberg_extract_file_sync(NULL);
        assert(result == NULL);

        struct CErrorDetails *details = kreuzberg_get_error_details_ptr();
        if (details != NULL) {
            assert(details->message != NULL);
            assert(strlen(details->message) > 0);

            if (details->error_type != NULL) {
                assert(strlen(details->error_type) > 0);
            }



            kreuzberg_free_error_details(details);
        }
    }

    {
        const struct CExtractionResult *result =
        kreuzberg_extract_file_sync("/nonexistent/error_test.pdf");
        assert(result == NULL);

        struct CErrorDetails details = kreuzberg_get_error_details();

        if (details.message != NULL) {
            assert(strlen(details.message) > 0);
            kreuzberg_free_string(details.message);
        }
        if (details.error_type != NULL) {
            kreuzberg_free_string(details.error_type);
        }
        if (details.source_file != NULL) {
            kreuzberg_free_string(details.source_file);
        }
        if (details.source_function != NULL) {
            kreuzberg_free_string(details.source_function);
        }
        if (details.context_info != NULL) {
            kreuzberg_free_string(details.context_info);
        }
    }

    {
        uint32_t validation_code = kreuzberg_error_code_validation();
        uint32_t io_code = kreuzberg_error_code_io();
        uint32_t parse_code = kreuzberg_error_code_parsing();

        assert(validation_code != io_code);
        assert(validation_code != parse_code);
        assert(io_code != parse_code);
    }

    {
        const char *long_str = "This is a longer string to test kreuzberg_clone_string with "
                               "more content. It includes multiple sentences and should be "
                               "cloned exactly as-is without any truncation or modification.";
        char *cloned = kreuzberg_clone_string(long_str);
        assert(cloned != NULL);
        assert(strcmp(cloned, long_str) == 0);
        assert(strlen(cloned) == strlen(long_str));
        kreuzberg_free_string(cloned);
    }

    printf("test_error_extended: all tests passed\n");
    return 0;
}
