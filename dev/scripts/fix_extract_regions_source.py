#!/usr/bin/env python3
"""Migrate `ExtractRegions` steps from per-region `source`/`segment` to a step-level one.

`ExtractRegions` used to accept a `source`/`segment` key on every region dict.
It now takes a single `source`/`segment` at the `[[step]]` level instead.

For every `ExtractRegions` step this script:
  * If all of the step's regions reference the same source/segment, the key is
    removed from each region dict and hoisted to the `[[step]]` level (right
    after the `action` line).
  * If they differ (true cross-segment extraction), the step is left untouched
    and a warning is printed naming the file.

The `source` vs `segment` label and the file's general formatting are preserved;
only the per-region key is removed and a single step-level line is added.

Usage:
    dev/scripts/fix_extract_regions_source.py [PATH ...] [--dry-run]

PATH may be a file or a directory (searched recursively for *.toml). With no
PATH, the current directory is scanned recursively.
"""

import argparse
import re
import sys
from pathlib import Path

# A `[[step]]` table header (not commented out).
STEP_HEADER_RE = re.compile(r"^[ \t]*\[\[[ \t]*step[ \t]*\]\]")
# `action = 'ExtractRegions'` — plural only, never ExtractRegion / ...OfLowQuality.
ACTION_RE = re.compile(r"^([ \t]*)action[ \t]*=[ \t]*['\"]ExtractRegions['\"]")
ACTION_SEARCH_RE = re.compile(r"action[ \t]*=[ \t]*['\"]ExtractRegions['\"]")
# A region inline table (region dicts never nest braces).
INLINE_TABLE_RE = re.compile(r"\{[^{}]*\}")
# The source/segment key=value inside a region dict (value is always a quoted string).
SOURCE_KV_RE = re.compile(r"(segment|source)[ \t]*=[ \t]*('[^']*'|\"[^\"]*\")")
# Removal variants — use [ \t] (never \s) so we never swallow newlines / merge lines.
_VAL = r"(?:'[^']*'|\"[^\"]*\")"
REMOVE_KEY_COMMA_RE = re.compile(r"[ \t]*(?:segment|source)[ \t]*=[ \t]*" + _VAL + r"[ \t]*,")
REMOVE_COMMA_KEY_RE = re.compile(r",[ \t]*(?:segment|source)[ \t]*=[ \t]*" + _VAL + r"[ \t]*")
REMOVE_KEY_ONLY_RE = re.compile(r"[ \t]*(?:segment|source)[ \t]*=[ \t]*" + _VAL + r"[ \t]*")
# A region source that sits on its own line (multi-line region dict) — drop the whole line.
STANDALONE_SOURCE_LINE_RE = re.compile(
    r"(?m)^[ \t]*(?:segment|source)[ \t]*=[ \t]*" + _VAL + r"[ \t]*,?[ \t]*\r?\n"
)


def is_table_header(line: str) -> bool:
    """True for any (non-comment) TOML table header line, e.g. [output] or [[step]]."""
    return line.lstrip().startswith("[")


def remove_source_kv(table: str) -> str:
    """Remove the source/segment key (and one adjacent comma) from one region dict."""
    new, n = REMOVE_KEY_COMMA_RE.subn("", table, count=1)
    if n:
        return new
    new, n = REMOVE_COMMA_KEY_RE.subn("", table, count=1)
    if n:
        return new
    return REMOVE_KEY_ONLY_RE.sub("", table, count=1)


def has_step_level_source(block: str) -> bool:
    """True if a source/segment key already exists outside the region dicts."""
    stripped = INLINE_TABLE_RE.sub("", block)
    return re.search(r"(?m)^[ \t]*(segment|source)[ \t]*=", stripped) is not None


REGIONS_OPEN_RE = re.compile(r"regions[ \t]*=[ \t]*\[")


def insert_step_source(block: str, label: str, raw_value: str) -> str:
    """Insert `<label> = <raw_value>` right after the `regions` array, matching the
    `action` line's indent (this is where the project's existing configs put it)."""
    lines = block.splitlines(keepends=True)

    indent = ""
    for line in lines:
        m = ACTION_RE.match(line)
        if m:
            indent = m.group(1)
            break

    start = next((i for i, l in enumerate(lines) if REGIONS_OPEN_RE.search(l)), None)
    if start is None:  # no array found (shouldn't happen) — fall back to after action
        for i, line in enumerate(lines):
            if ACTION_RE.match(line):
                lines.insert(i + 1, f"{indent}{label} = {raw_value}\n")
                break
        return "".join(lines)

    close = next((i for i in range(start, len(lines)) if "]" in lines[i]), start)
    if not lines[close].endswith("\n"):
        lines[close] += "\n"
    lines.insert(close + 1, f"{indent}{label} = {raw_value}\n")
    return "".join(lines)


