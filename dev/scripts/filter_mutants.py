#!/usr/bin/env python3
"""Filter cargo-mutants' missed.txt against //mutants::skip comments.

cargo-mutants doesn't understand our informal `//mutants::skip` line-comment
convention (only the `#[mutants::skip]` attribute), so mutants next to one of
these comments still show up in missed.txt. This cross-references the two and
reports what's genuinely missed vs. what's already annotated, plus any
`//mutants::skip` comments that no longer cover a missed mutant.
"""

import re
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
MISSED_TXT = PROJECT_ROOT / "mutants.out" / "missed.txt"

SKIP_PATTERN = r"//\s*mutants::skip"
SKIP_RE = re.compile(SKIP_PATTERN)
SKIP_STARTS_RE = re.compile(r"^" + SKIP_PATTERN)
MISSED_LINE_RE = re.compile(r"^(?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+): (?P<desc>.+)$")


def build_skip_lookup():
    """rg for //mutants::skip comments, return {(file, line): raw_line}."""
    result = subprocess.run(
        ["rg", "-n", "--no-heading", SKIP_PATTERN, "-g", "*.rs"],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode not in (0, 1):  # 1 == no matches, still fine
        print(result.stderr, file=sys.stderr)
        sys.exit(result.returncode)

    lookup = {}
    for line in result.stdout.splitlines():
        file, lineno, content = line.split(":", 2)
        lookup[(file, int(lineno))] = content
    return lookup


def main():
    if not MISSED_TXT.exists():
        print(f"Not found: {MISSED_TXT}", file=sys.stderr)
        sys.exit(1)

    lookup = build_skip_lookup()
    used = set()

    remaining = []
    for raw in MISSED_TXT.read_text().splitlines():
        m = MISSED_LINE_RE.match(raw)
        if not m:
            continue
        file = m.group("file")
        line = int(m.group("line"))

        if (file, line) in lookup:
            used.add((file, line))
            continue

        next_line = lookup.get((file, line + 1))
        if next_line is not None and SKIP_STARTS_RE.match(next_line.strip()):
            used.add((file, line + 1))
            continue

        remaining.append(raw)

    superfluous = sorted(
        f"{file}:{line}:{lookup[(file, line)].strip()}"
        for (file, line) in lookup
        if (file, line) not in used
    )

    print(f"=== Remaining missed mutants ({len(remaining)}) ===")
    for line in remaining:
        print(line)

    print(f"\n=== Superfluous //mutants::skip ({len(superfluous)}) ===")
    for line in superfluous:
        print(line)


if __name__ == "__main__":
    main()
