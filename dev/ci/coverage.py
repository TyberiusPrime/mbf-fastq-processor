#!/usr/bin/env python3
"""
Coverage collection script for fastqrab

This script runs cargo llvm-cov and generates coverage reports in multiple formats.

## Coverage exclusion comments

Lines can be excluded from coverage statistics by adding these comments:

    fn unreachable_branch() { // cov:excl-line

    // cov:excl-start
    fn debug_only_helper() {
        ...
    }
    // cov:excl-stop

HTML reports use genhtml's native exclusion support (--rc lcov_excl_*) so
excluded lines are visually marked as excluded.  Summary and --lcov modes use
the same keywords via lcov post-processing.  JSON is generated directly by
cargo-llvm-cov and does not reflect exclusions.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

EXCL_LINE = "cov:excl-line"
EXCL_START = "cov:excl-start"
EXCL_STOP = "cov:excl-stop"


def run_command(cmd, description):
    """Run a command and handle errors."""
    print(f"  {description}...")
    try:
        result = subprocess.run(
            cmd, check=True, shell=True, capture_output=True, text=True
        )
        return result
    except subprocess.CalledProcessError as e:
        print(f"  FAILED: {description} (exit code {e.returncode})")
        if e.stdout:
            print(f"  stdout: {e.stdout}")
        if e.stderr:
            print(f"  stderr: {e.stderr}")
        sys.exit(1)


def get_excluded_lines(source_path: Path) -> set:
    """Return the set of line numbers excluded via coverage comments."""
    excluded = set()
    try:
        text = source_path.read_text(errors="replace")
    except OSError:
        return excluded

    in_block = False
    for lineno, line in enumerate(text.splitlines(), 1):
        if EXCL_START in line:
            in_block = True
        if in_block:
            excluded.add(lineno)
        if EXCL_STOP in line:
            in_block = False
        elif EXCL_LINE in line:
            excluded.add(lineno)

    return excluded


def apply_exclusions_to_lcov(lcov_path: Path, base_dir: Path = None, strip_excl: bool = True) -> tuple:
    """
    Post-process an lcov file in-place.

    Always normalizes relative SF: paths to absolute (resolved against base_dir
    or cwd) so genhtml sees a consistent path tree.

    When strip_excl is True (default), also removes DA/BRDA entries for lines
    marked with coverage exclusion comments and recomputes LF/LH/BRF/BRH.

    Returns (excluded_line_count, files_with_exclusions, total_lf, total_lh).
    """
    if base_dir is None:
        base_dir = Path.cwd()
    raw = lcov_path.read_text()
    output = []

    # Per-record state
    current_source = None
    excluded: set = set()
    record_header: list = []
    da_lines: list = []
    brda_lines: list = []
    lf = lh = brf = brh = 0

    total_excl = files_with_excl = total_lf = total_lh = 0

    for line in raw.splitlines():
        if line.startswith("SF:"):
            sf = Path(line[3:])
            if not sf.is_absolute():
                sf = (base_dir / sf).resolve()
            current_source = sf
            excluded = get_excluded_lines(current_source)
            if excluded:
                files_with_excl += 1
            record_header = [f"SF:{current_source}"]
            da_lines = []
            brda_lines = []
            lf = lh = brf = brh = 0

        elif line.startswith("DA:"):
            parts = line[3:].split(",")
            lineno = int(parts[0])
            if strip_excl and lineno in excluded:
                total_excl += 1
            else:
                hits = int(parts[1])
                lf += 1
                if hits > 0:
                    lh += 1
                da_lines.append(line)

        elif line.startswith("BRDA:"):
            # BRDA:line_number,block,branch,taken
            parts = line[5:].split(",")
            lineno = int(parts[0])
            if not (strip_excl and lineno in excluded):
                taken = parts[3]
                brf += 1
                if taken not in ("-", "0"):
                    brh += 1
                brda_lines.append(line)

        elif line.startswith(("LF:", "LH:", "BRF:", "BRH:")):
            pass  # recomputed below

        elif line == "end_of_record":
            output.extend(record_header)
            output.extend(da_lines)
            if brda_lines:
                output.extend(brda_lines)
                output.append(f"BRF:{brf}")
                output.append(f"BRH:{brh}")
            output.append(f"LF:{lf}")
            output.append(f"LH:{lh}")
            output.append("end_of_record")
            total_lf += lf
            total_lh += lh
            current_source = None
            record_header = []
            da_lines = []
            brda_lines = []
            lf = lh = brf = brh = 0

        else:
            if record_header:
                record_header.append(line)
            else:
                output.append(line)

    lcov_path.write_text("\n".join(output) + "\n")
    return total_excl, files_with_excl, total_lf, total_lh


def sum_lcov(lcov_path: Path) -> tuple:
    """Read total LF/LH from an lcov file."""
    lf = lh = 0
    for line in lcov_path.read_text().splitlines():
        if line.startswith("LF:"):
            lf += int(line[3:])
        elif line.startswith("LH:"):
            lh += int(line[3:])
    return lf, lh


def print_lcov_summary(total_excl: int, files_with_excl: int, total_lf: int, total_lh: int):
    """Print a concise coverage summary."""
    pct = (100.0 * total_lh / total_lf) if total_lf else 0.0
    print(f"\nCoverage summary:")
    print(f"  Lines: {total_lh}/{total_lf} ({pct:.1f}%)")
    if total_excl:
        print(f"  Excluded: {total_excl} line(s) across {files_with_excl} file(s)")


def check_genhtml():
    """Check that genhtml is available; exit with instructions if not."""
    try:
        subprocess.run(["genhtml", "--version"], check=True, capture_output=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("genhtml is not available (required for HTML reports).")
        print("Install via nix: add pkgs.lcov to your devShell, then re-enter with 'nix develop'.")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Generate coverage reports for fastqrab"
    )
    parser.add_argument("--html", action="store_true", help="Generate HTML coverage report")
    parser.add_argument("--lcov", action="store_true", help="Generate LCOV coverage report")
    parser.add_argument("--json", action="store_true", help="Generate JSON coverage report")
    parser.add_argument("--summary", action="store_true", help="Show coverage summary")
    parser.add_argument("--all", action="store_true", help="Generate all report formats")
    parser.add_argument(
        "--open",
        action="store_true",
        help="Open HTML report in browser after generation",
    )
    parser.add_argument(
        "--no-excl",
        action="store_true",
        help="Disable exclusion comment processing",
    )

    args = parser.parse_args()

    if not any([args.html, args.lcov, args.json, args.all, args.summary]):
        args.summary = True

    try:
        subprocess.run(["cargo", "llvm-cov", "--version"], check=True, capture_output=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("cargo-llvm-cov is not installed.")
        print("Install it with: cargo install cargo-llvm-cov")
        sys.exit(1)

    need_summary = args.summary or args.all
    need_lcov = args.lcov or args.all
    need_html = args.html or args.all
    need_json = args.json or args.all

    if need_html:
        check_genhtml()

    project_root = Path(__file__).parent.parent
    print(f"Running coverage from: {project_root}")

    # All coverage-data-producing modes share one test run via a raw lcov file.
    # HTML uses genhtml's native exclusion support (--rc lcov_excl_*) on the raw lcov.
    # Summary and --lcov post-process a copy of the raw lcov themselves.
    # JSON gets a separate cargo-llvm-cov run (no equivalent of genhtml --rc there).

    need_lcov_run = need_summary or need_lcov or need_html

    if need_lcov_run:
        raw_tmp = tempfile.NamedTemporaryFile(suffix=".lcov", delete=False)
        raw_lcov = Path(raw_tmp.name)
        raw_tmp.close()

        run_command(
            f"cargo llvm-cov test --lcov --output-path {raw_lcov}",
            "Generating coverage data",
        )

        # Normalize SF: paths and optionally strip excluded lines.
        # All subsequent consumers (genhtml, summary, coverage.lcov) get this file.
        excl, files, lf, lh = apply_exclusions_to_lcov(
            raw_lcov, base_dir=project_root, strip_excl=not args.no_excl
        )

        # --- HTML ---
        if need_html:
            html_dir = project_root / "coverage-html"
            run_command(
                f"genhtml {raw_lcov} --output-directory {html_dir} "
                f"--prefix {project_root} "
                f"--show-details --legend --no-function-coverage --ignore-errors category --quiet",
                "Rendering HTML report",
            )
            print(f"  HTML report: {html_dir}/index.html")
            if args.open:
                try:
                    import webbrowser
                    webbrowser.open(f"file://{(html_dir / 'index.html').absolute()}")
                except Exception as e:
                    print(f"  Could not open browser: {e}")

        # --- Summary / LCOV ---
        if need_summary:
            print_lcov_summary(excl, files, lf, lh)

        if need_lcov:
            shutil.copy(raw_lcov, "coverage.lcov")
            print(f"  LCOV report: coverage.lcov")

        raw_lcov.unlink(missing_ok=True)

    # --- JSON: separate run, no exclusion post-processing ---
    if need_json:
        run_command(
            "cargo llvm-cov test --json --output-path coverage.json",
            "Generating JSON coverage report",
        )
        print("  JSON report: coverage.json")

    print("\nCoverage collection complete.")


if __name__ == "__main__":
    main()
