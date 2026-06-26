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
import os
from pathlib import Path
import re

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
    wrong_exclusions: dict = {}  # src Path -> [lineno, ...]

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
            hits = int(parts[1])
            if strip_excl and lineno in excluded:
                total_excl += 1
                if hits > 0:
                    wrong_exclusions.setdefault(current_source, []).append(lineno)
            else:
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
    return total_excl, files_with_excl, total_lf, total_lh, wrong_exclusions


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


def _inject_css(html_dir: Path):
    """Append CSS rules for wrong-exclusion highlighting to gcov.css."""
    css_path = html_dir / "gcov.css"
    extra = """

/* Wrong exclusion: line marked cov:excl but actually executed */
span.wrongExcl {
  background-color: #e080e0;
}
td.coverNumWrong {
  text-align: right;
  padding-left: 10px;
  padding-right: 10px;
  background-color: #e080e0;
  font-weight: bold;
  font-family: sans-serif;
}
span.coverLegendWrong {
  padding-left: 10px;
  padding-right: 10px;
  padding-bottom: 2px;
  background-color: #e080e0;
}
"""
    css_path.write_text(css_path.read_text() + extra)


def _color_wrong_lines(gcov_path: Path, wrong_linenos: list[int]):
    """Color wrongly-excluded lines in a source-view HTML file."""
    text = gcov_path.read_text()
    for lineno in wrong_linenos:
        # Excluded lines have no coverage span, just ': content' in plain text:
        #   <span id="L{N}"><span class="lineNum">     {N}</span>              : content</span>
        pattern = rf'(<span id="L{lineno}"><span class="lineNum">[^<]*</span>)(\s+: [^<]*)(</span>)'
        text = re.sub(pattern, r'\1<span class="wrongExcl">\2</span>\3', text)
    gcov_path.write_text(text)


def _add_source_legend(gcov_path: Path):
    """Add 'wrong excl' entry to the source-view legend."""
    text = gcov_path.read_text()
    text = re.sub(
        r'(<span class="coverLegendNoCov">not hit</span>)',
        r'\1\n            <span class="coverLegendWrong">wrong excl</span>',
        text,
    )
    gcov_path.write_text(text)


def _process_index(idx_path: Path, html_dir: Path,
                   file_wrong: dict[str, int], dir_wrong: dict[str, int]):
    """Add a 'Wrong' column to an index page's coverage table."""
    text = idx_path.read_text()
    lines = text.split('\n')
    idx_rel_dir = str(idx_path.relative_to(html_dir).parent)

    output = []
    i = 0
    n = len(lines)

    while i < n:
        line = lines[i]

        # --- Header row 1: contains 'Line Coverage' colspan ---
        if 'colspan=4>Line Coverage' in line:
            # Emit everything up to (and including) the </tr> of this row.
            while i < n and lines[i].strip() != '</tr>':
                output.append(lines[i])
                i += 1
            # Insert Wrong column header before </tr>
            indent = lines[i][:len(lines[i]) - len(lines[i].lstrip())]
            output.append(f'{indent}<td class="tableHead" rowspan=2>Wrong</td>')
            output.append(lines[i])
            i += 1
            continue

        # --- Data row: starts with coverFile or coverDirectory ---
        if ('<td class="coverFile">' in line or '<td class="coverDirectory">' in line) \
                and '<a href=' in line:
            href_match = re.search(r'href="([^"]+)"', line)
            href = href_match.group(1) if href_match else None
            is_directory = '<td class="coverDirectory">' in line

            output.append(line)
            i += 1

            # Collect remaining lines of this row until </tr>.
            # Use strip() to skip nested </tr> inside the coverBar's inner table.
            while i < n and lines[i].strip() != '</tr>':
                output.append(lines[i])
                i += 1

            # Compute wrong-exclusion count for this row
            wrong_count = 0
            if href:
                if is_directory:
                    sub_dir = str((Path(idx_rel_dir) / href).parent)
                    wrong_count = dir_wrong.get(sub_dir, 0)
                else:
                    gcov_full = str(Path(idx_rel_dir) / href)
                    wrong_count = file_wrong.get(gcov_full, 0)

            # Insert Wrong column cell before </tr>
            indent = lines[i][:len(lines[i]) - len(lines[i].lstrip())]
            if wrong_count > 0:
                output.append(f'{indent}<td class="coverNumWrong">{wrong_count}</td>')
            else:
                output.append(f'{indent}<td class="coverNumDflt"></td>')
            output.append(lines[i])
            i += 1
            continue

        output.append(line)
        i += 1

    idx_path.write_text('\n'.join(output))


def _post_process_html(html_dir: Path, wrong_exclusions: dict, project_root: Path):
    """Post-process genhtml output to highlight wrongly-excluded lines.

    Injects CSS, colors source lines, adds legend entries, and appends a
    'Wrong' column to all coverage report tables.
    """
    if not wrong_exclusions:
        return

    # Build look-up tables keyed by path *relative to html_dir*.
    # genhtml mirrors the absolute source path under html_dir, e.g.
    #   SF:/project/main/fastqrab/src/lib.rs
    #   -> html_dir/project/main/fastqrab/src/lib.rs.gcov.html
    file_wrong: dict[str, int] = {}   # .gcov.html rel-path -> count
    dir_wrong: dict[str, int] = {}     # directory rel-path   -> total count

    for src, linenos in wrong_exclusions.items():
        rel = str(src)[1:]                     # strip leading '/'
        gcov_rel = rel + '.gcov.html'
        file_wrong[gcov_rel] = len(linenos)
        parent = str(Path(gcov_rel).parent)
        dir_wrong[parent] = dir_wrong.get(parent, 0) + len(linenos)

    # 1. CSS
    _inject_css(html_dir)

    # 2. Source views: color wrong lines + legend
    for src, linenos in wrong_exclusions.items():
        rel = str(src)[1:]
        gcov_path = html_dir / (rel + '.gcov.html')
        if gcov_path.exists():
            _color_wrong_lines(gcov_path, linenos)
            _add_source_legend(gcov_path)

    # 3. Index tables: add 'Wrong' column
    for idx_path in sorted(html_dir.rglob('index*.html')):
        if 'detail' in idx_path.name:
            continue
        _process_index(idx_path, html_dir, file_wrong, dir_wrong)


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
        excl, files, lf, lh, wrong_exclusions = apply_exclusions_to_lcov(
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
            _post_process_html(html_dir, wrong_exclusions, project_root)
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

        # --- Wrong exclusions ---
        if wrong_exclusions:
            total_wrong = sum(len(v) for v in wrong_exclusions.values())
            files_wrong = len(wrong_exclusions)
            # Compute a sensible display root from the actual source paths
            # (project_root may be dev/ while sources live under the workspace root)
            common_parent = Path(os.path.commonpath(
                [str(p) for p in wrong_exclusions]
            ))
            if not common_parent.is_dir():
                common_parent = common_parent.parent
            print(f"\n  Wrong exclusions: {total_wrong} line(s) across {files_wrong} file(s)")
            for src, linenos in sorted(wrong_exclusions.items()):
                try:
                    rel = src.relative_to(project_root)
                except ValueError:
                    rel = src.relative_to(common_parent)
                print(f"    {len(linenos):>4d}  {rel}")

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
