from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
from typing import Any

import pytest

from kreuzberg import (
    ChunkingConfig,
    ExtractionConfig,
    ImageExtractionConfig,
    LanguageDetectionConfig,
    OcrConfig,
    OutputFormat,
    PdfConfig,
    PostProcessorConfig,
    ResultFormat,
    TokenReductionConfig,
)

_WORKSPACE_ROOT = Path(__file__).resolve().parent.parent.parent.parent
_TEST_DOCUMENTS = _WORKSPACE_ROOT / "test_documents"


def resolve_document(relative: str) -> Path:
    """Return absolute path to a document in test_documents."""

    return _TEST_DOCUMENTS / relative


def build_config(config: dict[str, Any] | None) -> ExtractionConfig:
    """Construct an ExtractionConfig from a plain dictionary."""

    if not config:
        return ExtractionConfig()

    kwargs: dict[str, Any] = {}

    for key in ("use_cache", "enable_quality_processing", "force_ocr"):
        if key in config:
            kwargs[key] = config[key]

    if (ocr_data := config.get("ocr")) is not None:
        kwargs["ocr"] = OcrConfig(**ocr_data)

    if (chunking_data := config.get("chunking")) is not None:
        kwargs["chunking"] = ChunkingConfig(**chunking_data)

    if (images_data := config.get("images")) is not None:
        kwargs["images"] = ImageExtractionConfig(**images_data)

    if (pdf_options := config.get("pdf_options")) is not None:
        kwargs["pdf_options"] = PdfConfig(**pdf_options)

    if (token_reduction := config.get("token_reduction")) is not None:
        kwargs["token_reduction"] = TokenReductionConfig(**token_reduction)

    if (language_detection := config.get("language_detection")) is not None:
        kwargs["language_detection"] = LanguageDetectionConfig(**language_detection)

    if (postprocessor := config.get("postprocessor")) is not None:
        kwargs["postprocessor"] = PostProcessorConfig(**postprocessor)

    if (output_format := config.get("output_format")) is not None:
        kwargs["output_format"] = OutputFormat(output_format)

    if (result_format := config.get("result_format")) is not None:
        kwargs["result_format"] = ResultFormat(result_format)

    return ExtractionConfig(**kwargs)


def assert_expected_mime(result: Any, expected: list[str]) -> None:
    if not expected:
        return
    if not any(token in result.mime_type for token in expected):
        pytest.fail(f"Expected MIME {result.mime_type!r} to match one of {expected!r}")


def assert_min_content_length(result: Any, minimum: int) -> None:
    if len(result.content) < minimum:
        pytest.fail(
            f"Expected content length >= {minimum}, got {len(result.content)}"
        )


def assert_max_content_length(result: Any, maximum: int) -> None:
    if len(result.content) > maximum:
        pytest.fail(
            f"Expected content length <= {maximum}, got {len(result.content)}"
        )


def assert_content_contains_any(result: Any, snippets: list[str]) -> None:
    if not snippets:
        return
    lowered = result.content.lower()
    preview = result.content[:160]
    if not any(snippet.lower() in lowered for snippet in snippets):
        pytest.fail(
            f"Expected content to contain any of {snippets!r}. Preview: {preview!r}"
        )


def assert_content_contains_all(result: Any, snippets: list[str]) -> None:
    if not snippets:
        return
    lowered = result.content.lower()
    missing = [snippet for snippet in snippets if snippet.lower() not in lowered]
    if missing:
        pytest.fail(
            f"Expected content to contain all snippets {snippets!r}. Missing {missing!r}"
        )


def assert_table_count(result: Any, minimum: int | None, maximum: int | None) -> None:
    count = len(getattr(result, "tables", []) or [])
    if minimum is not None and count < minimum:
        pytest.fail(f"Expected at least {minimum} tables, found {count}")
    if maximum is not None and count > maximum:
        pytest.fail(f"Expected at most {maximum} tables, found {count}")


def assert_detected_languages(
    result: Any, expected: list[str], min_confidence: float | None
) -> None:
    if not expected:
        return
    languages = result.detected_languages
    if languages is None:
        pytest.fail("Expected detected languages but field is None")

    missing = [lang for lang in expected if lang not in languages]
    if missing:
        pytest.fail(f"Expected languages {expected!r}, missing {missing!r}")

    if min_confidence is not None:
        confidence = (
            result.metadata.get("confidence")
            if isinstance(result.metadata, Mapping)
            else None
        )
        if confidence is not None and confidence < min_confidence:
            pytest.fail(
                f"Expected confidence >= {min_confidence}, got {confidence}"
            )


