#!/usr/bin/env python3
"""Run cargo test until the first failure and open a fish shell in the test case directory."""

import os
import re
import subprocess
import sys
from pathlib import Path

PATTERN = re.compile(r"Test case is in: (\S+)")


def main():
    project_root = Path(__file__).resolve().parent.parent.parent

    print("Running cargo test (stopping at first failure)...\n")
    result = subprocess.run(
        ["cargo", "test"],
        cwd=project_root,
        capture_output=True,
        text=True,
    )

    combined = result.stdout + result.stderr
    print(result.stdout)
    print(result.stderr, file=sys.stderr)

    matches = PATTERN.findall(combined)
    if not matches:
        if result.returncode == 0:
            print("\nAll tests passed.")
        else:
            print("\nNo 'Test case is in:' found in output.")
        sys.exit(result.returncode)

    test_case_dir = project_root / matches[-1]
    if not test_case_dir.exists():
        print(f"\nDirectory not found: {test_case_dir}")
        sys.exit(1)

    print(f"\nOpening fish shell in: {test_case_dir}\n")
    os.chdir(test_case_dir)
    os.execvp("fish", ["fish"])


if __name__ == "__main__":
    main()
