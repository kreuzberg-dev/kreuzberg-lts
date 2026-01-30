# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "markitdown>=0.1.4",
# ]
# ///
"""MarkItDown extraction wrapper for benchmark harness."""

from __future__ import annotations

import json
import sys
import time

from markitdown import MarkItDown


def extract_sync(file_path: str) -> dict:
    """Extract using MarkItDown."""
    start = time.perf_counter()
    md = MarkItDown()
    result = md.convert(file_path)
    duration_ms = (time.perf_counter() - start) * 1000.0

    return {
        "content": result.text_content or "",
        "metadata": {"framework": "markitdown"},
        "_extraction_time_ms": duration_ms,
    }


def run_server() -> None:
    """Persistent server mode: read paths from stdin, write JSON to stdout."""
    for line in sys.stdin:
        file_path = line.strip()
        if not file_path:
            continue
        try:
            payload = extract_sync(file_path)
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
        print("Usage: markitdown_extract.py [--ocr|--no-ocr] <mode> <file_path>", file=sys.stderr)
        print("Modes: sync, server", file=sys.stderr)
        sys.exit(1)

    mode = args[0]
    if mode == "server":
        run_server()
    elif mode == "sync":
        if len(args) < 2:
            print("Error: sync mode requires a file path", file=sys.stderr)
            sys.exit(1)
        file_path = args[1]
        try:
            payload = extract_sync(file_path)
            print(json.dumps(payload), end="")
        except Exception as e:
            print(f"Error extracting with MarkItDown: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        # Legacy fallback for direct file path
        try:
            payload = extract_sync(args[0])
            print(json.dumps(payload), end="")
        except Exception as e:
            print(f"Error extracting with MarkItDown: {e}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
