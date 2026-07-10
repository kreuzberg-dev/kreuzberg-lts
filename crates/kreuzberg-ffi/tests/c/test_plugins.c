#include "../../kreuzberg.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ---- Stub callbacks for each plugin type ---- */

static char *my_doc_extractor(const uint8_t *content, uintptr_t content_len, const char *mime_type,
    const char *config_json) {
    (void)content;
    (void)content_len;
    (void)mime_type;
    (void)config_json;
    return NULL;
}

static char *my_ocr_backend(const uint8_t *image_bytes, uintptr_t image_length,
    const char *config_json) {
    (void)image_bytes;
    (void)image_length;
    (void)config_json;
    return NULL;
}

static char *my_post_processor(const char *result_json) {
    (void)result_json;
    return NULL;
}

static char *my_validator(const char *result_json) {
    (void)result_json;
    return NULL;
}

static int json_list_contains(const char *json, const char *name) {
    if (json == NULL || name == NULL)
    return 0;
    return strstr(json, name) != NULL;
}

int main(void) {
    printf("  testing document extractors...\n");
    {
        bool ok = kreuzberg_clear_document_extractors();
        assert(ok);

        char *list = kreuzberg_list_document_extractors();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-doc-extractor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_document_extractor("test-doc-extractor", my_doc_extractor,
            "application/x-test", 100);
        assert(ok);

        list = kreuzberg_list_document_extractors();
        assert(list != NULL);
        assert(json_list_contains(list, "test-doc-extractor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_document_extractor("test-doc-extractor");
        assert(ok);

        list = kreuzberg_list_document_extractors();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-doc-extractor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_document_extractor("nonexistent-extractor");
        assert(ok);

        ok = kreuzberg_clear_document_extractors();
        assert(ok);
    }

    printf("  testing OCR backends...\n");
    {
        bool ok = kreuzberg_clear_ocr_backends();
        assert(ok);

        char *list = kreuzberg_list_ocr_backends();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-ocr"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_ocr_backend("test-ocr", my_ocr_backend);
        assert(ok);

        list = kreuzberg_list_ocr_backends();
        assert(list != NULL);
        assert(json_list_contains(list, "test-ocr"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_ocr_backend("test-ocr");
        assert(ok);

        list = kreuzberg_list_ocr_backends();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-ocr"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_ocr_backend_with_languages("test-ocr-lang", my_ocr_backend,
            "[\"en\", \"de\", \"fr\"]");
        assert(ok);

        list = kreuzberg_list_ocr_backends();
        assert(list != NULL);
        assert(json_list_contains(list, "test-ocr-lang"));
        kreuzberg_free_string(list);

        char *languages = kreuzberg_get_ocr_languages("test-ocr-lang");
        if (languages != NULL) {
            assert(strlen(languages) > 0);
            kreuzberg_free_string(languages);
        }

        int32_t supported = kreuzberg_is_language_supported("test-ocr-lang", "en");
        assert(supported == 0 || supported == 1);

        supported = kreuzberg_is_language_supported("test-ocr-lang", "zh");
        assert(supported == 0 || supported == 1);

        supported = kreuzberg_is_language_supported(NULL, "en");
        assert(supported == 0);

        supported = kreuzberg_is_language_supported("test-ocr-lang", NULL);
        assert(supported == 0);

        char *backends_with_langs = kreuzberg_list_ocr_backends_with_languages();
        if (backends_with_langs != NULL) {
            assert(strlen(backends_with_langs) > 0);
            kreuzberg_free_string(backends_with_langs);
        }

        ok = kreuzberg_clear_ocr_backends();
        assert(ok);

        list = kreuzberg_list_ocr_backends();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-ocr-lang"));
        kreuzberg_free_string(list);
    }

    printf("  testing post-processors...\n");
    {
        bool ok = kreuzberg_clear_post_processors();
        assert(ok);

        char *list = kreuzberg_list_post_processors();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-processor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_post_processor("test-processor", my_post_processor, 100);
        assert(ok);

        list = kreuzberg_list_post_processors();
        assert(list != NULL);
        assert(json_list_contains(list, "test-processor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_post_processor("test-processor");
        assert(ok);

        list = kreuzberg_list_post_processors();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-processor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_post_processor_with_stage("test-stage-processor", my_post_processor,
            50, "early");
        assert(ok);

        list = kreuzberg_list_post_processors();
        assert(list != NULL);
        assert(json_list_contains(list, "test-stage-processor"));
        kreuzberg_free_string(list);

        ok = kreuzberg_clear_post_processors();
        assert(ok);

        list = kreuzberg_list_post_processors();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-stage-processor"));
        kreuzberg_free_string(list);
    }

    printf("  testing validators...\n");
    {
        bool ok = kreuzberg_clear_validators();
        assert(ok);

        char *list = kreuzberg_list_validators();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-validator"));
        kreuzberg_free_string(list);

        ok = kreuzberg_register_validator("test-validator", my_validator, 100);
        assert(ok);

        list = kreuzberg_list_validators();
        assert(list != NULL);
        assert(json_list_contains(list, "test-validator"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_validator("test-validator");
        assert(ok);

        list = kreuzberg_list_validators();
        assert(list != NULL);
        assert(!json_list_contains(list, "test-validator"));
        kreuzberg_free_string(list);

        ok = kreuzberg_unregister_validator("nonexistent-validator");
        assert(ok);

        ok = kreuzberg_clear_validators();
        assert(ok);
    }

    printf("test_plugins: all tests passed\n");
    return 0;
}
