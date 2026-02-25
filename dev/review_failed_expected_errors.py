#!/usr/bin/env python3
"""Review failed expected-error test cases interactively.

1. Removes all 'actual' and 'actual_N' folders from test_cases/
2. Runs 'cargo test'
3. For each failed test with an expected_error.txt, shows the expected
   content vs the actual stderr
4. Keys: [a] accept (update expected_error.txt)  [s/Enter] skip  [q] quit
"""

import re
import shutil
import subprocess
import sys
import termios
import tty
from pathlib import Path

# ANSI colours
RESET = "\033[0m"
BOLD = "\033[1m"
RED = "\033[31m"
GREEN = "\033[32m"
YELLOW = "\033[33m"
CYAN = "\033[36m"
DIM = "\033[2m"


def get_single_key() -> str:
    """Read one keypress without requiring Enter."""
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
    return ch


def remove_actual_folders(base_dir: Path) -> int:
    """Remove all 'actual' and 'actual_N' folders under base_dir."""
    removed = 0
    # collect first so we don't modify while iterating
    folders = [
        p for p in base_dir.rglob("*")
        if p.is_dir() and re.match(r"^actual(_\d+)?$", p.name)
    ]
    for folder in folders:
        shutil.rmtree(folder)
        removed += 1
    print(f"Removed {removed} actual folder(s).")
    return removed


def run_cargo_test() -> int:
    """Run cargo test; output goes straight to the terminal."""
    print(f"\n{BOLD}Running cargo test...{RESET}\n")
    result = subprocess.run(["cargo", "test"])
    return result.returncode


def find_failed_error_tests(base_dir: Path) -> list:
    """Return list of (stderr_file, expected_error_file, test_dir) tuples."""
    seen = set()
    results = []
    for stderr_file in sorted(base_dir.rglob("stderr")):
        actual_dir = stderr_file.parent
        if not re.match(r"^actual(_\d+)?$", actual_dir.name):
            continue
        test_dir = actual_dir.parent
        if test_dir in seen:
            continue
        expected_file = test_dir / "expected_error.txt"
        if expected_file.exists():
            seen.add(test_dir)
            results.append((stderr_file, expected_file, test_dir))
    return results


def extract_error_text(stderr_content: str) -> str:
    """Extract from the first 'Error' occurrence to the end."""
    idx = stderr_content.find("Error")
    return stderr_content[idx:] if idx != -1 else stderr_content


def print_block(label: str, colour: str, text: str, max_lines: int = 60):
    """Print a labelled, coloured block of text."""
    print(f"\n{BOLD}{colour}{label}{RESET}")
    lines = text.rstrip().splitlines()
    if len(lines) > max_lines:
        for line in lines[:max_lines]:
            print(f"  {colour}{line}{RESET}")
        print(f"  {DIM}... ({len(lines) - max_lines} more lines){RESET}")
    else:
        for line in lines:
            print(f"  {colour}{line}{RESET}")


def review(stderr_file: Path, expected_file: Path, test_dir: Path,
           idx: int, total: int) -> str:
    """Display one test and return the key pressed."""
    expected_content = expected_file.read_text(encoding="utf-8")
    stderr_content = stderr_file.read_text(encoding="utf-8")
    new_content = extract_error_text(stderr_content)

    sep = "─" * 72
    print(f"\n{BOLD}{sep}{RESET}")
    print(f"{BOLD}[{idx}/{total}]{RESET}  {CYAN}{test_dir}{RESET}")
    print(sep)

    print_block("── EXPECTED (expected_error.txt) ──", YELLOW, expected_content)
    print_block("── ACTUAL   (stderr)              ──", GREEN, new_content)

    print(
        f"\n  {BOLD}[a]{RESET} accept  "
        f"{BOLD}[s/Enter]{RESET} skip  "
        f"{BOLD}[q]{RESET} quit"
        "  → ",
        end="", flush=True,
    )
    key = get_single_key()
    print(key)
    return key


def main():
    # Locate test_cases from the project root (script lives in dev/)
    script_dir = Path(__file__).resolve().parent
    base_dir = script_dir.parent / "test_cases"
    if not base_dir.exists():
        sys.exit(f"Error: test_cases not found at {base_dir}")

    remove_actual_folders(base_dir)
    run_cargo_test()

    failed = find_failed_error_tests(base_dir)
    if not failed:
        print(f"\n{GREEN}No failed error tests found.{RESET}")
        return

    print(f"\n{BOLD}{len(failed)} failed error test(s) to review.{RESET}")

    accepted = skipped = 0
    for idx, (stderr_file, expected_file, test_dir) in enumerate(failed, 1):
        key = review(stderr_file, expected_file, test_dir, idx, len(failed))

        if key in ("a", "A"):
            content = stderr_file.read_text(encoding="utf-8")
            expected_file.write_text(extract_error_text(content), encoding="utf-8")
            print(f"  {GREEN}✓ Updated:{RESET} {expected_file}")
            accepted += 1
        elif key in ("q", "Q", "\x03", "\x04"):   # q / Ctrl-C / Ctrl-D
            print(f"  {YELLOW}Quitting.{RESET}")
            break
        else:
            print(f"  {DIM}Skipped.{RESET}")
            skipped += 1

    remaining = len(failed) - accepted - skipped
    print(
        f"\n{BOLD}Done.{RESET}  "
        f"Accepted: {GREEN}{accepted}{RESET}  "
        f"Skipped: {skipped}  "
        f"Remaining: {remaining}"
    )


if __name__ == "__main__":
    main()
