from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING, Any
from unittest.mock import patch

import pytest

from kreuzberg._config import discover_config, load_config_from_path

if TYPE_CHECKING:
    from litestar.testing import AsyncTestClient


TEST_DATA_DIR = Path(__file__).parent.parent / "test_source_files"
IS_CI = os.environ.get("CI") == "true"


@pytest.fixture
def mounted_config_dir(tmp_path: Path) -> Path:
    app_dir = tmp_path / "app"
    app_dir.mkdir()

    config_content = """# User's production configuration
force_ocr = true
chunk_content = false
extract_tables = false
extract_entities = false
extract_keywords = false
ocr_backend = "tesseract"
auto_detect_language = false
auto_detect_document_type = false

[tesseract]
language = "eng"
psm = 4  # Single column variable sizes - user's specific setting
"""
    (app_dir / "kreuzberg.toml").write_text(config_content)

    return app_dir


@pytest.fixture
def google_doc_pdf() -> Path:
    return TEST_DATA_DIR / "google-doc-document.pdf"


@pytest.fixture
def xerox_pdf() -> Path:
    if IS_CI:
        return TEST_DATA_DIR / "scanned.pdf"
    return TEST_DATA_DIR / "Xerox_AltaLink_series_mfp_sag_en-US 2.pdf"


@pytest.fixture
def test_xls() -> Path:
    return TEST_DATA_DIR / "testXls.xls"


@pytest.mark.anyio
async def test_api_with_mounted_config(
    test_client: AsyncTestClient[Any], mounted_config_dir: Path, google_doc_pdf: Path
) -> None:
    with patch("kreuzberg._config.Path.cwd", return_value=mounted_config_dir):
        config = discover_config()
        assert config is not None, "Config should be discovered from mounted toml"
        assert config.force_ocr is True, "force_ocr should be true from config"
        assert config.ocr_backend == "tesseract", "OCR backend should be tesseract"

        with google_doc_pdf.open("rb") as f:
            pdf_content = f.read()

        response = await test_client.post(
            "/extract",
            files=[("data", (google_doc_pdf.name, pdf_content, "application/pdf"))],
        )

        assert response.status_code == 201, f"Failed: {response.status_code}: {response.text}"
        data = response.json()
        assert len(data) == 1
        assert data[0]["mime_type"] == "text/plain"
        assert "Beautiful is better" in data[0]["content"]


@pytest.mark.anyio
async def test_psm_mode_4_from_mounted_config(
    test_client: AsyncTestClient[Any], mounted_config_dir: Path, xerox_pdf: Path
) -> None:
    with patch("kreuzberg._config.Path.cwd", return_value=mounted_config_dir):
        config_path = mounted_config_dir / "kreuzberg.toml"
        config = load_config_from_path(config_path)

        assert config.ocr_config is not None, "OCR config should be present"
        if hasattr(config.ocr_config, "psm"):
            psm_value = config.ocr_config.psm
            if hasattr(psm_value, "value"):
                assert psm_value.value == 4, f"PSM should be 4, got {psm_value.value}"
            else:
                assert psm_value == 4, f"PSM should be 4, got {psm_value}"

        with xerox_pdf.open("rb") as f:
            pdf_content = f.read()

        response = await test_client.post(
            "/extract",
            files=[("data", (xerox_pdf.name, pdf_content, "application/pdf"))],
        )

        assert response.status_code == 201, f"Failed: {response.status_code}: {response.text}"
        data = response.json()
        assert len(data) == 1
        assert len(data[0]["content"]) > 1000


@pytest.mark.anyio
async def test_xls_with_mounted_config(
    test_client: AsyncTestClient[Any], mounted_config_dir: Path, test_xls: Path
) -> None:
    with patch("kreuzberg._config.Path.cwd", return_value=mounted_config_dir):
        with test_xls.open("rb") as f:
            xls_content = f.read()

        response = await test_client.post(
            "/extract",
            files=[("data", (test_xls.name, xls_content, "application/vnd.ms-excel"))],
        )

        assert response.status_code == 201, f"Failed: {response.status_code}: {response.text}"
        data = response.json()
        assert len(data) == 1
        assert data[0]["mime_type"] == "text/markdown"
        assert "Microsoft Office 365" in data[0]["content"]
        assert "workbook.xml.rels" not in data[0]["content"]


@pytest.mark.anyio
async def test_batch_with_mounted_config(
    test_client: AsyncTestClient[Any], mounted_config_dir: Path, google_doc_pdf: Path, test_xls: Path
) -> None:
    with patch("kreuzberg._config.Path.cwd", return_value=mounted_config_dir):
        files = []

        with google_doc_pdf.open("rb") as f:
            files.append(("data", (google_doc_pdf.name, f.read(), "application/pdf")))

        with test_xls.open("rb") as f:
            files.append(("data", (test_xls.name, f.read(), "application/vnd.ms-excel")))

        response = await test_client.post(
            "/extract",
            files=files,
        )

        assert response.status_code == 201, f"Failed: {response.status_code}: {response.text}"
        data = response.json()
        assert len(data) == 2

        assert data[0]["mime_type"] == "text/plain"

        assert data[1]["mime_type"] == "text/markdown"
        assert "workbook.xml.rels" not in data[1]["content"]


def test_config_loading_from_mounted_path(mounted_config_dir: Path) -> None:
    config_path = mounted_config_dir / "kreuzberg.toml"
    config = load_config_from_path(config_path)

    assert config.force_ocr is True
    assert config.chunk_content is False
    assert config.extract_tables is False
    assert config.ocr_backend == "tesseract"
    assert config.auto_detect_language is False

    assert config.ocr_config is not None
    if hasattr(config.ocr_config, "language"):
        assert config.ocr_config.language == "eng"
    if hasattr(config.ocr_config, "psm"):
        psm_value = config.ocr_config.psm
        if hasattr(psm_value, "value"):
            assert psm_value.value == 4
        else:
            assert psm_value == 4
