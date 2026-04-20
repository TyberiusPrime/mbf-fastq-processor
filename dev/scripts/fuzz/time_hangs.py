#!/usr/bin/env python3
"""Time `fastqrab process` on every AFL hang input, sorted longest-first.

For each `dev/fuzz/fastq_parser/output/*/hangs/id:*`:

  1. mkdir a fresh tempdir
  2. copy the hang into it as `bug.fq`
  3. write a minimal `input.toml`:
         [input]
         read1 = 'bug.fq'
         [output]
         prefix = 'output'
  4. run `fastqrab process input.toml` with a wall-clock timeout
  5. record the duration and exit status

When done, prints all runs sorted by duration descending — the slowest /
timed-out inputs are the ones worth reproducing under a debugger.

Note on `read1`: the user-facing spec was `input.read = bug.fq`, but the
actual fastqrab schema field is `read1`. Strings also need quotes in TOML.

Note on format detection: `fastqrab process` runs format auto-detection on
the input, which peels gzip then expects '@' (fastq) or '>' (fasta). Many
AFL hangs are gzip blobs that decompress to garbage, so they will fail
format detection in milliseconds rather than reaching the parser. The hangs
that *do* reach the parser are the interesting ones.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
OUTPUT_DIR = REPO_ROOT / "dev" / "fuzz" / "fastq_parser" / "output"
DEFAULT_BIN = REPO_ROOT / "target_claude" / "release" / "fastqrab"

INPUT_TOML = (
    "[input]\n"
    "read1 = 'bug.fq'\n"
    "[output]\n"
    "prefix = 'output'\n"
)


def time_one(binary: Path, hang: Path, timeout: float) -> tuple[float, str]:
    with tempfile.TemporaryDirectory(prefix="fastqrab-hang-") as td:
        td_path = Path(td)
        shutil.copyfile(hang, td_path / "bug.fq")
        (td_path / "input.toml").write_text(INPUT_TOML)
        start = time.monotonic()
        try:
            r = subprocess.run(
                [str(binary), "process", "input.toml"],
                cwd=td_path,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=timeout,
            )
            dur = time.monotonic() - start
            status = "ok" if r.returncode == 0 else f"exit={r.returncode}"
        except subprocess.TimeoutExpired:
            dur = timeout
            status = "TIMEOUT"
        return dur, status


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--timeout", type=float, default=10.0,
                    help="Per-hang wall-clock timeout in seconds (default: 10)")
    ap.add_argument("--bin", type=Path, default=DEFAULT_BIN,
                    help=f"fastqrab binary (default: {DEFAULT_BIN})")
    ap.add_argument("--limit", type=int, default=None,
                    help="Only process the first N hangs (smoke test)")
    args = ap.parse_args()

    if not args.bin.is_file():
        print(f"error: fastqrab binary not found at {args.bin}", file=sys.stderr)
        return 2
    if not OUTPUT_DIR.exists():
        print(f"error: no fuzz output at {OUTPUT_DIR}", file=sys.stderr)
        return 2

    hangs = sorted(OUTPUT_DIR.glob("*/hangs/id:*"))
    if args.limit:
        hangs = hangs[: args.limit]
    if not hangs:
        print("error: no hangs found", file=sys.stderr)
        return 1

    results: list[tuple[float, str, Path]] = []
    for i, h in enumerate(hangs, 1):
        dur, status = time_one(args.bin, h, args.timeout)
        results.append((dur, status, h))
        print(
            f"[{i:>3}/{len(hangs)}] {dur:7.3f}s  {status:<10}  {h.name}",
            file=sys.stderr, flush=True,
        )

    print()
    print(f"# {len(results)} hangs, longest first (timeout={args.timeout}s)")
    print(f"# {'duration':>9}  {'status':<10}  path")
    for dur, status, h in sorted(results, key=lambda r: r[0], reverse=True):
        rel = h.relative_to(REPO_ROOT)
        print(f"  {dur:7.3f}s  {status:<10}  {rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
