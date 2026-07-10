#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    {
        struct ResultPool *pool = kreuzberg_result_pool_new(10);
        assert(pool != NULL);

        struct CResultPoolStats stats = kreuzberg_result_pool_stats(pool);
        assert(stats.current_count == 0);
        assert(stats.capacity == 10);
        assert(stats.total_allocations == 0);
        assert(stats.growth_events == 0);
        assert(stats.estimated_memory_bytes == 0);

        kreuzberg_result_pool_reset(pool);

        stats = kreuzberg_result_pool_stats(pool);
        assert(stats.current_count == 0);

        kreuzberg_result_pool_free(pool);
    }

    kreuzberg_result_pool_free(NULL);

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(0);
        if (pool != NULL) {
            struct CResultPoolStats stats = kreuzberg_result_pool_stats(pool);
            assert(stats.current_count == 0);
            assert(stats.capacity == 0);
            kreuzberg_result_pool_free(pool);
        }
    }

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(1000);
        assert(pool != NULL);

        struct CResultPoolStats stats = kreuzberg_result_pool_stats(pool);
        assert(stats.capacity == 1000);
        assert(stats.current_count == 0);
        assert(stats.total_allocations == 0);

        kreuzberg_result_pool_free(pool);
    }

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(5);
        assert(pool != NULL);

        kreuzberg_result_pool_reset(pool);
        kreuzberg_result_pool_reset(pool);
        kreuzberg_result_pool_reset(pool);

        struct CResultPoolStats stats = kreuzberg_result_pool_stats(pool);
        assert(stats.current_count == 0);

        kreuzberg_result_pool_free(pool);
    }

    {
        struct ResultPool *pool = kreuzberg_result_pool_new(10);
        assert(pool != NULL);

        const struct CExtractionResultView *view =
        kreuzberg_extract_file_into_pool("/nonexistent/file.txt", NULL, pool);

        assert(view == NULL);

        struct CResultPoolStats stats = kreuzberg_result_pool_stats(pool);
        assert(stats.current_count == 0);

        kreuzberg_result_pool_free(pool);
    }

    printf("test_result_pool: all tests passed\n");
    return 0;
}
