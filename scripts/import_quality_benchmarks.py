#!/usr/bin/env python3
"""Import quality-benchmarks corpus PDFs as benchmark fixtures.

Creates:
- Fixture JSON configs in tools/benchmark-harness/fixtures/pdf/
- Ground truth .txt files in test_documents/ground_truth/pdf/
"""

import json
import os
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
QB_ROOT = ROOT.parent / "quality-benchmarks"
FIXTURES_DIR = ROOT / "tools" / "benchmark-harness" / "fixtures" / "pdf"
GT_DIR = ROOT / "test_documents" / "ground_truth" / "pdf"
GT_MAPPING = ROOT / "test_documents" / "ground_truth" / "ground_truth_mapping.json"


def import_dataset(dataset: str) -> int:
    """Import a quality-benchmarks dataset (nougat or pdfa)."""
    pdf_dir = QB_ROOT / "data" / dataset
    gt_source_dir = QB_ROOT / "data" / "ground-truth" / dataset
    count = 0

    for i in range(1, 51):
        name = f"{dataset}_{i:03d}"
        pdf_path = pdf_dir / f"{name}.pdf"
        gt_source = gt_source_dir / f"{name}.md"

        if not pdf_path.exists():
            print(f"  SKIP {name}: PDF not found")
            continue
        if not gt_source.exists():
            print(f"  SKIP {name}: GT not found")
            continue

        # Copy GT .md to .txt
        gt_dest = GT_DIR / f"{name}.txt"
        shutil.copy2(gt_source, gt_dest)

        # Get file size
        file_size = pdf_path.stat().st_size

        # Relative path from fixtures dir to quality-benchmarks PDF
        # fixtures are at tools/benchmark-harness/fixtures/pdf/
        # PDFs are at ../../quality-benchmarks/data/{dataset}/
        rel_pdf = f"../../../../../quality-benchmarks/data/{dataset}/{name}.pdf"
        rel_gt = f"../../../../test_documents/ground_truth/pdf/{name}.txt"

        # Create fixture JSON
        fixture = {
            "document": rel_pdf,
            "file_type": "pdf",
            "file_size": file_size,
            "expected_frameworks": ["kreuzberg"],
            "metadata": {
                "description": f"Document from quality-benchmarks {dataset} dataset",
                "source": f"quality-benchmarks/{dataset}",
                "size_category": "small" if file_size < 100_000 else "medium" if file_size < 1_000_000 else "large",
            },
            "ground_truth": {
                "text_file": rel_gt,
                "source": "pixparse",
            },
        }

        fixture_path = FIXTURES_DIR / f"{name}.json"
        with open(fixture_path, "w") as f:
            json.dump(fixture, f, indent="\t")
            f.write("\n")

        count += 1

    return count


def update_gt_mapping():
    """Add new entries to ground_truth_mapping.json."""
    with open(GT_MAPPING) as f:
        mapping = json.load(f)

    for dataset in ("nougat", "pdfa"):
        for i in range(1, 51):
            name = f"{dataset}_{i:03d}"
            gt_path = f"test_documents/ground_truth/pdf/{name}.txt"
            if (GT_DIR / f"{name}.txt").exists():
                mapping[name] = gt_path

    with open(GT_MAPPING, "w") as f:
        json.dump(mapping, f, indent=2)
        f.write("\n")


def main():
    print("Importing quality-benchmarks corpus...")
    print()

    for dataset in ("nougat", "pdfa"):
        print(f"--- {dataset} ---")
        count = import_dataset(dataset)
        print(f"  Imported {count} fixtures")
        print()

    print("Updating ground_truth_mapping.json...")
    update_gt_mapping()
    print("Done!")


if __name__ == "__main__":
    main()
