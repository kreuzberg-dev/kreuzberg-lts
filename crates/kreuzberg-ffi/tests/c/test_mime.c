#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

static void create_temp_file(const char *name) {
    FILE *f = fopen(name, "w");
    assert(f != NULL);
    fclose(f);
}

int main(void) {
    create_temp_file("_test_mime.pdf");
    create_temp_file("_test_mime.txt");
    create_temp_file("_test_mime.html");

    char *mime = kreuzberg_detect_mime_type_from_path("_test_mime.pdf");
    assert(mime != NULL);
    assert(strcmp(mime, "application/pdf") == 0);
    kreuzberg_free_string(mime);

    mime = kreuzberg_detect_mime_type_from_path("_test_mime.txt");
    assert(mime != NULL);
    assert(strcmp(mime, "text/plain") == 0);
    kreuzberg_free_string(mime);

    mime = kreuzberg_detect_mime_type_from_path("_test_mime.html");
    assert(mime != NULL);
    assert(strcmp(mime, "text/html") == 0);
    kreuzberg_free_string(mime);

    remove("_test_mime.pdf");
    remove("_test_mime.txt");
    remove("_test_mime.html");

    mime = kreuzberg_detect_mime_type_from_path("/nonexistent/file.pdf");
    assert(mime == NULL);

    char *valid = kreuzberg_validate_mime_type("application/pdf");
    assert(valid != NULL);
    kreuzberg_free_string(valid);

    valid = kreuzberg_validate_mime_type("text/plain");
    assert(valid != NULL);
    kreuzberg_free_string(valid);

    mime = kreuzberg_detect_mime_type_from_path(NULL);
    assert(mime == NULL);

    printf("test_mime: all tests passed\n");
    return 0;
}
