#!/usr/bin/env python3
"""
Reorganize test cases based on rename_list.txt.
"""

import shutil
from pathlib import Path
import os

def fix_symlinks_in_dir(directory):
    """Fix all symlinks in a directory after it has been moved."""
    for item in directory.rglob('*'):
        if item.is_symlink():
            try:
                # Read the symlink target (don't resolve, it's broken)
                old_target = os.readlink(item)

                # Find the sample_data part in the target
                if 'sample_data/' in old_target:
                    # Extract everything from sample_data onwards
                    sample_data_part = old_target[old_target.find('sample_data/'):]

                    # Calculate depth from test_cases to current location
                    relative_path = item.relative_to(Path('test_cases'))
                    depth = len(relative_path.parts) - 1  # -1 because we don't count the file itself

                    # Build new relative path: ../../../sample_data/...
                    new_target = '../' * depth + sample_data_part

                    # Update the symlink
                    item.unlink()
                    item.symlink_to(new_target)
                    print(f"    Fixed symlink: {item.name} -> {new_target}")
                else:
                    print(f"    Skipped non-sample_data symlink: {item.name} -> {old_target}")
            except Exception as e:
                print(f"    Warning: Could not fix symlink {item.relative_to(Path('test_cases'))}: {e}")

def main():
    # Read rename list
    renames = []
    with open('rename_list.txt', 'r') as f:
        lines = [line.rstrip('\n') for line in f]
        i = 0
        while i < len(lines):
            if i + 1 < len(lines) and lines[i] and lines[i + 1]:
                renames.append((lines[i], lines[i + 1]))
                i += 3  # Skip old, new, and empty line
            else:
                i += 1

    print(f"Found {len(renames)} renames")

    # Rename test_cases to test_cases_to_rename
    if Path('test_cases').exists():
        print("Renaming test_cases -> test_cases_to_rename")
        Path('test_cases').rename('test_cases_to_rename')
    else:
        print("ERROR: test_cases directory not found")
        return

    # Create new test_cases
    print("Creating new test_cases/")
    Path('test_cases').mkdir()

    # Perform renames
    print("Performing renames...")
    for old_path, new_path in renames:
        old_full = Path('test_cases_to_rename') / old_path
        new_full = Path('test_cases') / new_path

        if old_full.exists():
            new_full.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(old_full), str(new_full))
            print(f"  {old_path} -> {new_path}")

            # Fix symlinks in the moved directory
            if new_full.is_dir():
                fix_symlinks_in_dir(new_full)
        else:
            print(f"  SKIP (not found): {old_path}")

    # Remove empty directories from test_cases_to_rename
    print("Removing empty directories...")
    for dirpath in sorted(Path('test_cases_to_rename').rglob('*'), key=lambda p: len(p.parts), reverse=True):
        if dirpath.is_dir() and not any(dirpath.iterdir()):
            dirpath.rmdir()
            print(f"  Removed empty: {dirpath.relative_to('test_cases_to_rename')}")

    # Merge anything left over back to test_cases
    print("Merging remaining items...")
    remaining = list(Path('test_cases_to_rename').iterdir())
    if remaining:
        for item in remaining:
            dest = Path('test_cases') / item.name
            print(f"  Moving back: {item.name}")
            shutil.move(str(item), str(dest))

            # Fix symlinks in merged directories
            if dest.is_dir():
                fix_symlinks_in_dir(dest)
    else:
        print("  Nothing left to merge")

    # Remove test_cases_to_rename
    print("Removing test_cases_to_rename/")
    Path('test_cases_to_rename').rmdir()

    print("\nDone!")
    print("Next steps:")
    print("  1. dev/update_tests.py")
    print("  2. cargo test")

if __name__ == '__main__':
    main()
