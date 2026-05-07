#!/usr/bin/env python3
"""For each ancestor revision, count lines of Rust code using tokei."""

import csv
import subprocess
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
CSV_PATH = PROJECT_ROOT / "dev" / "lines_per_revision.csv"


def run(cmd, **kwargs):
    return subprocess.run(
        cmd, cwd=PROJECT_ROOT, capture_output=True, text=True, check=True, **kwargs
    )


def get_revisions():
    result = run(
        [
            "jj",
            "log",
            "--no-graph",
            "-r",
            "ancestors(@)",
            "-T",
            "commit_id ++ '---' ++ committer.timestamp() ++ '!!!'",
        ]
    )
    revisions = []
    for line in result.stdout.strip().split("!!!"):
        if not line:
            continue
        print(repr(line))
        rev_id, date = line.split("---", 1)
        revisions.append((rev_id, date))
    return revisions


def get_current_rev():
    result = run(["jj", "log", "-r", "@", "-T", "commit_id", "--no-graph"])
    return result.stdout.strip()


def count_rust_lines():
    result = subprocess.run(
        "tokei -o json --exclude generated.rs | jq .Rust.code",
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
        shell=True,
    )
    if result.returncode != 0:
        return None
    value = result.stdout.strip()
    if value == "null" or not value:
        return 0
    return int(value)


def load_existing():
    existing = {}
    if CSV_PATH.exists():
        with open(CSV_PATH, "r") as f:
            reader = csv.reader(f)
            for row in reader:
                if len(row) >= 3:
                    existing[row[0]] = (row[1], row[2])
    return existing


def main():
    original_rev = get_current_rev()
    revisions = get_revisions()
    existing = load_existing()

    missing = [(rid, date) for rid, date in revisions if rid not in existing]

    if not missing:
        print("All revisions already recorded.")
        return

    print(
        f"Processing {len(missing)} missing revisions out of {len(revisions)} total."
    )

    for rev_id, date in missing:
        print(f"Switching to {rev_id[:12]} ({date})...")
        run(["jj", "new", rev_id])

        lines = count_rust_lines()
        if lines is None:
            print("  Failed to count lines, skipping.")
            continue

        print(f"  {lines} lines of Rust code")
        existing[rev_id] = (date, str(lines))

    print(f"Restoring to original revision {original_rev[:12]}...")
    run(["jj", "new", original_rev])

    with open(CSV_PATH, "w") as f:
        writer = csv.writer(f)
        for rev_id, date in revisions:
            if rev_id in existing:
                stored_date, lines = existing[rev_id]
                writer.writerow([rev_id, stored_date, lines])

    print(f"Results written to {CSV_PATH}")


if __name__ == "__main__":
    main()
