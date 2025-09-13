from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING, Any

import pytest

if TYPE_CHECKING:
    from litestar.testing import AsyncTestClient


TEST_DATA_DIR = Path(__file__).parent.parent / "test_source_files"
IS_CI = os.environ.get("CI") == "true"


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
async def test_large_pdf_upload(test_client: AsyncTestClient[Any], xerox_pdf: Path) -> None:
    with xerox_pdf.open("rb") as f:
        pdf_content = f.read()

    if IS_CI:
        assert len(pdf_content) > 50_000, f"File too small: {len(pdf_content)} bytes"
    else:
        assert len(pdf_content) > 2_000_000, f"File too small: {len(pdf_content)} bytes"

    response = await test_client.post(
        "/extract?force_ocr=true&ocr_backend=tesseract",
        files=[("data", (xerox_pdf.name, pdf_content, "application/pdf"))],
    )

    assert response.status_code == 201, f"Failed with status {response.status_code}: {response.text}"
    data = response.json()
    assert len(data) == 1
    assert data[0]["mime_type"] == "text/plain"
    assert len(data[0]["content"]) > 100
    if not IS_CI:
        assert "Xerox" in data[0]["content"]


@pytest.mark.anyio
async def test_xls_upload_with_correct_mime(test_client: AsyncTestClient[Any], test_xls: Path) -> None:
    with test_xls.open("rb") as f:
        xls_content = f.read()

    response = await test_client.post(
        "/extract",
        files=[("data", (test_xls.name, xls_content, "application/vnd.ms-excel"))],
    )

    assert response.status_code == 201, f"Failed with status {response.status_code}: {response.text}"
    data = response.json()
    assert len(data) == 1
    assert data[0]["mime_type"] == "text/markdown"
    assert "Microsoft Office 365" in data[0]["content"]

    assert "workbook.xml.rels" not in data[0].get("content", "")
    assert "ParsingError" not in data[0].get("content", "")


@pytest.mark.anyio
async def test_batch_upload_mixed_large_files(
    test_client: AsyncTestClient[Any], google_doc_pdf: Path, xerox_pdf: Path, test_xls: Path
) -> None:
    files = []

    with google_doc_pdf.open("rb") as f:
        files.append(("data", (google_doc_pdf.name, f.read(), "application/pdf")))

    with xerox_pdf.open("rb") as f:
        files.append(("data", (xerox_pdf.name, f.read(), "application/pdf")))

    with test_xls.open("rb") as f:
        files.append(("data", (test_xls.name, f.read(), "application/vnd.ms-excel")))

    response = await test_client.post(
        "/extract?force_ocr=false",
        files=files,
    )

    assert response.status_code == 201, f"Failed with status {response.status_code}: {response.text}"
    data = response.json()
    assert len(data) == 3

    assert data[0]["mime_type"] == "text/plain"
    assert "Beautiful is better than ugly" in data[0]["content"] or "Example" in data[0]["content"]

    assert data[1]["mime_type"] == "text/plain"
    assert len(data[1]["content"]) > 1000

    assert data[2]["mime_type"] == "text/markdown"
    assert "Microsoft Office 365" in data[2]["content"]


@pytest.mark.anyio
async def test_api_with_psm_config(test_client: AsyncTestClient[Any], google_doc_pdf: Path) -> None:
    with google_doc_pdf.open("rb") as f:
        pdf_content = f.read()

    response = await test_client.post(
        "/extract?force_ocr=true&ocr_backend=tesseract",
        files=[("data", (google_doc_pdf.name, pdf_content, "application/pdf"))],
    )

    assert response.status_code == 201, f"Failed with status {response.status_code}: {response.text}"
    data = response.json()
    assert len(data) == 1
    assert data[0]["mime_type"] == "text/plain"
    assert "Beautiful is better than ugly" in data[0]["content"] or "Example" in data[0]["content"]


@pytest.mark.anyio
async def test_api_handles_large_batch(test_client: AsyncTestClient[Any], google_doc_pdf: Path) -> None:
    with google_doc_pdf.open("rb") as f:
        pdf_content = f.read()

    files = [("data", (f"doc_{i}.pdf", pdf_content, "application/pdf")) for i in range(5)]

    response = await test_client.post(
        "/extract",
        files=files,
    )

    assert response.status_code == 201, f"Failed with status {response.status_code}: {response.text}"
    data = response.json()
    assert len(data) == 5

    for i, result in enumerate(data):
        assert result["mime_type"] == "text/plain", f"Result {i} has wrong mime_type"
        assert len(result["content"]) > 100, f"Result {i} has insufficient content"


@pytest.mark.anyio
async def test_api_error_handling_with_corrupted_data(test_client: AsyncTestClient[Any]) -> None:
    corrupted_pdf = b"%PDF-1.4\nThis is not a valid PDF content"

    response = await test_client.post(
        "/extract",
        files=[("data", ("corrupted.pdf", corrupted_pdf, "application/pdf"))],
    )

    assert response.status_code in [201, 422], f"Unexpected status {response.status_code}: {response.text}"

    if response.status_code == 201:
        data = response.json()
        assert isinstance(data, list)


@pytest.mark.anyio
async def test_memory_efficient_streaming(test_client: AsyncTestClient[Any], xerox_pdf: Path) -> None:
    with xerox_pdf.open("rb") as f:
        pdf_content = f.read()

    for i in range(2):
        response = await test_client.post(
            "/extract?force_ocr=true&ocr_backend=tesseract",
            files=[("data", (f"large_{i}.pdf", pdf_content, "application/pdf"))],
        )

        assert response.status_code == 201, f"Failed on iteration {i}: {response.status_code}: {response.text}"
        data = response.json()
        assert len(data) == 1, f"Expected 1 result, got {len(data)}"
        assert len(data[0]["content"]) > 1000, f"Insufficient content on iteration {i}"
        assert data[0]["mime_type"] == "text/plain"
