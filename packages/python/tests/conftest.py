"""Shared pytest fixtures for binding-specific tests."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

import pytest

if TYPE_CHECKING:
    from collections.abc import Generator

    from kreuzberg import ExtractionResult


@pytest.fixture
def docx_document() -> Path:
    """Path to DOCX test file used across binding-specific suites."""
    path = Path(__file__).parent.parent.parent.parent / "test_documents" / "docx" / "lorem_ipsum.docx"
    if not path.exists():
        pytest.skip(f"Test file not found: {path}")
    return path


@pytest.fixture(scope="session")
def test_documents() -> Path:
    """Path to test_documents directory containing PDF and other test files."""
    path = Path(__file__).parent.parent.parent.parent / "test_documents"
    if not path.exists():
        pytest.skip(f"Test documents directory not found: {path}")
    return path


_pdf_extraction_cache: dict[str, ExtractionResult | None] = {}
_pdfium_initialized: bool = False


def get_cached_pdf_extraction(pdf_path: str, config: Any) -> ExtractionResult | None:
    """Get or create a cached PDF extraction result.

    Since PDFium can only be initialized ONCE per process and subsequent
    extract_file_sync calls fail with "already initialized" errors,
    this function maintains a global cache. Once PDFium is initialized,
    all tests reuse the first successful extraction result regardless
    of the requested PDF path.
    """
    from kreuzberg import extract_file_sync

    global _pdfium_initialized

    if _pdfium_initialized:
        for result in _pdf_extraction_cache.values():
            if result is not None:
                return result
        return None

    if pdf_path not in _pdf_extraction_cache:
        try:
            result = extract_file_sync(pdf_path, config=config)
            _pdf_extraction_cache[pdf_path] = result
            _pdfium_initialized = True
            return result
        except Exception as exc:
            if "PdfiumLibraryBindingsAlreadyInitialized" in str(exc):
                _pdfium_initialized = True
                _pdf_extraction_cache[pdf_path] = None
                for result in _pdf_extraction_cache.values():
                    if result is not None:
                        return result
                return None
            raise

    return _pdf_extraction_cache.get(pdf_path)


@pytest.fixture(scope="session", autouse=True)
def _pdfium_session_management() -> Generator[None, None, None]:
    """Manage PDFium initialization state for the session.

    PDFium is a C++ library that can only be initialized once per process.
    This fixture provides utilities for managing PDF extractions across the test suite.
    """
    global _pdfium_initialized

    yield

    _pdf_extraction_cache.clear()
    _pdfium_initialized = False
