#!/usr/bin/env python3
"""Review failed HTML-report test cases interactively.

1. Removes all 'actual' and 'actual_N' folders from test_cases/ and cookbooks/
2. Runs 'cargo test'
3. For each failed test where actual/…/output.html differs from the reference,
   shows a unified diff of the two files
4. Keys: [a] accept (overwrite reference with actual)  [s/Enter] skip  [q] quit
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


def remove_actual_folders(base_dirs: list[Path]) -> int:
    """Remove all 'actual' and 'actual_N' folders under the given base dirs."""
    removed = 0
    for base_dir in base_dirs:
        folders = [
            p
            for p in base_dir.rglob("*")
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


def find_failed_html_tests(base_dirs: list[Path]) -> list:
    """Return list of (actual_html, expected_html) pairs that differ."""
    results = []
    for base_dir in base_dirs:
        for filename in "output.html", "out.html":
            for actual_html in sorted(base_dir.rglob(filename)):
                # Must be inside an actual / actual_N directory
                actual_dir = None
                for part in actual_html.parts:
                    if re.match(r"^actual(_\d+)?$", part):
                        actual_dir = actual_html
                        break
                if actual_dir is None:
                    continue

                # Find the corresponding reference file:
                # strip the leading actual(_N) component from the path relative to base
                rel = actual_html.relative_to(base_dir)
                parts = list(rel.parts)
                # drop the 'actual' or 'actual_N' segment
                actual_idx = next(
                    i for i, p in enumerate(parts) if re.match(r"^actual(_\d+)?$", p)
                )
                ref_parts = parts[:actual_idx] + parts[actual_idx + 1 :]
                expected_html = base_dir.joinpath(*ref_parts)

                if not expected_html.exists():
                    continue

                actual_text = actual_html.read_text(encoding="utf-8")
                expected_text = expected_html.read_text(encoding="utf-8")
                if actual_text != expected_text:
                    results.append((actual_html, expected_html))

    return results


def print_diff(
    expected_path: Path,
    actual_path: Path,
    expected_text: str,
    actual_text: str,
    max_lines: int = 120,
):
    """Print a unified diff between expected and actual."""
    exp_lines = expected_text.rstrip().splitlines(keepends=True)
    act_lines = actual_text.rstrip().splitlines(keepends=True)
    diff = list(
        difflib.unified_diff(
            exp_lines,
            act_lines,
            fromfile=str(expected_path),
            tofile=str(actual_path),
        )
    )
    print(f"\n{BOLD}── DIFF (expected → actual) ──{RESET}")
    if not diff:
        print(f"  {DIM}(no differences){RESET}")
        return
    shown = 0
    for line in diff:
        if shown >= max_lines:
            remaining = len(diff) - shown
            print(f"  {DIM}... {remaining} more diff line(s) truncated{RESET}")
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


def render(actual_html: Path, expected_html: Path, idx: int, total: int):
    """Clear screen and render the current test."""
    sep = "─" * 72
    print(f"\n{BOLD}{sep}{RESET}")
    print(f"{BOLD}[{idx}/{total}]{RESET}  {CYAN}{expected_html}{RESET}")
    print(sep)

    actual_text = actual_html.read_text(encoding="utf-8")
    expected_text = expected_html.read_text(encoding="utf-8")
    print_diff(expected_html, actual_html, expected_text, actual_text)

    exp_lines = expected_text.rstrip().count("\n") + 1
    act_lines = actual_text.rstrip().count("\n") + 1
    print(f"\n  {DIM}Expected: {exp_lines} lines   Actual: {act_lines} lines{RESET}")
    print(
        f"\n  {BOLD}[a]{RESET} accept  "
        f"{BOLD}[s/Enter]{RESET} skip  "
        f"{BOLD}[r]{RESET} restart  "
        f"{BOLD}[q]{RESET} quit"
        "  → ",
        end="",
        flush=True,
    )


def main():
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent.parent
    base_dirs = [
        project_root / "test_cases",
        project_root / "cookbooks",
    ]
    for d in base_dirs:
        if not d.exists():
            sys.exit(f"Error: directory not found: {d}")

    remove_actual_folders(base_dirs)
    run_cargo_test()

    failed = find_failed_html_tests(base_dirs)
    if not failed:
        print(f"\n{GREEN}No failed HTML report tests found.{RESET}")
        return

    print(f"\n{BOLD}{len(failed)} failed HTML report test(s) to review.{RESET}")

    accepted = skipped = 0
    while True:
        i = 0
        while i < len(failed):
            actual_html, expected_html = failed[i]
            render(actual_html, expected_html, i + 1, len(failed))
            key = get_single_key()
            print(key)

            if key in ("a", "A"):
                shutil.copyfile(actual_html, expected_html)
                print(f"  {GREEN}✓ Updated:{RESET} {expected_html}")
                accepted += 1
                i += 1
            elif key in ("r", "R"):
                print(
                    f"  {CYAN}Restarting: removing actuals and re-running cargo test...{RESET}"
                )
                remove_actual_folders(base_dirs)
                run_cargo_test()
                failed = find_failed_html_tests(base_dirs)
                if not failed:
                    print(f"\n{GREEN}All HTML report tests pass now.{RESET}")
                    break
                print(
                    f"\n{BOLD}{len(failed)} failed HTML report test(s) remaining.{RESET}"
                )
                break  # break inner loop; outer loop restarts from i=0
            elif key in ("q", "Q", "\x03", "\x04"):  # q / Ctrl-C / Ctrl-D
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
            break
        if not failed:
            break

    remaining = len(failed) - skipped
    print(
        f"\n{BOLD}Done.{RESET}  "
        f"Accepted: {GREEN}{accepted}{RESET}  "
        f"Skipped: {skipped}  "
        f"Remaining: {remaining}"
    )


if __name__ == "__main__":
    main()
