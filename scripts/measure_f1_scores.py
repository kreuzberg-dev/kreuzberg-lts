#!/usr/bin/env python3
"""Quick F1 score measurement using pdftotext as baseline.

Extracts text from PDFs using kreuzberg (via cargo test runner) and compares
against pdftotext GT. This just prints the F1 scores for threshold calibration.
"""

import re
import sys


def parse_test_output(output: str):
    """Parse the test output to extract F1 scores."""
    pattern = r"^(\S+)\s+[\d.]+%\s+[\d.]+%\s+([\d.]+)%\s+[\d.]+%\s+(PASS|FAIL)"
    results = []
    for line in output.split("\n"):
        m = re.match(pattern, line.strip())
        if m:
            name = m.group(1)
            f1 = float(m.group(2)) / 100.0
            results.append((name, f1))
    return results


def suggest_thresholds(results):
    """Suggest thresholds at ~7% below measured F1."""
    print(f"\n{'Name':<55} {'Measured':>8} {'Suggested':>10}")
    print("-" * 75)
    for name, f1 in sorted(results, key=lambda x: x[0]):
        threshold = max(0.0, round(f1 - 0.07, 2))
        print(f"{name:<55} {f1:>7.2f}   {threshold:>9.2f}")


if __name__ == "__main__":
    # Read from stdin or file
    if len(sys.argv) > 1:
        with open(sys.argv[1]) as f:
            data = f.read()
    else:
        data = sys.stdin.read()

    results = parse_test_output(data)
    suggest_thresholds(results)
