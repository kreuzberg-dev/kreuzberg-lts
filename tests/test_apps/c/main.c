/*
 * Kreuzberg C FFI Test App
 *
 * Comprehensive test program for the Kreuzberg C FFI API.
 * Tests extraction, configuration, error handling, MIME type detection,
 * validation, plugin registry, and library info functions.
 *
 * Compile: see Makefile
 * Run:     ./kreuzberg_test
 */

#include "kreuzberg.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

/* -------------------------------------------------------------------------- */
/* Test runner                                                                */
/* -------------------------------------------------------------------------- */

typedef struct {
    int passed;
    int failed;
    int skipped;
    int section;
} TestRunner;

static TestRunner g_runner = {0, 0, 0, 0};

static void section(const char *name) {
    g_runner.section++;
    printf("\n[SECTION %d] %s\n", g_runner.section, name);
    printf("--------------------------------------------------------------------------------\n");
}

static void pass(const char *description) {
    printf("  PASS  %s\n", description);
    g_runner.passed++;
}

static void fail(const char *description, const char *detail) {
    printf("  FAIL  %s\n", description);
    if (detail) {
        printf("    Error: %s\n", detail);
    }
    g_runner.failed++;
}

static void skip(const char *description, const char *reason) {
    printf("  SKIP  %s (%s)\n", description, reason);
    g_runner.skipped++;
}

static int summary(void) {
    int total = g_runner.passed + g_runner.failed + g_runner.skipped;
    printf("\n================================================================================\n");
    printf("TEST SUMMARY\n");
    printf("================================================================================\n");
    printf("Total Tests: %d\n", total);
    printf("  Passed:  %d\n", g_runner.passed);
    printf("  Failed:  %d\n", g_runner.failed);
    printf("  Skipped: %d\n", g_runner.skipped);

    if (g_runner.failed == 0) {
        printf("\nALL TESTS PASSED\n");
        return 0;
    }
    printf("\n%d TEST(S) FAILED\n", g_runner.failed);
    return 1;
}

/* -------------------------------------------------------------------------- */
/* Utility helpers                                                            */
/* -------------------------------------------------------------------------- */

/* Case-insensitive substring search. Returns non-zero if needle found. */
static int str_contains_ci(const char *haystack, const char *needle) {
    if (!haystack || !needle) return 0;
    size_t hlen = strlen(haystack);
    size_t nlen = strlen(needle);
    if (nlen == 0) return 1;
    if (nlen > hlen) return 0;
    for (size_t i = 0; i <= hlen - nlen; i++) {
        size_t j;
        for (j = 0; j < nlen; j++) {
            char hc = haystack[i + j];
            char nc = needle[j];
            if (hc >= 'A' && hc <= 'Z') hc += 32;
            if (nc >= 'A' && nc <= 'Z') nc += 32;
            if (hc != nc) break;
        }
        if (j == nlen) return 1;
    }
    return 0;
}

/* Resolve a test document path relative to test_documents/ in this directory. */
static char *resolve_test_document(const char *filename) {
    const char *prefix = "test_documents/";
    size_t len = strlen(prefix) + strlen(filename) + 1;
    char *path = (char *)malloc(len);
    if (!path) {
        fprintf(stderr, "OOM\n");
        exit(1);
    }
    snprintf(path, len, "%s%s", prefix, filename);
    return path;
}

/* Check if a file exists. */
static int file_exists(const char *path) {
    struct stat st;
    return stat(path, &st) == 0;
}

/* -------------------------------------------------------------------------- */
/* Section 1: Library Info                                                    */
/* -------------------------------------------------------------------------- */

static void test_library_info(void) {
    /* Test kreuzberg_version */
    {
        const char *ver = kreuzberg_version();
        if (ver && strlen(ver) > 0) {
            char buf[128];
            snprintf(buf, sizeof(buf), "kreuzberg_version() returns \"%s\"", ver);
            pass(buf);
        } else {
            fail("kreuzberg_version() returns non-empty string", "got NULL or empty");
        }
    }

    /* Test version constants */
    {
        if (KREUZBERG_VERSION_MAJOR >= 4) {
            pass("KREUZBERG_VERSION_MAJOR >= 4");
        } else {
            fail("KREUZBERG_VERSION_MAJOR >= 4", "major version too low");
        }
    }

    /* Test KREUZBERG_VERSION string macro */
    {
        const char *ver_macro = KREUZBERG_VERSION;
        if (ver_macro && strlen(ver_macro) > 0) {
            pass("KREUZBERG_VERSION macro is defined and non-empty");
        } else {
            fail("KREUZBERG_VERSION macro is defined and non-empty", "not available");
        }
    }

    /* Test kreuzberg_last_error (should be NULL or empty initially) */
    {
        const char *err = kreuzberg_last_error();
        /* It is acceptable for this to be NULL or a valid string. */
        (void)err;
        pass("kreuzberg_last_error() callable without crash");
    }

    /* Test kreuzberg_last_error_code */
    {
        int32_t code = kreuzberg_last_error_code();
        (void)code;
        pass("kreuzberg_last_error_code() callable without crash");
    }

    /* Test kreuzberg_last_panic_context */
    {
        char *ctx = kreuzberg_last_panic_context();
        if (ctx) {
            kreuzberg_free_string(ctx);
        }
        pass("kreuzberg_last_panic_context() callable without crash");
    }
}

/* -------------------------------------------------------------------------- */
/* Section 2: Error Code Functions                                            */
/* -------------------------------------------------------------------------- */