def assert_metadata_expectation(result: Any, path: str, expectation: dict[str, Any]) -> None:
    value = _lookup_path(result.metadata, path)
    if value is None:
        pytest.fail(f"Metadata path '{path}' missing in {result.metadata!r}")

    if "eq" in expectation and not _values_equal(value, expectation["eq"]):
        pytest.fail(
            f"Expected metadata '{path}' == {expectation['eq']!r}, got {value!r}"
        )

    if "gte" in expectation:
        actual = float(value)
        if actual < float(expectation["gte"]):
            pytest.fail(
                f"Expected metadata '{path}' >= {expectation['gte']}, got {actual}"
            )

    if "lte" in expectation:
        actual = float(value)
        if actual > float(expectation["lte"]):
            pytest.fail(
                f"Expected metadata '{path}' <= {expectation['lte']}, got {actual}"
            )

    if "contains" in expectation:
        expected_values = expectation["contains"]
        if isinstance(value, str) and isinstance(expected_values, str):
            if expected_values not in value:
                pytest.fail(
                    f"Expected metadata '{path}' string to contain {expected_values!r}"
                )
        elif isinstance(value, (list, tuple, set)) and isinstance(expected_values, str):
            if expected_values not in value:
                pytest.fail(
                    f"Expected metadata '{path}' to contain {expected_values!r}"
                )
        elif isinstance(value, (list, tuple, set)):
            missing = [item for item in expected_values if item not in value]
            if missing:
                pytest.fail(
                    f"Expected metadata '{path}' to contain {expected_values!r}, missing {missing!r}"
                )
        else:
            pytest.fail(
                f"Unsupported contains expectation for metadata '{path}': {value!r}"
            )

    if expectation.get("exists") is False:
        pytest.fail("exists=False not supported for metadata expectations")


def _lookup_path(metadata: Mapping[str, Any] | None, path: str) -> Any:
    if not isinstance(metadata, Mapping):
        return None

    def _lookup(source: Mapping[str, Any]) -> Any:
        current: Any = source
        for segment in path.split("."):
            if not isinstance(current, Mapping) or segment not in current:
                return None
            current = current[segment]
        return current

    direct = _lookup(metadata)
    if direct is not None:
        return direct

    format_metadata = metadata.get("format")
    if isinstance(format_metadata, Mapping):
        return _lookup(format_metadata)
    return None


def _values_equal(lhs: Any, rhs: Any) -> bool:
    if isinstance(lhs, str) and isinstance(rhs, str):
        return lhs == rhs
    if isinstance(lhs, (int, float)) and isinstance(rhs, (int, float)):
        return float(lhs) == float(rhs)
    if isinstance(lhs, bool) and isinstance(rhs, bool):
        return lhs is rhs
    return bool(lhs == rhs)


def assert_chunks(
    result: Any,
    min_count: int | None = None,
    max_count: int | None = None,
    each_has_content: bool | None = None,
    each_has_embedding: bool | None = None,
) -> None:
    chunks = getattr(result, "chunks", None)
    if chunks is None:
        pytest.fail("Expected chunks but got None")
    count = len(chunks)
    if min_count is not None and count < min_count:
        pytest.fail(f"Expected at least {min_count} chunks, found {count}")
    if max_count is not None and count > max_count:
        pytest.fail(f"Expected at most {max_count} chunks, found {count}")
    if each_has_content:
        for i, chunk in enumerate(chunks):
            if not getattr(chunk, "content", None):
                pytest.fail(f"Chunk {i} has no content")
    if each_has_embedding:
        for i, chunk in enumerate(chunks):
            if not getattr(chunk, "embedding", None):
                pytest.fail(f"Chunk {i} has no embedding")


def assert_images(
    result: Any,
    min_count: int | None = None,
    max_count: int | None = None,
    formats_include: list[str] | None = None,
) -> None:
    images = getattr(result, "images", None)
    if images is None:
        pytest.fail("Expected images but got None")
    count = len(images)
    if min_count is not None and count < min_count:
        pytest.fail(f"Expected at least {min_count} images, found {count}")
    if max_count is not None and count > max_count:
        pytest.fail(f"Expected at most {max_count} images, found {count}")
    if formats_include:
        found_formats = {getattr(img, "format", None) for img in images}
        for fmt in formats_include:
            if fmt not in found_formats:
                pytest.fail(f"Expected image format {fmt!r} not found in {found_formats}")


def assert_pages(
    result: Any,
    min_count: int | None = None,
    exact_count: int | None = None,
) -> None:
    pages = getattr(result, "pages", None)
    if pages is None:
        pytest.fail("Expected pages but got None")
    count = len(pages)
    if exact_count is not None and count != exact_count:
        pytest.fail(f"Expected exactly {exact_count} pages, found {count}")
    if min_count is not None and count < min_count:
        pytest.fail(f"Expected at least {min_count} pages, found {count}")


def assert_elements(
    result: Any,
    min_count: int | None = None,
    types_include: list[str] | None = None,
) -> None:
    elements = getattr(result, "elements", None)
    if elements is None:
        pytest.fail("Expected elements but got None")
    count = len(elements)
    if min_count is not None and count < min_count:
        pytest.fail(f"Expected at least {min_count} elements, found {count}")
    if types_include:
        found_types = {getattr(el, "type", None) for el in elements}
        for el_type in types_include:
            if el_type not in found_types:
                pytest.fail(f"Expected element type {el_type!r} not found in {found_types}")


def assert_output_format(result: Any, expected: str) -> None:
    actual = getattr(result, "output_format", None)
    if actual is None:
        pytest.fail("Expected output_format but field is None")
    actual_value = actual.value if hasattr(actual, "value") else str(actual)
    if actual_value.lower() != expected.lower():
        pytest.fail(f"Expected output_format {expected!r}, got {actual_value!r}")


def assert_result_format(result: Any, expected: str) -> None:
    actual = getattr(result, "result_format", None)
    if actual is None:
        pytest.fail("Expected result_format but field is None")
    actual_value = actual.value if hasattr(actual, "value") else str(actual)
    if actual_value.lower() != expected.lower():
        pytest.fail(f"Expected result_format {expected!r}, got {actual_value!r}")
