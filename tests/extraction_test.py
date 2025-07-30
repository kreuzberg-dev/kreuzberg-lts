from __future__ import annotations

import os
from pathlib import Path

import pytest

from kreuzberg import TesseractConfig
from kreuzberg._mime_types import (
    DOCX_MIME_TYPE,
    EXCEL_MIME_TYPE,
    MARKDOWN_MIME_TYPE,
    PDF_MIME_TYPE,
    PLAIN_TEXT_MIME_TYPE,
    POWER_POINT_MIME_TYPE,
)
from kreuzberg._types import ExtractionConfig, ExtractionResult
from kreuzberg.exceptions import ValidationError
from kreuzberg.extraction import (
    batch_extract_bytes,
    batch_extract_bytes_sync,
    batch_extract_file,
    batch_extract_file_sync,
    extract_bytes,
    extract_bytes_sync,
    extract_file,
    extract_file_sync,
)

IS_CI = os.environ.get("CI", "false").lower() == "true"


@pytest.mark.anyio
async def test_extract_bytes_pdf(scanned_pdf: Path) -> None:
    content = scanned_pdf.read_bytes()
    result = await extract_bytes(content, PDF_MIME_TYPE)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)


@pytest.mark.anyio
@pytest.mark.skip(reason="non_english_pdf fixture not available")
async def test_extract_bytes_pdf_non_english(non_english_pdf: Path) -> None:
    content = non_english_pdf.read_bytes()
    config = ExtractionConfig(ocr_backend="tesseract", ocr_config=TesseractConfig(language="deu"))
    result = await extract_bytes(content, PDF_MIME_TYPE, config=config)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)


@pytest.mark.anyio
async def test_extract_bytes_docx(docx_document: Path) -> None:
    content = docx_document.read_bytes()
    result = await extract_bytes(content, DOCX_MIME_TYPE)
    assert_extraction_result(result, mime_type=MARKDOWN_MIME_TYPE)
    assert result.content.strip() != ""


@pytest.mark.anyio
async def test_extract_bytes_excel(excel_document: Path) -> None:
    content = excel_document.read_bytes()
    result = await extract_bytes(content, EXCEL_MIME_TYPE)
    assert_extraction_result(result, mime_type=MARKDOWN_MIME_TYPE)
    assert result.content.strip() != ""


@pytest.mark.anyio
async def test_extract_bytes_pptx(pptx_document: Path) -> None:
    content = pptx_document.read_bytes()
    result = await extract_bytes(content, POWER_POINT_MIME_TYPE)
    assert_extraction_result(result, mime_type=MARKDOWN_MIME_TYPE)
    assert result.content.strip() != ""


@pytest.mark.anyio
async def test_extract_bytes_plain_text() -> None:
    content = b"This is plain text content."
    result = await extract_bytes(content, PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert result.content == "This is plain text content."


@pytest.mark.anyio
async def test_extract_bytes_invalid_mime() -> None:
    content = b"Some content"
    with pytest.raises(ValidationError):
        await extract_bytes(content, "application/unknown")


@pytest.mark.anyio
async def test_extract_file_pdf(scanned_pdf: Path) -> None:
    result = await extract_file(scanned_pdf)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)


@pytest.mark.anyio
async def test_extract_file_no_extension(tmp_path: Path) -> None:
    file_path = tmp_path / "file_without_extension"
    file_path.write_bytes(b"Text content")
    with pytest.raises(ValidationError):
        await extract_file(file_path)


