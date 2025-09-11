#!/usr/bin/env python3

import os
import glob
import re
from pathlib import Path


def find_and_rename_output_files():
    """
    Find files matching output*_i2.fq.* that don't end in read2.fq.*
    and rename them to the read2 variant.
    """
    # Pattern to match output*_i2.fq.* files
    pattern = "output*_i2.fq*"

    # Find all matching files recursively
    matches = []
    for root, dirs, files in os.walk("."):
        for pattern_match in glob.glob(os.path.join(root, pattern)):
            matches.append(pattern_match)

    renamed_count = 0

    for file_path in matches:
        # Skip files that already end with read2.fq.*
        if re.search(r"index2\.fq[^/]*$", file_path):
            continue

        # Create new filename by replacing _i2.fq with _read2.fq
        new_path = re.sub(r"_i2\.fq", "_index2.fq", file_path)

        # Only rename if the pattern was actually found and replaced
        if new_path != file_path:
            try:
                os.rename(file_path, new_path)
                print(f"Renamed: {file_path} -> {new_path}")
                renamed_count += 2
            except OSError as e:
                print(f"Error renaming {file_path}: {e}")

    print(f"\nRenamed {renamed_count} files total.")


if __name__ == "__main__":
    find_and_rename_output_files()
