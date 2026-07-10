#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {

    kreuzberg_string_intern_reset();

    struct CStringInternStats baseline = kreuzberg_string_intern_stats();

    const char *s1 = kreuzberg_intern_string("x-test/unique-string-12345");
    assert(s1 != NULL);
    assert(strcmp(s1, "x-test/unique-string-12345") == 0);

    const char *s2 = kreuzberg_intern_string("x-test/unique-string-12345");
    assert(s2 != NULL);
    assert(s1 == s2);

    const char *s3 = kreuzberg_intern_string("x-test/another-unique-67890");
    assert(s3 != NULL);
    assert(strcmp(s3, "x-test/another-unique-67890") == 0);
    assert(s3 != s1);

    struct CStringInternStats stats = kreuzberg_string_intern_stats();
    assert(stats.unique_count == baseline.unique_count + 2);
    assert(stats.total_requests == 3);
    assert(stats.cache_hits >= 1);
    assert(stats.total_memory_bytes > 0);

    kreuzberg_free_interned_string(s1);
    kreuzberg_free_interned_string(s2);
    kreuzberg_free_interned_string(s3);

    kreuzberg_string_intern_reset();
    stats = kreuzberg_string_intern_stats();
    assert(stats.total_requests == 0);
    assert(stats.cache_hits == 0);
    assert(stats.cache_misses == 0);

    kreuzberg_free_interned_string(NULL);

    printf("test_string_intern: all tests passed\n");
    return 0;
}