@pytest.mark.anyio
async def test_extract_file_explicit_mime(tmp_path: Path) -> None:
    file_path = tmp_path / "file_without_extension"
    file_path.write_bytes(b"Text content")
    result = await extract_file(file_path, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert result.content == "Text content"


@pytest.mark.anyio
async def test_extract_file_not_exists() -> None:
    with pytest.raises(ValidationError) as exc_info:
        await extract_file("nonexistent_file.txt")
    assert "file does not exist" in str(exc_info.value)


@pytest.mark.anyio
async def test_batch_extract_file_single(test_article: Path) -> None:
    results = await batch_extract_file([str(test_article)])
    assert len(results) == 1
    assert_extraction_result(results[0], mime_type=PLAIN_TEXT_MIME_TYPE)


@pytest.mark.anyio
async def test_batch_extract_file_multiple(searchable_pdf: Path, test_article: Path, docx_document: Path) -> None:
    file_paths = [str(searchable_pdf), str(test_article), str(docx_document)]
    results = await batch_extract_file(file_paths)
    assert len(results) == 3
    assert_extraction_result(results[0], mime_type=PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(results[1], mime_type=PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(results[2], mime_type=MARKDOWN_MIME_TYPE)


@pytest.mark.anyio
async def test_batch_extract_bytes_single() -> None:
    contents = [(b"Single text content", PLAIN_TEXT_MIME_TYPE)]
    results = await batch_extract_bytes(contents)
    assert len(results) == 1
    assert_extraction_result(results[0], mime_type=PLAIN_TEXT_MIME_TYPE)
    assert results[0].content == "Single text content"


@pytest.mark.anyio
async def test_batch_extract_bytes_multiple(searchable_pdf: Path, docx_document: Path) -> None:
    contents = [
        (b"First text", PLAIN_TEXT_MIME_TYPE),
        (searchable_pdf.read_bytes(), PDF_MIME_TYPE),
        (docx_document.read_bytes(), DOCX_MIME_TYPE),
    ]
    results = await batch_extract_bytes(contents)
    assert len(results) == 3
    assert results[0].content == "First text"
    assert_extraction_result(results[1], mime_type=PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(results[2], mime_type=MARKDOWN_MIME_TYPE)


@pytest.mark.anyio
async def test_extract_file_with_custom_config(tmp_path: Path) -> None:
    file_path = tmp_path / "text.txt"
    file_path.write_text("Test content for extraction with config")

    custom_config = ExtractionConfig(chunk_content=True, max_chars=10, max_overlap=2)
    result = await extract_file(file_path, config=custom_config)

    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert len(result.chunks) > 0


@pytest.mark.anyio
async def test_batch_extract_empty_list() -> None:
    file_results = await batch_extract_file([])
    assert file_results == []

    bytes_results = await batch_extract_bytes([])
    assert bytes_results == []


@pytest.mark.anyio
@pytest.mark.xfail(
    not IS_CI, reason="GMFT tests may fail locally if gmft dependencies are not installed", raises=Exception
)
async def test_extract_pdf_with_tables(pdfs_with_tables_list: list[Path]) -> None:
    """Test table extraction from PDFs with GMFT enabled."""
    config = ExtractionConfig(extract_tables=True)

    for pdf in pdfs_with_tables_list:
        result = await extract_file(pdf, config=config)
        assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
        assert len(result.tables) > 0


@pytest.mark.anyio
async def test_extract_bytes_with_explicit_mime() -> None:
    """Test that extract_bytes works correctly with explicit mime type."""
    # Plain text should work with explicit mime type
    content = b"Plain text content"
    result = await extract_bytes(content, PLAIN_TEXT_MIME_TYPE)
    assert result.content == "Plain text content"


def assert_extraction_result(result: ExtractionResult, mime_type: str | None = None) -> None:
    """Helper to validate extraction results."""
    assert result is not None
    assert isinstance(result, ExtractionResult)
    assert result.content is not None
    assert len(result.content) > 0
    if mime_type:
        assert result.mime_type == mime_type
    assert isinstance(result.metadata, dict)
    assert isinstance(result.chunks, list)


def test_extract_bytes_sync_plain_text() -> None:
    content = b"This is plain text content."
    result = extract_bytes_sync(content, PLAIN_TEXT_MIME_TYPE)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert result.content == "This is plain text content."


def test_extract_file_sync_plain_text(tmp_path: Path) -> None:
    file_path = tmp_path / "test.txt"
    file_path.write_text("Test content")
    result = extract_file_sync(file_path)
    assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
    assert result.content == "Test content"


def test_extract_bytes_sync_invalid_mime() -> None:
    with pytest.raises(ValidationError):
        extract_bytes_sync(b"content", "application/unknown")


def test_extract_file_sync_invalid_mime(tmp_path: Path) -> None:
    file_path = tmp_path / "test.unknown"
    file_path.write_text("content")
    with pytest.raises(ValidationError):
        extract_file_sync(file_path)


def test_extract_file_sync_not_exists() -> None:
    with pytest.raises(ValidationError):
        extract_file_sync("nonexistent.txt")


@pytest.mark.anyio
async def test_batch_extract_with_different_configs() -> None:
    """Test that batch operations use the same config for all files."""
    # Set max_overlap to be less than max_chars to avoid validation error
    config = ExtractionConfig(chunk_content=True, max_chars=20, max_overlap=5)

    contents = [
        (b"First content that should be chunked", PLAIN_TEXT_MIME_TYPE),
        (b"Second content that should also be chunked", PLAIN_TEXT_MIME_TYPE),
    ]

    results = await batch_extract_bytes(contents, config=config)
    assert len(results) == 2
    # Both should have chunks due to the config
    assert len(results[0].chunks) > 0
    assert len(results[1].chunks) > 0


@pytest.mark.anyio
async def test_extract_with_quality_processing() -> None:
    """Test extraction with quality processing enabled."""
    config = ExtractionConfig(enable_quality_processing=True)

    content = b"Test content for quality processing"
    result = await extract_bytes(content, PLAIN_TEXT_MIME_TYPE, config=config)

    assert result.content == "Test content for quality processing"
    # Quality score should be added if processing was done
    if "quality_score" in result.metadata:
        assert isinstance(result.metadata["quality_score"], (int, float))


# Tests for progress callback functionality
@pytest.mark.anyio
async def test_extract_file_with_progress_callback() -> None:
    """Test extraction with progress callback."""
    progress_updates = []

    def progress_callback(current: int, total: int, message: str) -> None:
        progress_updates.append((current, total, message))

    from tempfile import NamedTemporaryFile

    with NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write("Test content for progress")
        temp_path = f.name

    try:
        # Note: progress_callback is not yet implemented in the extraction functions
        # This test is a placeholder for when it's added
        result = await extract_file(temp_path)
        assert result.content == "Test content for progress"
    finally:
        Path(temp_path).unlink(missing_ok=True)


def test_batch_extract_file_sync_mixed(test_article: Path) -> None:
    """Test synchronous batch processing of files."""
    file_paths = [str(test_article)]
    results = batch_extract_file_sync(file_paths)

    assert len(results) == 1
    assert_extraction_result(results[0], mime_type=PLAIN_TEXT_MIME_TYPE)


def test_batch_extract_bytes_sync_mixed(searchable_pdf: Path, docx_document: Path) -> None:
    contents = [
        (b"This is plain text", PLAIN_TEXT_MIME_TYPE),
        (
            docx_document.read_bytes(),
            DOCX_MIME_TYPE,
        ),
        (searchable_pdf.read_bytes(), PDF_MIME_TYPE),
    ]

    results = batch_extract_bytes_sync(contents)
    assert len(results) == len(contents)
    for i, result in enumerate(results):
        if i == 0:
            assert_extraction_result(result, mime_type=PLAIN_TEXT_MIME_TYPE)
            assert result.content.strip() == "This is plain text"
        else:
            assert_extraction_result(result, mime_type=MARKDOWN_MIME_TYPE if i == 1 else PLAIN_TEXT_MIME_TYPE)


def test_batch_extract_file_sync_with_errors(tmp_path: Path, searchable_pdf: Path) -> None:
    """Test batch file extraction with some files causing errors."""
    # Create a valid file and a non-existent file
    valid_file = tmp_path / "valid.pdf"
    valid_file.write_bytes(searchable_pdf.read_bytes())
    non_existent = tmp_path / "non_existent.pdf"

    # Create a file that will cause an error
    bad_file = tmp_path / "bad.unknown"
    bad_file.write_text("unknown format")

    file_paths = [str(valid_file), str(non_existent), str(bad_file)]

    results = batch_extract_file_sync(file_paths)

    assert len(results) == 3
    # First file should succeed
    assert len(results[0].content) > 0
    assert results[0].mime_type == PLAIN_TEXT_MIME_TYPE
    # Second file should have error
    assert "Error:" in results[1].content
    assert results[1].metadata.get("error") is True
    # Third file should have error
    assert "Error:" in results[2].content
    assert results[2].metadata.get("error") is True


def test_batch_extract_bytes_sync_with_errors(searchable_pdf: Path) -> None:
    """Test batch bytes extraction with some content causing errors."""
    pdf_content = searchable_pdf.read_bytes()

    contents = [
        (pdf_content, PDF_MIME_TYPE),
        (b"invalid content", "application/unknown"),  # This will cause an error
        (b"test text", PLAIN_TEXT_MIME_TYPE),
    ]

    results = batch_extract_bytes_sync(contents)

    assert len(results) == 3
    # First should succeed
    assert len(results[0].content) > 0
    assert results[0].mime_type == PLAIN_TEXT_MIME_TYPE
    # Second should have error
    assert "Error:" in results[1].content
    assert results[1].metadata.get("error") is True
    # Third should succeed
    assert results[2].content == "test text"
