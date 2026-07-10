#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    ExtractionConfig *config = kreuzberg_config_from_json("{}");
    assert(config != NULL);

    char *json = kreuzberg_config_to_json(config);
    assert(json != NULL);
    assert(strlen(json) > 0);
    kreuzberg_free_string(json);

    char *field = kreuzberg_config_get_field(config, "use_cache");
    if (field != NULL) {
        assert(strlen(field) > 0);
        kreuzberg_free_string(field);
    }

    ExtractionConfig *overlay = kreuzberg_config_from_json("{}");
    assert(overlay != NULL);
    int32_t merge_result = kreuzberg_config_merge(config, overlay);
    assert(merge_result == 1);
    kreuzberg_config_free(overlay);

    kreuzberg_config_free(config);

    const ExtractionConfig *bad_config = kreuzberg_config_from_json("not valid json");
    assert(bad_config == NULL);

    assert(kreuzberg_config_is_valid("{}") == 1);

    assert(kreuzberg_config_is_valid("not valid json") == 0);

    kreuzberg_config_free(NULL);

    char *discovered = kreuzberg_config_discover();
    if (discovered != NULL) {
        kreuzberg_free_string(discovered);
    }

    char *presets = kreuzberg_list_embedding_presets();
    assert(presets != NULL);
    assert(presets[0] == '[');
    kreuzberg_free_string(presets);

    char *bad_preset = kreuzberg_get_embedding_preset("nonexistent_preset_xyz");
    if (bad_preset != NULL) {
        kreuzberg_free_string(bad_preset);
    }

    printf("test_config: all tests passed\n");
    return 0;
}
