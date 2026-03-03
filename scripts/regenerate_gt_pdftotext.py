#!/usr/bin/env python3
"""Regenerate ground truth for PDF fixtures using pdftotext (poppler).

Uses pdftotext -layout as an independent baseline for all PDFs in the
PDFIUM_GROUND_TRUTH list from the regression test.
"""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEST_DOCS = ROOT / "test_documents"
GT_DIR = TEST_DOCS / "ground_truth" / "pdf"

# All PDFs from PDFIUM_GROUND_TRUTH in the regression test
DOCUMENTS = [
    # Docling vendored
    "2203.01017v2",
    "2206.01062",
    "2305.03393v1",
    "2305.03393v1-pg9",
    "amt_handbook_sample",
    "code_and_formula",
    "multi_page",
    "picture_classification",
    "redp5110_sampled",
    "right_to_left_01",
    "right_to_left_02",
    "right_to_left_03",
    # pdfplumber vendored
    "2023-06-20-PV",
    "annotations",
    "annotations-rotated-180",
    "annotations-rotated-270",
    "annotations-rotated-90",
    "annotations-unicode-issues",
    "chelsea_pdta",
    "cupertino_usd_4-6-16",
    "extra-attrs-example",
    "federal-register-2020-17221",
    "figure_structure",
    "hello_structure",
    "image_structure",
    "issue-1054-example",
    "issue-1114-dedupe-chars",
    "issue-1147-example",
    "issue-1181",
    "issue-1279-example",
    "issue-140-example",
    "issue-192-example",
    "issue-316-example",
    "issue-33-lorem-ipsum",
    "issue-336-example",
    "issue-461-example",
    "issue-463-example",
    "issue-466-example",
    "issue-53-example",
    "issue-598-example",
    "issue-67-example",
    "issue-71-duplicate-chars",
    "issue-71-duplicate-chars-2",
    "issue-842-example",
    "issue-848",
    "issue-90-example",
    "issue-905",
    "issue-912",
    "issue-982-example",
    "issue-987-test",
    "la-precinct-bulletin-2014-p1",
    "line-char-render-example",
    "malformed-from-issue-932",
    "mcid_example",
    "nics-background-checks-2015-11",
    "nics-background-checks-2015-11-rotated",
    "page-boxes-example",
    "pdf_structure",
    "pdffill-demo",
    "pr-136-example",
    "pr-138-example",
    "pr-88-example",
    "scotus-transcript-p1",
    "senate-expenditures",
    "table-curves-example",
    "test-punkt",
    "WARN-Report-for-7-1-2015-to-03-25-2016",
    "word365_structure",
    # markitdown vendored
    "masterformat_partial_numbering",
    "RECEIPT-2024-TXN-98765_retail_purchase",
    "REPAIR-2022-INV-001_multipage",
    "SPARSE-2024-INV-1234_borderless_table",
    "test",
]


def find_pdf(name: str) -> Path | None:
    """Search for a PDF in known locations."""
    candidates = [
        TEST_DOCS / "pdf" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "docling" / "pdf" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "pdfplumber" / "pdf" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "pdfplumber" / "pdf" / "from-oss-fuzz" / "load" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "markitdown" / "pdf" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "markitdown" / f"{name}.pdf",
        TEST_DOCS / "vendored" / "pdfium-render" / f"{name}.pdf",
    ]
    for p in candidates:
        if p.exists():
            return p
    return None


def run_pdftotext(pdf_path: Path, output_path: Path) -> bool:
    """Run pdftotext -layout on a PDF."""
    try:
        result = subprocess.run(
            ["/opt/homebrew/bin/pdftotext", "-layout", str(pdf_path), str(output_path)],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return result.returncode == 0
    except subprocess.TimeoutExpired:
        return False


def main():
    regenerated = 0
    skipped = 0
    failed = 0

    print("Regenerating ground truth using pdftotext -layout")
    print("=" * 70)

    for name in DOCUMENTS:
        pdf_path = find_pdf(name)
        if pdf_path is None:
            print(f"  SKIP {name}: PDF not found")
            skipped += 1
            continue

        gt_path = GT_DIR / f"{name}.txt"

        if run_pdftotext(pdf_path, gt_path):
            # Check output
            content = gt_path.read_text(errors="replace").strip()
            word_count = len(content.split())
            print(f"  OK   {name}: {word_count} words")
            regenerated += 1
        else:
            print(f"  FAIL {name}: pdftotext failed")
            failed += 1

    print("=" * 70)
    print(f"Done: {regenerated} regenerated, {skipped} skipped, {failed} failed")


if __name__ == "__main__":
    main()
