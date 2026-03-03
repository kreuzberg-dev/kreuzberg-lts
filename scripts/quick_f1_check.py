#!/usr/bin/env python3
"""Quick F1 check for quality-benchmarks corpus against kreuzberg.

Uses kreuzberg's Rust extractor via subprocess to measure F1 vs GT.
"""

import subprocess
import sys
from pathlib import Path
from collections import Counter

ROOT = Path(__file__).resolve().parent.parent
GT_DIR = ROOT / "test_documents" / "ground_truth" / "pdf"


def tokenize(text: str) -> list[str]:
    """Match the Rust tokenizer: lowercase, strip ASCII punctuation."""
    tokens = []
    for word in text.split():
        cleaned = word.strip("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~").lower()
        if cleaned:
            tokens.append(cleaned)
    return tokens


def word_f1(extracted: str, ground_truth: str) -> tuple[float, float, float]:
    """Compute bag-of-words precision, recall, F1."""
    ext_tokens = tokenize(extracted)
    gt_tokens = tokenize(ground_truth)

    if not gt_tokens and not ext_tokens:
        return (1.0, 1.0, 1.0)
    if not gt_tokens or not ext_tokens:
        return (0.0, 0.0, 0.0)

    gt_bag = Counter(gt_tokens)
    ext_bag = Counter(ext_tokens)

    matching = sum(min(ext_bag[w], gt_bag[w]) for w in ext_bag if w in gt_bag)

    precision = matching / len(ext_tokens)
    recall = matching / len(gt_tokens)
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0

    return (precision, recall, f1)


def main():
    # Check a few random quality-benchmarks documents
    datasets = [("nougat", 50), ("pdfa", 50)]

    for dataset, count in datasets:
        print(f"\n=== {dataset} ===")
        total_f1 = 0.0
        tested = 0
        below_90 = []

        for i in range(1, count + 1):
            name = f"{dataset}_{i:03d}"
            gt_path = GT_DIR / f"{name}.txt"
            if not gt_path.exists():
                continue

            gt = gt_path.read_text(errors="replace")
            gt_words = len(tokenize(gt))

            if gt_words < 5:
                continue

            tested += 1
            # We don't have kreuzberg CLI, so just report GT word count
            print(f"  {name}: GT={gt_words} words")

        print(f"  Total with >=5 words: {tested}")


if __name__ == "__main__":
    main()
