#!/usr/bin/env python3
"""Print AFL crash inputs for the fastq parser, excluding canary trips.

Walks `dev/fuzz/fastq_parser/output/*/crashes/id:*` and filters out any file
that contains the `AFL_POSITIVE_CONTROL` marker — those are the positive
control panics from the `afl-positive-control` feature, not real parser bugs.

Output is one path per line, suitable for piping into xargs.
"""

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
OUTPUT_DIR = REPO_ROOT / "dev" / "fuzz" / "fastq_parser" / "output"
CANARY_MARKER = b"AFL_POSITIVE_CONTROL"


def main() -> int:
    if not OUTPUT_DIR.exists():
        print(f"error: no fuzzing output at {OUTPUT_DIR}", file=sys.stderr)
        return 2

    real = 0
    canary = 0
    for crash in sorted(OUTPUT_DIR.glob("*/crashes/id:*")):
        if CANARY_MARKER in crash.read_bytes():
            canary += 1
            continue
        print(crash)
        real += 1

    print(f"# {real} real crashes, {canary} canary trips filtered", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
