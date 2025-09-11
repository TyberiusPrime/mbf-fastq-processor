#!/usr/bin/env python3
"""
Find file pairs output_*.fq / actual/output_*.fq that differ by one byte in size.
If the only difference is a newline at the end, copy the actual file over the expected one.
"""

import os
import shutil
from pathlib import Path


def find_and_fix_newline_diffs(root_dir="."):
    """Find and fix files that differ only by a trailing newline."""
    root_path = Path(root_dir)
    fixes_made = 0
    pairs_checked = 0

    # Find all output_*.fq files
    for output_file in root_path.rglob("output_*.fq"):
        # Skip files in actual/ directories
        if "actual" in output_file.parts:
            continue

        # Find corresponding actual file
        actual_file = output_file.parent / "actual" / output_file.name

        if not actual_file.exists():
            continue

        pairs_checked += 1

        # Check if they differ by exactly 1 byte
        output_size = output_file.stat().st_size
        actual_size = actual_file.stat().st_size

        if abs(output_size - actual_size) != 1:
            continue

        # Read both files
        try:
            with open(output_file, "rb") as f:
                output_content = f.read()
            with open(actual_file, "rb") as f:
                actual_content = f.read()
        except IOError as e:
            print(f"Error reading files {output_file} or {actual_file}: {e}")
            continue

        # Check if the only difference is a trailing newline
        differs_by_newline = False

        # if len(output_content) == len(actual_content) + 1:
        #     # output_file is 1 byte larger
        #     if output_content == actual_content + b"\n":
        #         differs_by_newline = True
        if len(actual_content) == len(output_content) + 1:
            # actual_file is 1 byte larger
            if actual_content == output_content + b"\n":
                differs_by_newline = True

        if differs_by_newline:
            print(f"Fixing newline difference: {output_file}")
            try:
                shutil.copy2(actual_file, output_file)
                fixes_made += 1
            except IOError as e:
                print(f"Error copying {actual_file} to {output_file}: {e}")

    print(f"Checked {pairs_checked} file pairs")
    print(f"Fixed {fixes_made} files with newline differences")


if __name__ == "__main__":
    find_and_fix_newline_diffs()
