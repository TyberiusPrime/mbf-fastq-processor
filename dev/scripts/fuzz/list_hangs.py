#!/usr/bin/env python3
"""Classify AFL hang inputs for the fastq parser by file format.

Walks `dev/fuzz/fastq_parser/output/*/hangs/id:*` and groups each input by
the leading bytes:

  - fastq:  starts with '>'
  - gzip:   starts with the gzip magic (0x1f 0x8b)
  - other:  anything else (including empty files)

Prints a summary count to stderr and one `<class>\t<path>` line per file to
stdout, suitable for piping into `sort` / `awk` / `xargs`.
"""

import sys
from collections import Counter
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
OUTPUT_DIR = REPO_ROOT / "dev" / "fuzz" / "fastq_parser" / "output"

GZIP_MAGIC = b"\x1f\x8b"
FASTQ_MARKER = b">"


def classify(path: Path) -> str:
    head = path.read_bytes()[:2]
    if head.startswith(GZIP_MAGIC):
        return "gzip"
    if head.startswith(FASTQ_MARKER):
        return "fastq"
    return "other"


def main() -> int:
    if not OUTPUT_DIR.exists():
        print(f"error: no fuzzing output at {OUTPUT_DIR}", file=sys.stderr)
        return 2

    counts: Counter[str] = Counter()
    for hang in sorted(OUTPUT_DIR.glob("*/hangs/id:*")):
        cls = classify(hang)
        counts[cls] += 1
        print(f"{cls}\t{hang}")

    total = sum(counts.values())
    summary = ", ".join(f"{counts[k]} {k}" for k in ("fastq", "gzip", "other"))
    print(f"# {total} hangs: {summary}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
