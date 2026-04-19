#!/usr/bin/env python3
"""Drive an AFL++ fuzzing run against the fastq parser.

Builds the instrumented fuzz target (`dev/fuzz/fastq_parser`) with cargo-afl,
then launches one or more parallel `afl-fuzz` instances against the seed
corpus. With `-j N` (the default, N = CPU count), one instance is started
as `-M main` and the rest as `-S secN`; they share the same output directory
and synchronize through the filesystem. Extra flags after `--` are forwarded
to every afl-fuzz instance.

Live stats across all instances:
    afl-whatsup -s dev/fuzz/fastq_parser/output

Examples:
    ./dev/scripts/fuzzy_fastq_parser.py                  # parallel, all cores
    ./dev/scripts/fuzzy_fastq_parser.py -j 1             # single instance (TUI)
    ./dev/scripts/fuzzy_fastq_parser.py -j 8 --clean
    ./dev/scripts/fuzzy_fastq_parser.py -- -t 2000       # forward to afl-fuzz
"""

import argparse
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
FUZZ_DIR = REPO_ROOT / "dev" / "fuzz" / "fastq_parser"
CORPUS_DIR = FUZZ_DIR / "corpus"
OUTPUT_DIR = FUZZ_DIR / "output"
LOG_DIR = FUZZ_DIR / "logs"
# Pin the target directory so we don't chase a user-wide CARGO_TARGET_DIR
# or ~/.cargo/config.toml `build.target-dir` setting.
TARGET_DIR = FUZZ_DIR / "target"
TARGET_BIN = TARGET_DIR / "release" / "fastq_parser_fuzz"

CARGO_AFL = os.environ.get("CARGO_AFL") or shutil.which("cargo-afl")


def run(cmd, **kwargs):
    print("+", " ".join(str(c) for c in cmd), flush=True)
    return subprocess.run(cmd, **kwargs)


def build_afl_cmd(role_flag: str, name: str, extra: list[str]) -> list[str]:
    return [
        CARGO_AFL, "afl", "fuzz",
        "-i", str(CORPUS_DIR),
        "-o", str(OUTPUT_DIR),
        role_flag, name,
        *extra,
        "--",
        str(TARGET_BIN),
    ]


def run_single(extra: list[str], env: dict) -> int:
    # Foreground: let afl-fuzz own the terminal so the user sees the TUI.
    cmd = build_afl_cmd("-M", "main", extra)
    print("+", " ".join(cmd), flush=True)
    os.execvpe(cmd[0], cmd, env)


