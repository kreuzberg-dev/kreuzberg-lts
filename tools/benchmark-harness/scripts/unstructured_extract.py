# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "unstructured>=0.18.21",
# ]
# ///
"""Unstructured extraction wrapper for benchmark harness."""

from __future__ import annotations

import json
import sys
import time

from unstructured.partition.auto import partition


def extract_sync(file_path: str, ocr_enabled: bool) -> dict:
    """Extract using Unstructured partition API."""
    strategy = "hi_res" if ocr_enabled else "fast"
    start = time.perf_counter()
    elements = partition(filename=file_path, strategy=strategy)
    duration_ms = (time.perf_counter() - start) * 1000.0

    text = "\n\n".join(str(el) for el in elements)
    return {
        "content": text,
        "metadata": {"framework": "unstructured", "strategy": strategy},
        "_extraction_time_ms": duration_ms,
    }


def run_server(ocr_enabled: bool) -> None:
    """Persistent server mode: read paths from stdin, write JSON to stdout."""
    for line in sys.stdin:
        file_path = line.strip()
        if not file_path:
            continue
        try:
            payload = extract_sync(file_path, ocr_enabled)
            print(json.dumps(payload), flush=True)
        except Exception as e:
            print(json.dumps({"error": str(e), "_extraction_time_ms": 0}), flush=True)


def main() -> None:
    ocr_enabled = False
    args = []
    for arg in sys.argv[1:]:
        if arg == "--ocr":
            ocr_enabled = True
        elif arg == "--no-ocr":
            ocr_enabled = False
        else:
            args.append(arg)

    if len(args) < 1:
        print("Usage: unstructured_extract.py [--ocr|--no-ocr] <mode> <file_path>", file=sys.stderr)
        print("Modes: sync, server", file=sys.stderr)
        sys.exit(1)

    mode = args[0]

    if mode == "server":
        run_server(ocr_enabled)
    elif mode == "sync":
        if len(args) < 2:
            print("Error: sync mode requires a file path", file=sys.stderr)
            sys.exit(1)
        try:
            payload = extract_sync(args[1], ocr_enabled)
            print(json.dumps(payload), end="")
        except Exception as e:
            print(f"Error extracting with Unstructured: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        # Legacy mode: first arg is the file path directly
        try:
            payload = extract_sync(args[0], ocr_enabled)
            print(json.dumps(payload), end="")
        except Exception as e:
            print(f"Error extracting with Unstructured: {e}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
