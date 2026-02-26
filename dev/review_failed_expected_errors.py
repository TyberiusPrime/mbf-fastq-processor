#!/usr/bin/env python3
"""Review failed expected-error test cases interactively.

1. Removes all 'actual' and 'actual_N' folders from test_cases/
2. Runs 'cargo test'
3. For each failed test with an expected_error.txt, shows the expected
   content vs the actual stderr
4. Keys: [a] accept (update expected_error.txt)  [s/Enter] skip  [q] quit
"""

import difflib
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


def print_diff(expected: str, actual: str, max_lines: int = 80):
    """Print a unified diff between expected and actual."""
    exp_lines = expected.rstrip().splitlines(keepends=True)
    act_lines = actual.rstrip().splitlines(keepends=True)
    diff = list(difflib.unified_diff(
        exp_lines, act_lines,
        fromfile="expected_error.txt",
        tofile="stderr",
    ))
    print(f"\n{BOLD}── DIFF (expected → actual) ──{RESET}")
    if not diff:
        print(f"  {DIM}(no differences){RESET}")
        return
    shown = 0
    for line in diff:
        if shown >= max_lines:
            print(f"  {DIM}... output truncated{RESET}")
            break
        line = line.rstrip("\n")
        if line.startswith("---") or line.startswith("+++"):
            print(f"  {BOLD}{line}{RESET}")
        elif line.startswith("@@"):
            print(f"  {CYAN}{line}{RESET}")
        elif line.startswith("-"):
            print(f"  {RED}{line}{RESET}")
        elif line.startswith("+"):
            print(f"  {GREEN}{line}{RESET}")
        else:
            print(f"  {DIM}{line}{RESET}")
        shown += 1


def render(expected_content: str, new_content: str,
           test_dir: Path, idx: int, total: int, diff_mode: bool):
    """Clear screen and render the current test."""
    sep = "─" * 72
    print(f"\n{BOLD}{sep}{RESET}")
    mode_tag = f"{CYAN}[diff]{RESET}" if diff_mode else f"{DIM}[side-by-side]{RESET}"
    print(f"{BOLD}[{idx}/{total}]{RESET}  {CYAN}{test_dir}{RESET}  {mode_tag}")
    print(sep)
    if diff_mode:
        print_diff(expected_content, new_content)
    else:
        print_block("── EXPECTED (expected_error.txt) ──", YELLOW, expected_content)
        print_block("── ACTUAL   (stderr)              ──", GREEN, new_content)
    print(
        f"\n  {BOLD}[a]{RESET} accept  "
        f"{BOLD}[s/Enter]{RESET} skip  "
        f"{BOLD}[d]{RESET} toggle diff  "
        f"{BOLD}[r]{RESET} restart  "
        f"{BOLD}[q]{RESET} quit"
        "  → ",
        end="", flush=True,
    )


def review(stderr_file: Path, expected_file: Path, test_dir: Path,
           idx: int, total: int, diff_mode: bool) -> tuple[str, bool]:
    """Display one test, handling 'd' to toggle diff. Returns (key, diff_mode)."""
    expected_content = expected_file.read_text(encoding="utf-8")
    stderr_content = stderr_file.read_text(encoding="utf-8")
    new_content = extract_error_text(stderr_content)

    while True:
        render(expected_content, new_content, test_dir, idx, total, diff_mode)
        key = get_single_key()
        print(key)
        if key in ("d", "D"):
            diff_mode = not diff_mode
        else:
            return key, diff_mode


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
    diff_mode = False
    while True:
        i = 0
        while i < len(failed):
            stderr_file, expected_file, test_dir = failed[i]
            key, diff_mode = review(stderr_file, expected_file, test_dir, i + 1, len(failed), diff_mode)

            if key in ("a", "A"):
                content = stderr_file.read_text(encoding="utf-8")
                expected_file.write_text(extract_error_text(content), encoding="utf-8")
                print(f"  {GREEN}✓ Updated:{RESET} {expected_file}")
                accepted += 1
                i += 1
            elif key in ("r", "R"):
                print(f"  {CYAN}Restarting: removing actuals and re-running cargo test...{RESET}")
                remove_actual_folders(base_dir)
                run_cargo_test()
                failed = find_failed_error_tests(base_dir)
                if not failed:
                    print(f"\n{GREEN}All tests pass now.{RESET}")
                    break
                print(f"\n{BOLD}{len(failed)} failed error test(s) remaining.{RESET}")
                break  # break inner loop; outer loop restarts from i=0
            elif key in ("q", "Q", "\x03", "\x04"):   # q / Ctrl-C / Ctrl-D
                print(f"  {YELLOW}Quitting.{RESET}")
                remaining = len(failed) - i
                print(
                    f"\n{BOLD}Done.{RESET}  "
                    f"Accepted: {GREEN}{accepted}{RESET}  "
                    f"Skipped: {skipped}  "
                    f"Remaining: {remaining}"
                )
                return
            else:
                print(f"  {DIM}Skipped.{RESET}")
                skipped += 1
                i += 1
        else:
            # inner while finished normally (no break) — all tests reviewed
            break
        if not failed:
            break  # restart found no failures

    remaining = len(failed) - skipped
    print(
        f"\n{BOLD}Done.{RESET}  "
        f"Accepted: {GREEN}{accepted}{RESET}  "
        f"Skipped: {skipped}  "
        f"Remaining: {remaining}"
    )


if __name__ == "__main__":
    main()