def run_parallel(jobs: int, extra: list[str], env: dict) -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    names = ["main"] + [f"sec{i}" for i in range(1, jobs)]

    children: list[tuple[str, subprocess.Popen]] = []
    log_handles = []

    # Each afl-fuzz instance owns its own stdout: AFL's TUI draws escape codes
    # that look like noise when many instances share a terminal, so we send
    # each one to its own log file and point the user at `afl-whatsup`.
    for i, name in enumerate(names):
        role = "-M" if i == 0 else "-S"
        cmd = build_afl_cmd(role, name, extra)
        log_path = LOG_DIR / f"{name}.log"
        f = log_path.open("wb")
        log_handles.append(f)
        print(f"+ [{name}] {' '.join(cmd)}  >  {log_path}", flush=True)
        # start_new_session=True so Ctrl-C hitting us doesn't also SIGINT the
        # children via the terminal; we forward it explicitly.
        p = subprocess.Popen(
            cmd,
            stdout=f,
            stderr=subprocess.STDOUT,
            start_new_session=True,
            env=env,
        )
        children.append((name, p))
        # Stagger slightly so instances don't all pick the same CPU before
        # AFL has updated its core-binding table.
        time.sleep(0.3)

    print()
    print(f"{jobs} afl-fuzz instances running. Live stats:")
    print(f"    afl-whatsup -s {OUTPUT_DIR}")
    print("Ctrl-C here to stop all instances.")

    def shutdown(_signum=None, _frame=None):
        print("\nstopping fuzzers...", flush=True)
        for _name, p in children:
            if p.poll() is None:
                try:
                    p.send_signal(signal.SIGINT)
                except ProcessLookupError:
                    pass
        deadline = time.monotonic() + 15
        for _name, p in children:
            remaining = max(0.1, deadline - time.monotonic())
            try:
                p.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                p.kill()

    signal.signal(signal.SIGINT, lambda *a: (shutdown(), sys.exit(130)))
    signal.signal(signal.SIGTERM, lambda *a: (shutdown(), sys.exit(143)))

    try:
        while True:
            alive = [(n, p) for n, p in children if p.poll() is None]
            if not alive:
                break
            if len(alive) < len(children):
                dead = [(n, p) for n, p in children if p.poll() is not None]
                for n, p in dead:
                    if (n, p) in children:
                        print(
                            f"warning: instance {n} exited with {p.returncode}; "
                            f"tail {LOG_DIR / f'{n}.log'} for details",
                            flush=True,
                        )
                        children.remove((n, p))
            time.sleep(2)
    finally:
        for f in log_handles:
            f.close()

    rc = max((p.returncode or 0 for _, p in children), default=0)
    return rc


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "-j", "--jobs",
        type=int,
        default=max(1, (os.cpu_count() or 2) - 1),
        help="Number of parallel afl-fuzz instances (default: cpu_count - 1 "
             "to leave a core free; AFL refuses to start instances when it "
             "thinks all cores are CPU-locked). With -j 1, runs a single "
             "foreground instance with the AFL TUI.",
    )
    parser.add_argument(
        "--no-affinity",
        action="store_true",
        help="Set AFL_NO_AFFINITY=1 so instances don't bind to specific cores. "
             "Use this if some instances exit with 'all N CPU cores allocated'.",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Wipe the afl output + logs directories before fuzzing (loses queue + crashes).",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip the cargo-afl build step.",
    )
    parser.add_argument(
        "--canary",
        action="store_true",
        help="Build with the afl-positive-control feature: injects a panic on "
             "any read name containing AFL_POSITIVE_CONTROL. The canary seed "
             "is one +1-byte mutation away from tripping it, so AFL's arith8 "
             "stage should find the crash within seconds. Use to validate the "
             "fuzzing setup end-to-end: binary, coverage, and mutation.",
    )
    parser.add_argument(
        "afl_args",
        nargs=argparse.REMAINDER,
        help="Extra args forwarded to every afl-fuzz instance (use `--` to separate).",
    )
    args = parser.parse_args()

    if args.jobs < 1:
        print("error: --jobs must be >= 1", file=sys.stderr)
        return 2

    if not CARGO_AFL or not Path(CARGO_AFL).is_file():
        print(
            "error: cargo-afl not found on PATH. Run this inside `nix develop`, "
            "or set the CARGO_AFL env var.",
            file=sys.stderr,
        )
        return 2

    if not CORPUS_DIR.exists() or not any(CORPUS_DIR.iterdir()):
        print(
            f"error: corpus directory is empty or missing: {CORPUS_DIR}",
            file=sys.stderr,
        )
        return 2

    if args.clean:
        for d in (OUTPUT_DIR, LOG_DIR):
            if d.exists():
                print(f"removing {d}")
                shutil.rmtree(d)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    build_env = os.environ.copy()
    build_env["CARGO_TARGET_DIR"] = str(TARGET_DIR)

    if not args.no_build:
        build_cmd = [CARGO_AFL, "afl", "build", "--release"]
        if args.canary:
            build_cmd += ["--features", "afl-positive-control"]
        r = run(build_cmd, cwd=FUZZ_DIR, env=build_env)
        if r.returncode != 0:
            return r.returncode

    if not TARGET_BIN.exists():
        print(f"error: fuzz binary not found at {TARGET_BIN}", file=sys.stderr)
        return 1

    extra = args.afl_args[1:] if args.afl_args and args.afl_args[0] == "--" else args.afl_args

    run_env = os.environ.copy()
    if args.no_affinity:
        run_env["AFL_NO_AFFINITY"] = "1"
    # Resume from an existing output dir (queue, crashes, coverage bitmap) when
    # one is present. Without this AFL refuses to reuse the dir; with --clean
    # the dir was just wiped so there's nothing to resume from and AFL starts
    # fresh either way.
    run_env["AFL_AUTORESUME"] = "1"

    if args.jobs == 1:
        return run_single(extra, run_env)
    return run_parallel(args.jobs, extra, run_env)


if __name__ == "__main__":
    sys.exit(main())