def process_block(block: str, rel: str, warnings: list[str]) -> tuple[str, bool]:
    """Migrate a single [[step]] block. Returns (new_block, changed)."""
    if not ACTION_SEARCH_RE.search(block):
        return block, False

    tables = INLINE_TABLE_RE.findall(block)
    if not tables:
        return block, False  # nothing to inspect (e.g. empty regions)

    found = [SOURCE_KV_RE.search(t) for t in tables]
    present = [m for m in found if m]

    if not present:
        return block, False  # already migrated (no per-region source)

    if len(present) != len(found):
        warnings.append(
            f"{rel}: ExtractRegions step has some regions with and some without "
            f"a source/segment key — left unchanged."
        )
        return block, False

    if has_step_level_source(block):
        warnings.append(
            f"{rel}: ExtractRegions step already has a step-level source/segment "
            f"as well as per-region ones — left unchanged."
        )
        return block, False

    values = {m.group(2)[1:-1] for m in present}
    if len(values) != 1:
        warnings.append(
            f"{rel}: ExtractRegions regions reference differing sources "
            f"{sorted(values)} — cannot hoist, left unchanged."
        )
        return block, False

    label = present[0].group(1)
    raw_value = present[0].group(2)
    # Sources on their own line (multi-line region dicts) drop the whole line;
    # sources sharing a line with other keys are stripped in place.
    new_block = STANDALONE_SOURCE_LINE_RE.sub("", block)
    new_block = INLINE_TABLE_RE.sub(lambda m: remove_source_kv(m.group(0)), new_block)
    new_block = insert_step_source(new_block, label, raw_value)
    return new_block, True


def process_file(path: Path, root: Path, dry_run: bool, warnings: list[str]) -> bool:
    """Migrate all ExtractRegions steps in one file. Returns True if it changed."""
    rel = str(path.relative_to(root)) if path.is_relative_to(root) else str(path)
    try:
        lines = path.read_text().splitlines(keepends=True)
    except (OSError, UnicodeDecodeError) as e:
        print(f"skipped (unreadable): {rel} ({e})", file=sys.stderr)
        return False

    out: list[str] = []
    changed = False
    i, n = 0, len(lines)
    while i < n:
        if STEP_HEADER_RE.match(lines[i]):
            j = i + 1
            while j < n and not is_table_header(lines[j]):
                j += 1
            new_block, block_changed = process_block("".join(lines[i:j]), rel, warnings)
            out.append(new_block)
            changed |= block_changed
            i = j
        else:
            out.append(lines[i])
            i += 1

    if changed and not dry_run:
        path.write_text("".join(out))
    if changed:
        print(f"{'would fix' if dry_run else 'fixed'}: {rel}")
    return changed


SKIP_DIRS = {".git", ".jj", "target", "__pycache__", ".venv", "node_modules"}


def _skip(part: str) -> bool:
    # Skip VCS/build dirs and the gitignored, regenerated test-run output dirs
    # (actual, actual_2, actual_docker, ...).
    return part in SKIP_DIRS or part == "actual" or part.startswith("actual_")


def iter_toml(paths: list[Path]):
    for p in paths:
        if p.is_dir():
            for f in sorted(p.rglob("*.toml")):
                if not any(_skip(part) for part in f.parts):
                    yield f
        elif p.suffix == ".toml":
            yield p


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="files or dirs (default: cwd)")
    parser.add_argument("--dry-run", action="store_true", help="report changes, write nothing")
    args = parser.parse_args()

    root = Path.cwd()
    targets = args.paths or [root]

    warnings: list[str] = []
    n_changed = 0
    for toml in iter_toml(targets):
        if process_file(toml, root, args.dry_run, warnings):
            n_changed += 1

    if warnings:
        print("\nWarnings (left unchanged):", file=sys.stderr)
        for w in warnings:
            print(f"  - {w}", file=sys.stderr)

    print(f"\n{n_changed} file(s) {'would be ' if args.dry_run else ''}changed, "
          f"{len(warnings)} warning(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
