#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    struct ConfigBuilder *builder = kreuzberg_config_builder_new();
    assert(builder != NULL);

    ExtractionConfig *config = kreuzberg_config_builder_build(builder);
    assert(config != NULL);

    char *json = kreuzberg_config_to_json(config);
    assert(json != NULL);
    assert(strlen(json) > 0);
    kreuzberg_free_string(json);
    kreuzberg_config_free(config);

    builder = kreuzberg_config_builder_new();
    assert(builder != NULL);

    int32_t rc = kreuzberg_config_builder_set_use_cache(builder, 1);
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_include_document_structure(builder, 0);
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_ocr(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_pdf(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_chunking(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_image_extraction(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_post_processor(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_language_detection(builder, "{}");
    assert(rc == 0);

    rc = kreuzberg_config_builder_set_content_filter(
        builder, "{\"include_headers\":true,\"strip_repeating_text\":false}");
    assert(rc == 0);

    config = kreuzberg_config_builder_build(builder);
    assert(config != NULL);

    json = kreuzberg_config_to_json(config);
    assert(json != NULL);
    assert(strlen(json) > 0);
    kreuzberg_free_string(json);
    kreuzberg_config_free(config);

    builder = kreuzberg_config_builder_new();
    assert(builder != NULL);
    kreuzberg_config_builder_free(builder);

    kreuzberg_config_builder_free(NULL);

    printf("test_config_builder: all tests passed\n");
    return 0;
}
