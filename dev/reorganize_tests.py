#!/usr/bin/env python3
"""
Reorganize test cases based on rename_list.txt.
"""

import shutil
from pathlib import Path

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