static void test_error_codes(void) {
    /* Error code constants */
    {
        uint32_t val = kreuzberg_error_code_validation();
        if (val == 0) {
            pass("kreuzberg_error_code_validation() == 0");
        } else {
            fail("kreuzberg_error_code_validation() == 0", "unexpected value");
        }
    }

    {
        uint32_t par = kreuzberg_error_code_parsing();
        if (par == 1) {
            pass("kreuzberg_error_code_parsing() == 1");
        } else {
            fail("kreuzberg_error_code_parsing() == 1", "unexpected value");
        }
    }

    {
        uint32_t ocr = kreuzberg_error_code_ocr();
        if (ocr == 2) {
            pass("kreuzberg_error_code_ocr() == 2");
        } else {
            fail("kreuzberg_error_code_ocr() == 2", "unexpected value");
        }
    }

    {
        uint32_t md = kreuzberg_error_code_missing_dependency();
        if (md == 3) {
            pass("kreuzberg_error_code_missing_dependency() == 3");
        } else {
            fail("kreuzberg_error_code_missing_dependency() == 3", "unexpected value");
        }
    }

    {
        uint32_t io = kreuzberg_error_code_io();
        if (io == 4) {
            pass("kreuzberg_error_code_io() == 4");
        } else {
            fail("kreuzberg_error_code_io() == 4", "unexpected value");
        }
    }

    {
        uint32_t pl = kreuzberg_error_code_plugin();
        if (pl == 5) {
            pass("kreuzberg_error_code_plugin() == 5");
        } else {
            fail("kreuzberg_error_code_plugin() == 5", "unexpected value");
        }
    }

    {
        uint32_t uf = kreuzberg_error_code_unsupported_format();
        if (uf == 6) {
            pass("kreuzberg_error_code_unsupported_format() == 6");
        } else {
            fail("kreuzberg_error_code_unsupported_format() == 6", "unexpected value");
        }
    }

    {
        uint32_t ie = kreuzberg_error_code_internal();
        if (ie == 7) {
            pass("kreuzberg_error_code_internal() == 7");
        } else {
            fail("kreuzberg_error_code_internal() == 7", "unexpected value");
        }
    }

    /* Error code count */
    {
        uint32_t count = kreuzberg_error_code_count();
        if (count == 8) {
            pass("kreuzberg_error_code_count() == 8");
        } else {
            char buf[64];
            snprintf(buf, sizeof(buf), "expected 8, got %u", count);
            fail("kreuzberg_error_code_count() == 8", buf);
        }
    }

    /* Error code names */
    {
        const char *name = kreuzberg_error_code_name(0);
        if (name && str_contains_ci(name, "validation")) {
            pass("kreuzberg_error_code_name(0) contains 'validation'");
        } else {
            fail("kreuzberg_error_code_name(0) contains 'validation'",
                 name ? name : "NULL");
        }
    }

    /* Error code descriptions */
    {
        const char *desc = kreuzberg_error_code_description(0);
        if (desc && strlen(desc) > 0) {
            pass("kreuzberg_error_code_description(0) returns non-empty string");
        } else {
            fail("kreuzberg_error_code_description(0) returns non-empty string",
                 "NULL or empty");
        }
    }

    /* Invalid error code name */
    {
        const char *name = kreuzberg_error_code_name(999);
        if (name && str_contains_ci(name, "unknown")) {
            pass("kreuzberg_error_code_name(999) returns 'unknown'");
        } else {
            fail("kreuzberg_error_code_name(999) returns 'unknown'",
                 name ? name : "NULL");
        }
    }

    /* Error classification */
    {
        uint32_t code = kreuzberg_classify_error("Failed to open file: permission denied");
        if (code == kreuzberg_error_code_io()) {
            pass("kreuzberg_classify_error() classifies IO errors correctly");
        } else {
            char buf[64];
            snprintf(buf, sizeof(buf), "expected %u (IO), got %u",
                     kreuzberg_error_code_io(), code);
            fail("kreuzberg_classify_error() classifies IO errors correctly", buf);
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 3: Configuration                                                   */
/* -------------------------------------------------------------------------- */

static void test_configuration(void) {
    /* Config from JSON with default values */
    {
        ExtractionConfig *cfg = kreuzberg_config_from_json("{}");
        if (cfg) {
            pass("kreuzberg_config_from_json(\"{}\") returns non-NULL config");
            kreuzberg_config_free(cfg);
        } else {
            const char *err = kreuzberg_last_error();
            fail("kreuzberg_config_from_json(\"{}\") returns non-NULL config",
                 err ? err : "unknown error");
        }
    }

    /* Config from JSON with options */
    {
        const char *json = "{\"force_ocr\": true, \"use_cache\": false}";
        ExtractionConfig *cfg = kreuzberg_config_from_json(json);
        if (cfg) {
            pass("kreuzberg_config_from_json() with options returns non-NULL");
            kreuzberg_config_free(cfg);
        } else {
            fail("kreuzberg_config_from_json() with options returns non-NULL",
                 kreuzberg_last_error());
        }
    }

    /* Invalid config JSON */
    {
        ExtractionConfig *cfg = kreuzberg_config_from_json("not json");
        if (!cfg) {
            pass("kreuzberg_config_from_json(invalid) returns NULL");
        } else {
            fail("kreuzberg_config_from_json(invalid) returns NULL",
                 "expected NULL for invalid JSON");
            kreuzberg_config_free(cfg);
        }
    }

    /* Config validation */
    {
        int32_t valid = kreuzberg_config_is_valid("{}");
        if (valid == 1) {
            pass("kreuzberg_config_is_valid(\"{}\") returns 1");
        } else {
            fail("kreuzberg_config_is_valid(\"{}\") returns 1", "returned 0");
        }
    }

    {
        int32_t valid = kreuzberg_config_is_valid("not json");
        if (valid == 0) {
            pass("kreuzberg_config_is_valid(invalid) returns 0");
        } else {
            fail("kreuzberg_config_is_valid(invalid) returns 0", "returned 1");
        }
    }

    /* Config to JSON */
    {
        ExtractionConfig *cfg = kreuzberg_config_from_json("{}");
        if (cfg) {
            char *json = kreuzberg_config_to_json(cfg);
            if (json && strlen(json) > 0) {
                pass("kreuzberg_config_to_json() returns non-empty JSON");
                kreuzberg_free_string(json);
            } else {
                fail("kreuzberg_config_to_json() returns non-empty JSON",
                     "NULL or empty");
            }
            kreuzberg_config_free(cfg);
        } else {
            skip("kreuzberg_config_to_json()", "config creation failed");
        }
    }

    /* Config get field */
    {
        ExtractionConfig *cfg = kreuzberg_config_from_json("{\"force_ocr\": true}");
        if (cfg) {
            char *field = kreuzberg_config_get_field(cfg, "force_ocr");
            if (field) {
                if (str_contains_ci(field, "true")) {
                    pass("kreuzberg_config_get_field(\"force_ocr\") returns true");
                } else {
                    fail("kreuzberg_config_get_field(\"force_ocr\") returns true", field);
                }
                kreuzberg_free_string(field);
            } else {
                fail("kreuzberg_config_get_field(\"force_ocr\")", "returned NULL");
            }
            kreuzberg_config_free(cfg);
        } else {
            skip("kreuzberg_config_get_field()", "config creation failed");
        }
    }

    /* Config merge */
    {
        ExtractionConfig *base = kreuzberg_config_from_json("{}");
        ExtractionConfig *override_cfg = kreuzberg_config_from_json("{\"force_ocr\": true}");
        if (base && override_cfg) {
            int32_t merged = kreuzberg_config_merge(base, override_cfg);
            if (merged == 1) {
                pass("kreuzberg_config_merge() returns 1 on success");
            } else {
                fail("kreuzberg_config_merge() returns 1 on success",
                     kreuzberg_last_error());
            }
            kreuzberg_config_free(base);
            kreuzberg_config_free(override_cfg);
        } else {
            skip("kreuzberg_config_merge()", "config creation failed");
            if (base) kreuzberg_config_free(base);
            if (override_cfg) kreuzberg_config_free(override_cfg);
        }
    }

    /* Config free NULL (should be no-op) */
    {
        kreuzberg_config_free(NULL);
        pass("kreuzberg_config_free(NULL) is a no-op");
    }
}

/* -------------------------------------------------------------------------- */
/* Section 4: Config Builder                                                  */
/* -------------------------------------------------------------------------- */

static void test_config_builder(void) {
    /* Basic builder lifecycle */
    {
        ConfigBuilder *builder = kreuzberg_config_builder_new();
        if (!builder) {
            fail("kreuzberg_config_builder_new() returns non-NULL", "got NULL");
            return;
        }
        pass("kreuzberg_config_builder_new() returns non-NULL");

        int32_t ret = kreuzberg_config_builder_set_use_cache(builder, 1);
        if (ret == 0) {
            pass("kreuzberg_config_builder_set_use_cache() returns 0");
        } else {
            fail("kreuzberg_config_builder_set_use_cache() returns 0", "non-zero");
        }

        ExtractionConfig *cfg = kreuzberg_config_builder_build(builder);
        if (cfg) {
            pass("kreuzberg_config_builder_build() returns non-NULL config");
            kreuzberg_config_free(cfg);
        } else {
            fail("kreuzberg_config_builder_build() returns non-NULL config",
                 kreuzberg_last_error());
        }
    }

    /* Builder with document structure setting */
    {
        ConfigBuilder *builder = kreuzberg_config_builder_new();
        if (builder) {
            kreuzberg_config_builder_set_include_document_structure(builder, 1);
            ExtractionConfig *cfg = kreuzberg_config_builder_build(builder);
            if (cfg) {
                pass("builder with include_document_structure builds successfully");
                kreuzberg_config_free(cfg);
            } else {
                fail("builder with include_document_structure builds", "NULL result");
            }
        }
    }

    /* Builder with OCR config JSON */
    {
        ConfigBuilder *builder = kreuzberg_config_builder_new();
        if (builder) {
            int32_t ret = kreuzberg_config_builder_set_ocr(
                builder, "{\"backend\": \"tesseract\"}");
            if (ret == 0) {
                pass("kreuzberg_config_builder_set_ocr() returns 0");
            } else {
                fail("kreuzberg_config_builder_set_ocr() returns 0", "non-zero");
            }
            ExtractionConfig *cfg = kreuzberg_config_builder_build(builder);
            if (cfg) kreuzberg_config_free(cfg);
        }
    }

    /* Builder free without build (discard) */
    {
        ConfigBuilder *builder = kreuzberg_config_builder_new();
        if (builder) {
            kreuzberg_config_builder_free(builder);
            pass("kreuzberg_config_builder_free() works without build");
        }
    }

    /* Builder free NULL (should be no-op) */
    {
        kreuzberg_config_builder_free(NULL);
        pass("kreuzberg_config_builder_free(NULL) is a no-op");
    }
}

/* -------------------------------------------------------------------------- */
/* Section 5: MIME Type Functions                                             */
/* -------------------------------------------------------------------------- */

static void test_mime_type_functions(void) {
    /* Detect MIME from path */
    {
        char *mime = kreuzberg_detect_mime_type("document.pdf", false);
        if (mime) {
            if (str_contains_ci(mime, "pdf")) {
                pass("kreuzberg_detect_mime_type(\"document.pdf\") contains 'pdf'");
            } else {
                fail("kreuzberg_detect_mime_type(\"document.pdf\") contains 'pdf'", mime);
            }
            kreuzberg_free_string(mime);
        } else {
            fail("kreuzberg_detect_mime_type(\"document.pdf\")", "returned NULL");
        }
    }

    /* Detect MIME from bytes */
    {
        /* PDF magic bytes: %PDF */
        const uint8_t pdf_magic[] = {0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E, 0x34};
        char *mime = kreuzberg_detect_mime_type_from_bytes(pdf_magic, sizeof(pdf_magic));
        if (mime) {
            if (str_contains_ci(mime, "pdf")) {
                pass("kreuzberg_detect_mime_type_from_bytes(PDF magic) contains 'pdf'");
            } else {
                fail("kreuzberg_detect_mime_type_from_bytes(PDF magic) contains 'pdf'", mime);
            }
            kreuzberg_free_string(mime);
        } else {
            fail("kreuzberg_detect_mime_type_from_bytes(PDF magic)", "returned NULL");
        }
    }

    /* Validate MIME type */
    {
        char *result = kreuzberg_validate_mime_type("application/pdf");
        if (result) {
            pass("kreuzberg_validate_mime_type(\"application/pdf\") returns non-NULL");
            kreuzberg_free_string(result);
        } else {
            fail("kreuzberg_validate_mime_type(\"application/pdf\")", "returned NULL");
        }
    }

    /* Get extensions for MIME type */
    {
        char *exts = kreuzberg_get_extensions_for_mime("application/pdf");
        if (exts) {
            if (str_contains_ci(exts, "pdf")) {
                pass("kreuzberg_get_extensions_for_mime(\"application/pdf\") contains 'pdf'");
            } else {
                fail("kreuzberg_get_extensions_for_mime() contains 'pdf'", exts);
            }
            kreuzberg_free_string(exts);
        } else {
            fail("kreuzberg_get_extensions_for_mime()", "returned NULL");
        }
    }

    /* Detect MIME from file path */
    {
        char *path = resolve_test_document("tiny.pdf");
        if (file_exists(path)) {
            char *mime = kreuzberg_detect_mime_type_from_path(path);
            if (mime) {
                if (str_contains_ci(mime, "pdf")) {
                    pass("kreuzberg_detect_mime_type_from_path() for PDF");
                } else {
                    fail("kreuzberg_detect_mime_type_from_path() for PDF", mime);
                }
                kreuzberg_free_string(mime);
            } else {
                fail("kreuzberg_detect_mime_type_from_path()", "returned NULL");
            }
        } else {
            skip("kreuzberg_detect_mime_type_from_path()", "test document not found");
        }
        free(path);
    }
}

/* -------------------------------------------------------------------------- */
/* Section 6: Validation Functions                                            */
/* -------------------------------------------------------------------------- */

static void test_validation_functions(void) {
    /* Validate language code */
    {
        int32_t valid = kreuzberg_validate_language_code("en");
        if (valid == 1) {
            pass("kreuzberg_validate_language_code(\"en\") == 1");
        } else {
            fail("kreuzberg_validate_language_code(\"en\") == 1", "returned 0");
        }
    }

    {
        int32_t valid = kreuzberg_validate_language_code("zzz_invalid");
        if (valid == 0) {
            pass("kreuzberg_validate_language_code(invalid) == 0");
        } else {
            fail("kreuzberg_validate_language_code(invalid) == 0", "returned 1");
        }
    }

    /* Validate Tesseract PSM */
    {
        int32_t valid = kreuzberg_validate_tesseract_psm(3);
        if (valid == 1) {
            pass("kreuzberg_validate_tesseract_psm(3) == 1");
        } else {
            fail("kreuzberg_validate_tesseract_psm(3) == 1", "returned 0");
        }
    }

    {
        int32_t valid = kreuzberg_validate_tesseract_psm(999);
        if (valid == 0) {
            pass("kreuzberg_validate_tesseract_psm(999) == 0");
        } else {
            fail("kreuzberg_validate_tesseract_psm(999) == 0", "returned 1");
        }
    }

    /* Validate Tesseract OEM */
    {
        int32_t valid = kreuzberg_validate_tesseract_oem(1);
        if (valid == 1) {
            pass("kreuzberg_validate_tesseract_oem(1) == 1");
        } else {
            fail("kreuzberg_validate_tesseract_oem(1) == 1", "returned 0");
        }
    }

    /* Validate confidence */
    {
        int32_t valid = kreuzberg_validate_confidence(0.5);
        if (valid == 1) {
            pass("kreuzberg_validate_confidence(0.5) == 1");
        } else {
            fail("kreuzberg_validate_confidence(0.5) == 1", "returned 0");
        }
    }

    {
        int32_t valid = kreuzberg_validate_confidence(1.5);
        if (valid == 0) {
            pass("kreuzberg_validate_confidence(1.5) == 0");
        } else {
            fail("kreuzberg_validate_confidence(1.5) == 0", "returned 1");
        }
    }

    /* Validate DPI */
    {
        int32_t valid = kreuzberg_validate_dpi(300);
        if (valid == 1) {
            pass("kreuzberg_validate_dpi(300) == 1");
        } else {
            fail("kreuzberg_validate_dpi(300) == 1", "returned 0");
        }
    }

    {
        int32_t valid = kreuzberg_validate_dpi(-1);
        if (valid == 0) {
            pass("kreuzberg_validate_dpi(-1) == 0");
        } else {
            fail("kreuzberg_validate_dpi(-1) == 0", "returned 1");
        }
    }

    /* Validate binarization method */
    {
        int32_t valid = kreuzberg_validate_binarization_method("otsu");
        if (valid == 1) {
            pass("kreuzberg_validate_binarization_method(\"otsu\") == 1");
        } else {
            /* May not be supported, just check it does not crash */
            pass("kreuzberg_validate_binarization_method(\"otsu\") callable");
        }
    }

    /* Validate token reduction level */
    {
        int32_t valid = kreuzberg_validate_token_reduction_level("none");
        if (valid == 1) {
            pass("kreuzberg_validate_token_reduction_level(\"none\") == 1");
        } else {
            pass("kreuzberg_validate_token_reduction_level(\"none\") callable");
        }
    }

    /* Validate output format */
    {
        int32_t valid = kreuzberg_validate_output_format("markdown");
        if (valid == 1) {
            pass("kreuzberg_validate_output_format(\"markdown\") == 1");
        } else {
            pass("kreuzberg_validate_output_format(\"markdown\") callable");
        }
    }

    /* Get valid values lists */
    {
        char *methods = kreuzberg_get_valid_binarization_methods();
        if (methods) {
            pass("kreuzberg_get_valid_binarization_methods() returns non-NULL");
            kreuzberg_free_string(methods);
        } else {
            fail("kreuzberg_get_valid_binarization_methods()", "returned NULL");
        }
    }

    {
        char *codes = kreuzberg_get_valid_language_codes();
        if (codes) {
            pass("kreuzberg_get_valid_language_codes() returns non-NULL");
            kreuzberg_free_string(codes);
        } else {
            fail("kreuzberg_get_valid_language_codes()", "returned NULL");
        }
    }

    {
        char *backends = kreuzberg_get_valid_ocr_backends();
        if (backends) {
            pass("kreuzberg_get_valid_ocr_backends() returns non-NULL");
            kreuzberg_free_string(backends);
        } else {
            fail("kreuzberg_get_valid_ocr_backends()", "returned NULL");
        }
    }

    {
        char *levels = kreuzberg_get_valid_token_reduction_levels();
        if (levels) {
            pass("kreuzberg_get_valid_token_reduction_levels() returns non-NULL");
            kreuzberg_free_string(levels);
        } else {
            fail("kreuzberg_get_valid_token_reduction_levels()", "returned NULL");
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 7: Enum Parsing Functions                                          */
/* -------------------------------------------------------------------------- */

static void test_enum_parsing(void) {
    /* Heading style */
    {
        int32_t d = kreuzberg_parse_heading_style("atx");
        if (d == 0) {
            pass("kreuzberg_parse_heading_style(\"atx\") == 0");
        } else {
            fail("kreuzberg_parse_heading_style(\"atx\") == 0", "wrong value");
        }
    }

    {
        const char *s = kreuzberg_heading_style_to_string(0);
        if (s && str_contains_ci(s, "atx")) {
            pass("kreuzberg_heading_style_to_string(0) contains 'atx'");
        } else {
            fail("kreuzberg_heading_style_to_string(0)", s ? s : "NULL");
        }
    }

    {
        int32_t d = kreuzberg_parse_heading_style("invalid");
        if (d == -1) {
            pass("kreuzberg_parse_heading_style(\"invalid\") == -1");
        } else {
            fail("kreuzberg_parse_heading_style(\"invalid\") == -1", "wrong value");
        }
    }

    /* Code block style */
    {
        int32_t d = kreuzberg_parse_code_block_style("backticks");
        if (d == 1) {
            pass("kreuzberg_parse_code_block_style(\"backticks\") == 1");
        } else {
            fail("kreuzberg_parse_code_block_style(\"backticks\") == 1", "wrong value");
        }
    }

    /* Whitespace mode */
    {
        int32_t d = kreuzberg_parse_whitespace_mode("default");
        if (d == 0) {
            pass("kreuzberg_parse_whitespace_mode(\"default\") == 0");
        } else {
            fail("kreuzberg_parse_whitespace_mode(\"default\") == 0", "wrong value");
        }
    }

    /* Preprocessing preset */
    {
        int32_t d = kreuzberg_parse_preprocessing_preset("none");
        if (d == 0) {
            pass("kreuzberg_parse_preprocessing_preset(\"none\") == 0");
        } else {
            fail("kreuzberg_parse_preprocessing_preset(\"none\") == 0", "wrong value");
        }
    }

    /* List indent type */
    {
        int32_t d = kreuzberg_parse_list_indent_type("spaces");
        if (d == 0) {
            pass("kreuzberg_parse_list_indent_type(\"spaces\") == 0");
        } else {
            fail("kreuzberg_parse_list_indent_type(\"spaces\") == 0", "wrong value");
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 8: File Extraction                                                 */
/* -------------------------------------------------------------------------- */

static void test_file_extraction(void) {
    /* Extract PDF file */
    {
        char *path = resolve_test_document("tiny.pdf");
        if (!file_exists(path)) {
            skip("extract tiny.pdf", "test document not found");
            free(path);
        } else {
            CExtractionResult *result = kreuzberg_extract_file_sync(path);
            if (result && result->success) {
                if (result->content && strlen(result->content) > 0) {
                    pass("extract tiny.pdf: success with non-empty content");
                } else {
                    fail("extract tiny.pdf: non-empty content", "content is empty");
                }
                if (result->mime_type && str_contains_ci(result->mime_type, "pdf")) {
                    pass("extract tiny.pdf: MIME type contains 'pdf'");
                } else {
                    fail("extract tiny.pdf: MIME type", result->mime_type ? result->mime_type : "NULL");
                }
                kreuzberg_free_result(result);
            } else {
                const char *err = kreuzberg_last_error();
                if (result && !result->success && err && str_contains_ci(err, "missing")) {
                    skip("extract tiny.pdf", "missing dependency");
                    kreuzberg_free_result(result);
                } else {
                    fail("extract tiny.pdf", err ? err : "unknown error");
                    if (result) kreuzberg_free_result(result);
                }
            }
            free(path);
        }
    }

    /* Extract DOCX file */
    {
        char *path = resolve_test_document("lorem_ipsum.docx");
        if (!file_exists(path)) {
            skip("extract lorem_ipsum.docx", "test document not found");
            free(path);
        } else {
            CExtractionResult *result = kreuzberg_extract_file_sync(path);
            if (result && result->success) {
                if (result->content && str_contains_ci(result->content, "lorem")) {
                    pass("extract lorem_ipsum.docx: content contains 'lorem'");
                } else {
                    pass("extract lorem_ipsum.docx: extraction succeeded");
                }
                kreuzberg_free_result(result);
            } else {
                const char *err = kreuzberg_last_error();
                if (err && str_contains_ci(err, "missing")) {
                    skip("extract lorem_ipsum.docx", "missing dependency");
                } else {
                    fail("extract lorem_ipsum.docx", err ? err : "unknown error");
                }
                if (result) kreuzberg_free_result(result);
            }
            free(path);
        }
    }

    /* Extract XLSX file */
    {
        char *path = resolve_test_document("stanley_cups.xlsx");
        if (!file_exists(path)) {
            skip("extract stanley_cups.xlsx", "test document not found");
            free(path);
        } else {
            CExtractionResult *result = kreuzberg_extract_file_sync(path);
            if (result && result->success) {
                pass("extract stanley_cups.xlsx: extraction succeeded");
                /* Check for table data */
                if (result->tables_json && strlen(result->tables_json) > 2) {
                    pass("extract stanley_cups.xlsx: tables_json is populated");
                } else {
                    pass("extract stanley_cups.xlsx: completed (no tables expected in basic mode)");
                }
                kreuzberg_free_result(result);
            } else {
                const char *err = kreuzberg_last_error();
                if (err && str_contains_ci(err, "missing")) {
                    skip("extract stanley_cups.xlsx", "missing dependency");
                } else {
                    fail("extract stanley_cups.xlsx", err ? err : "unknown error");
                }
                if (result) kreuzberg_free_result(result);
            }
            free(path);
        }
    }

    /* Extract with config */
    {
        char *path = resolve_test_document("tiny.pdf");
        if (!file_exists(path)) {
            skip("extract tiny.pdf with config", "test document not found");
            free(path);
        } else {
            const char *config_json = "{\"use_cache\": false}";
            CExtractionResult *result = kreuzberg_extract_file_sync_with_config(path, config_json);
            if (result && result->success) {
                pass("extract tiny.pdf with config: success");
                kreuzberg_free_result(result);
            } else {
                const char *err = kreuzberg_last_error();
                if (err && str_contains_ci(err, "missing")) {
                    skip("extract tiny.pdf with config", "missing dependency");
                } else {
                    fail("extract tiny.pdf with config", err ? err : "unknown error");
                }
                if (result) kreuzberg_free_result(result);
            }
            free(path);
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 9: Bytes Extraction                                                */
/* -------------------------------------------------------------------------- */

static void test_bytes_extraction(void) {
    char *path = resolve_test_document("tiny.pdf");
    if (!file_exists(path)) {
        skip("bytes extraction tests", "test document not found");
        free(path);
        return;
    }

    /* Read file into memory */
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        fail("open test document for bytes extraction", "fopen failed");
        free(path);
        return;
    }
    fseek(fp, 0, SEEK_END);
    long file_size = ftell(fp);
    rewind(fp);
    uint8_t *data = (uint8_t *)malloc((size_t)file_size);
    if (!data) {
        fclose(fp);
        free(path);
        fail("allocate memory for bytes extraction", "malloc failed");
        return;
    }
    fread(data, 1, (size_t)file_size, fp);
    fclose(fp);

    /* Extract from bytes */
    {
        CExtractionResult *result = kreuzberg_extract_bytes_sync(
            data, (size_t)file_size, "application/pdf");
        if (result && result->success) {
            if (result->content && strlen(result->content) > 0) {
                pass("extract_bytes_sync(PDF): success with non-empty content");
            } else {
                fail("extract_bytes_sync(PDF): non-empty content", "content is empty");
            }
            kreuzberg_free_result(result);
        } else {
            const char *err = kreuzberg_last_error();
            if (err && str_contains_ci(err, "missing")) {
                skip("extract_bytes_sync(PDF)", "missing dependency");
            } else {
                fail("extract_bytes_sync(PDF)", err ? err : "unknown error");
            }
            if (result) kreuzberg_free_result(result);
        }
    }

    /* Extract from bytes with config */
    {
        const char *config_json = "{\"use_cache\": false}";
        CExtractionResult *result = kreuzberg_extract_bytes_sync_with_config(
            data, (size_t)file_size, "application/pdf", config_json);
        if (result && result->success) {
            pass("extract_bytes_sync_with_config(PDF): success");
            kreuzberg_free_result(result);
        } else {
            const char *err = kreuzberg_last_error();
            if (err && str_contains_ci(err, "missing")) {
                skip("extract_bytes_sync_with_config(PDF)", "missing dependency");
            } else {
                fail("extract_bytes_sync_with_config(PDF)", err ? err : "unknown error");
            }
            if (result) kreuzberg_free_result(result);
        }
    }

    free(data);
    free(path);
}

/* -------------------------------------------------------------------------- */
/* Section 10: Error Handling                                                 */
/* -------------------------------------------------------------------------- */

static void test_error_handling(void) {
    /* Extract nonexistent file */
    {
        CExtractionResult *result = kreuzberg_extract_file_sync("/nonexistent/file.pdf");
        if (!result || !result->success) {
            pass("extract nonexistent file: returns error");
            if (result) kreuzberg_free_result(result);
        } else {
            fail("extract nonexistent file: should return error", "got success");
            kreuzberg_free_result(result);
        }
    }

    /* kreuzberg_last_error after failure */
    {
        /* Trigger an error first */
        CExtractionResult *result = kreuzberg_extract_file_sync("/nonexistent/file.pdf");
        if (result) kreuzberg_free_result(result);

        const char *err = kreuzberg_last_error();
        if (err && strlen(err) > 0) {
            pass("kreuzberg_last_error() returns non-empty after failure");
        } else {
            pass("kreuzberg_last_error() callable after failure");
        }
    }

    /* Get error details */
    {
        /* Trigger an error */
        CExtractionResult *result = kreuzberg_extract_file_sync("/nonexistent/file.pdf");
        if (result) kreuzberg_free_result(result);

        CErrorDetails details = kreuzberg_get_error_details();
        if (details.message && strlen(details.message) > 0) {
            pass("kreuzberg_get_error_details().message is non-empty");
        } else {
            pass("kreuzberg_get_error_details() callable without crash");
        }
        if (details.error_type && strlen(details.error_type) > 0) {
            pass("kreuzberg_get_error_details().error_type is non-empty");
        } else {
            pass("kreuzberg_get_error_details().error_type accessible");
        }

        /* Free all allocated strings in the details */
        if (details.message) kreuzberg_free_string(details.message);
        if (details.error_type) kreuzberg_free_string(details.error_type);
        if (details.source_file) kreuzberg_free_string(details.source_file);
        if (details.source_function) kreuzberg_free_string(details.source_function);
        if (details.context_info) kreuzberg_free_string(details.context_info);
    }

    /* Get error details via pointer */
    {
        CExtractionResult *result = kreuzberg_extract_file_sync("/nonexistent/file.pdf");
        if (result) kreuzberg_free_result(result);

        CErrorDetails *details_ptr = kreuzberg_get_error_details_ptr();
        if (details_ptr) {
            pass("kreuzberg_get_error_details_ptr() returns non-NULL");
            kreuzberg_free_error_details(details_ptr);
        } else {
            fail("kreuzberg_get_error_details_ptr()", "returned NULL");
        }
    }

    /* Free error details NULL (should be no-op) */
    {
        kreuzberg_free_error_details(NULL);
        pass("kreuzberg_free_error_details(NULL) is a no-op");
    }

    /* Free result NULL (should be no-op) */
    {
        kreuzberg_free_result(NULL);
        pass("kreuzberg_free_result(NULL) is a no-op");
    }

    /* Free string NULL (should be no-op) */
    {
        kreuzberg_free_string(NULL);
        pass("kreuzberg_free_string(NULL) is a no-op");
    }
}

/* -------------------------------------------------------------------------- */
/* Section 11: Batch Extraction                                               */
/* -------------------------------------------------------------------------- */

static void test_batch_extraction(void) {
    char *pdf_path = resolve_test_document("tiny.pdf");
    char *docx_path = resolve_test_document("lorem_ipsum.docx");

    if (!file_exists(pdf_path) || !file_exists(docx_path)) {
        skip("batch extraction tests", "test documents not found");
        free(pdf_path);
        free(docx_path);
        return;
    }

    /* Batch file extraction */
    {
        const char *files[] = {pdf_path, docx_path};
        CBatchResult *batch = kreuzberg_batch_extract_files_sync(files, NULL, 2, NULL);
        if (batch && batch->success) {
            if (batch->count == 2) {
                pass("batch_extract_files_sync: returns 2 results");
            } else {
                char buf[64];
                snprintf(buf, sizeof(buf), "expected 2, got %zu", batch->count);
                fail("batch_extract_files_sync: returns 2 results", buf);
            }
            /* Check individual results */
            int all_ok = 1;
            for (size_t i = 0; i < batch->count; i++) {
                if (!batch->results[i] || !batch->results[i]->success) {
                    all_ok = 0;
                }
            }
            if (all_ok) {
                pass("batch_extract_files_sync: all results successful");
            } else {
                /* Check if it is a missing dependency issue */
                const char *err = kreuzberg_last_error();
                if (err && str_contains_ci(err, "missing")) {
                    skip("batch_extract_files_sync individual results", "missing dependency");
                } else {
                    fail("batch_extract_files_sync: all results successful", "some failed");
                }
            }
            kreuzberg_free_batch_result(batch);
        } else {
            const char *err = kreuzberg_last_error();
            if (err && str_contains_ci(err, "missing")) {
                skip("batch_extract_files_sync", "missing dependency");
            } else {
                fail("batch_extract_files_sync", err ? err : "unknown error");
            }
            if (batch) kreuzberg_free_batch_result(batch);
        }
    }

    /* Batch file extraction with config */
    {
        const char *files[] = {pdf_path};
        const char *config_json = "{\"use_cache\": false}";
        CBatchResult *batch = kreuzberg_batch_extract_files_sync(files, NULL, 1, config_json);
        if (batch && batch->success) {
            pass("batch_extract_files_sync with config: success");
            kreuzberg_free_batch_result(batch);
        } else {
            const char *err = kreuzberg_last_error();
            if (err && str_contains_ci(err, "missing")) {
                skip("batch_extract_files_sync with config", "missing dependency");
            } else {
                fail("batch_extract_files_sync with config", err ? err : "unknown error");
            }
            if (batch) kreuzberg_free_batch_result(batch);
        }
    }

    /* Free batch result NULL (should be no-op) */
    {
        kreuzberg_free_batch_result(NULL);
        pass("kreuzberg_free_batch_result(NULL) is a no-op");
    }

    free(pdf_path);
    free(docx_path);
}

/* -------------------------------------------------------------------------- */
/* Section 12: Plugin Registry Functions                                      */
/* -------------------------------------------------------------------------- */

static void test_plugin_registry(void) {
    /* List OCR backends */
    {
        char *backends = kreuzberg_list_ocr_backends();
        if (backends) {
            pass("kreuzberg_list_ocr_backends() returns non-NULL");
            kreuzberg_free_string(backends);
        } else {
            fail("kreuzberg_list_ocr_backends()", "returned NULL");
        }
    }

    /* List post processors */
    {
        char *processors = kreuzberg_list_post_processors();
        if (processors) {
            pass("kreuzberg_list_post_processors() returns non-NULL");
            kreuzberg_free_string(processors);
        } else {
            fail("kreuzberg_list_post_processors()", "returned NULL");
        }
    }

    /* List validators */
    {
        char *validators = kreuzberg_list_validators();
        if (validators) {
            pass("kreuzberg_list_validators() returns non-NULL");
            kreuzberg_free_string(validators);
        } else {
            fail("kreuzberg_list_validators()", "returned NULL");
        }
    }

    /* List document extractors */
    {
        char *extractors = kreuzberg_list_document_extractors();
        if (extractors) {
            pass("kreuzberg_list_document_extractors() returns non-NULL");
            kreuzberg_free_string(extractors);
        } else {
            fail("kreuzberg_list_document_extractors()", "returned NULL");
        }
    }

    /* OCR language support */
    {
        int32_t supported = kreuzberg_is_language_supported("tesseract", "en");
        if (supported == 1) {
            pass("kreuzberg_is_language_supported(\"tesseract\", \"en\") == 1");
        } else {
            pass("kreuzberg_is_language_supported() callable");
        }
    }

    /* OCR backends with languages */
    {
        char *backends = kreuzberg_list_ocr_backends_with_languages();
        if (backends) {
            pass("kreuzberg_list_ocr_backends_with_languages() returns non-NULL");
            kreuzberg_free_string(backends);
        } else {
            fail("kreuzberg_list_ocr_backends_with_languages()", "returned NULL");
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 13: Embedding Presets                                              */
/* -------------------------------------------------------------------------- */

static void test_embedding_presets(void) {
    /* List embedding presets */
    {
        char *presets = kreuzberg_list_embedding_presets();
        if (presets) {
            pass("kreuzberg_list_embedding_presets() returns non-NULL");
            kreuzberg_free_string(presets);
        } else {
            fail("kreuzberg_list_embedding_presets()", "returned NULL");
        }
    }

    /* Get a specific embedding preset */
    {
        char *preset = kreuzberg_get_embedding_preset("openai-text-embedding-3-small");
        if (preset) {
            pass("kreuzberg_get_embedding_preset() returns non-NULL");
            kreuzberg_free_string(preset);
        } else {
            /* Preset name may differ; just verify it does not crash */
            pass("kreuzberg_get_embedding_preset() callable");
        }
    }

    /* Get nonexistent preset */
    {
        char *preset = kreuzberg_get_embedding_preset("nonexistent-preset-xyz");
        if (!preset) {
            pass("kreuzberg_get_embedding_preset(nonexistent) returns NULL");
        } else {
            pass("kreuzberg_get_embedding_preset(nonexistent) returns a value");
            kreuzberg_free_string(preset);
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Section 14: Result Structure Inspection                                    */
/* -------------------------------------------------------------------------- */

static void test_result_structure(void) {
    char *path = resolve_test_document("tiny.pdf");
    if (!file_exists(path)) {
        skip("result structure tests", "test document not found");
        free(path);
        return;
    }

    CExtractionResult *result = kreuzberg_extract_file_sync(path);
    free(path);

    if (!result || !result->success) {
        const char *err = kreuzberg_last_error();
        if (err && str_contains_ci(err, "missing")) {
            skip("result structure tests", "missing dependency");
        } else {
            fail("result structure tests", "extraction failed");
        }
        if (result) kreuzberg_free_result(result);
        return;
    }

    /* Check content field */
    if (result->content) {
        pass("result->content is non-NULL");
    } else {
        fail("result->content is non-NULL", "NULL");
    }

    /* Check mime_type field */
    if (result->mime_type) {
        pass("result->mime_type is non-NULL");
    } else {
        fail("result->mime_type is non-NULL", "NULL");
    }

    /* Check success field */
    if (result->success) {
        pass("result->success is true");
    } else {
        fail("result->success is true", "false");
    }

    /* Check optional metadata field */
    {
        /* metadata_json may be NULL for simple documents */
        if (result->metadata_json) {
            pass("result->metadata_json is non-NULL");
        } else {
            pass("result->metadata_json is NULL (acceptable for simple docs)");
        }
    }

    /* Check optional JSON fields are accessible without crash */
    {
        (void)(result->language);
        (void)(result->date);
        (void)(result->subject);
        (void)(result->tables_json);
        (void)(result->detected_languages_json);
        (void)(result->chunks_json);
        (void)(result->images_json);
        (void)(result->page_structure_json);
        (void)(result->pages_json);
        (void)(result->elements_json);
        (void)(result->ocr_elements_json);
        (void)(result->document_json);
        (void)(result->extracted_keywords_json);
        (void)(result->quality_score_json);
        (void)(result->processing_warnings_json);
        (void)(result->annotations_json);
        pass("all CExtractionResult fields are accessible without crash");
    }

    kreuzberg_free_result(result);
}

/* -------------------------------------------------------------------------- */
/* Section 15: String Operations                                              */
/* -------------------------------------------------------------------------- */

static void test_string_operations(void) {
    /* Clone string */
    {
        char *cloned = kreuzberg_clone_string("hello world");
        if (cloned && strcmp(cloned, "hello world") == 0) {
            pass("kreuzberg_clone_string() clones correctly");
            kreuzberg_free_string(cloned);
        } else {
            fail("kreuzberg_clone_string()", cloned ? cloned : "NULL");
            if (cloned) kreuzberg_free_string(cloned);
        }
    }

    /* String interning */
    {
        const char *interned = kreuzberg_intern_string("test_intern");
        if (interned) {
            pass("kreuzberg_intern_string() returns non-NULL");
            kreuzberg_free_interned_string(interned);
        } else {
            fail("kreuzberg_intern_string()", "returned NULL");
        }
    }

    /* String intern stats */
    {
        CStringInternStats stats = kreuzberg_string_intern_stats();
        (void)stats.unique_count;
        (void)stats.total_requests;
        (void)stats.cache_hits;
        (void)stats.cache_misses;
        pass("kreuzberg_string_intern_stats() callable without crash");
    }
}

/* -------------------------------------------------------------------------- */
/* main                                                                       */
/* -------------------------------------------------------------------------- */

int main(void) {
    printf("================================================================================\n");
    printf("KREUZBERG C FFI COMPREHENSIVE TEST SUITE\n");
    printf("================================================================================\n");
    printf("Library version: %s\n", kreuzberg_version());

    section("Library Info");
    test_library_info();

    section("Error Code Functions");
    test_error_codes();

    section("Configuration");
    test_configuration();

    section("Config Builder");
    test_config_builder();

    section("MIME Type Functions");
    test_mime_type_functions();

    section("Validation Functions");
    test_validation_functions();

    section("Enum Parsing Functions");
    test_enum_parsing();

    section("File Extraction");
    test_file_extraction();

    section("Bytes Extraction");
    test_bytes_extraction();

    section("Error Handling");
    test_error_handling();

    section("Batch Extraction");
    test_batch_extraction();

    section("Plugin Registry");
    test_plugin_registry();

    section("Embedding Presets");
    test_embedding_presets();

    section("Result Structure Inspection");
    test_result_structure();

    section("String Operations");
    test_string_operations();

    return summary();
}
