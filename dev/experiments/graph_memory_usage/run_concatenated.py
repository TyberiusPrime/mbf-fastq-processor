import os
import subprocess
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

REPS: List[int] = [1, 2, 4, 8, 9,10,11,12,13,14,15, 16, 160]

WORKDIR = Path(__file__).resolve().parent
PROJECT_ROOT = WORKDIR.parents[1]
TARGET_BIN = PROJECT_ROOT / "target" / "release" / "mbf-fastq-processor"
BASE_FASTQ_COMPRESSED = (
    PROJECT_ROOT
    / "test_cases"
    / "integration_tests"
    / "inspect_read1"
    / "input_read1.fq.zst"
).resolve()
RAW_FASTQ_PATH = WORKDIR / "base_input.fq"

_RAW_CACHE: Optional[bytes] = None


def ensure_binary() -> None:
    if not TARGET_BIN.exists():
        subprocess.run(
            ["cargo", "build", "--release"],
            cwd=PROJECT_ROOT,
            check=True,
        )


def ensure_raw_fastq() -> bytes:
    global _RAW_CACHE
    if _RAW_CACHE is not None:
        return _RAW_CACHE

    result = subprocess.run(
        ["zstd", "-d", "-q", "-c", str(BASE_FASTQ_COMPRESSED)],
        cwd=PROJECT_ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    RAW_FASTQ_PATH.write_bytes(result.stdout)
    _RAW_CACHE = result.stdout
    return _RAW_CACHE


def build_concatenated_input(repetitions: int) -> Path:
    raw_bytes = ensure_raw_fastq()
    raw_concat = WORKDIR / "input_concatenated.fq"
    compressed_concat = WORKDIR / "input_concatenated.fq.zst"

    with raw_concat.open("wb") as raw_out:
        for _ in range(repetitions):
            raw_out.write(raw_bytes)

    subprocess.run(
        ["zstd", "-q", "-f", str(raw_concat), "-o", str(compressed_concat)],
        cwd=WORKDIR,
        check=True,
    )
    return compressed_concat


def render_config(repetitions: int) -> Path:
    input_toml = WORKDIR / "input.toml"
    compressed_path = build_concatenated_input(repetitions)
    config = f"""
[input]
    read1 = ['{compressed_path}']

[options]
    accept_duplicate_files = true
    thread_count = 1

[output]
    prefix = 'no_output'
    format = 'None'
"""
    input_toml.write_text(config, encoding="utf-8")
    return input_toml


def parse_alloc_metrics(stderr: str) -> Optional[Dict[str, int]]:
    for line in stderr.splitlines():
        if line.startswith("alloc: "):
            metrics: Dict[str, int] = {}
            for token in line[len("alloc: ") :].split():
                if "=" not in token:
                    continue
                key, value = token.split("=", 1)
                try:
                    metrics[key] = int(value)
                except ValueError:
                    continue
            return metrics
    return None


def read_status_metrics(status_path: Path) -> Tuple[Optional[int], Optional[int]]:
    try:
        with status_path.open("r", encoding="utf-8") as status:
            vmhwm: Optional[int] = None
            vmrss: Optional[int] = None
            for line in status:
                if line.startswith("VmHWM:"):
                    parts = line.split()
                    if len(parts) >= 2:
                        vmhwm = int(parts[1]) * 1024
                elif line.startswith("VmRSS:"):
                    parts = line.split()
                    if len(parts) >= 2:
                        vmrss = int(parts[1]) * 1024
                if vmhwm is not None and vmrss is not None:
                    break
            return vmhwm, vmrss
    except FileNotFoundError:
        return None, None
    return None, None


def run_process(config: Path, env_override: Optional[Dict[str, str]]) -> Tuple[Optional[Dict[str, int]], Optional[int], Optional[int]]:
    env = os.environ.copy()
    if env_override:
        env.update(env_override)

    proc = subprocess.Popen(
        [
            str(TARGET_BIN),
            "process",
            str(config),
            str(WORKDIR),
        ],
        cwd=WORKDIR,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    status_path = Path("/proc") / str(proc.pid) / "status"
    max_rss: Optional[int] = None
    final_rss: Optional[int] = None

    while proc.poll() is None:
        vmhwm, vmrss = read_status_metrics(status_path)
        if vmhwm is not None:
            if max_rss is None or vmhwm > max_rss:
                max_rss = vmhwm
        if vmrss is not None:
            final_rss = vmrss
        time.sleep(0.05)

    _, stderr = proc.communicate()

    vmhwm, vmrss = read_status_metrics(status_path)
    if vmhwm is not None:
        if max_rss is None or vmhwm > max_rss:
            max_rss = vmhwm
    if vmrss is not None:
        final_rss = vmrss

    if proc.returncode != 0:
        raise RuntimeError(f"Command failed: {stderr}")

    alloc_metrics = parse_alloc_metrics(stderr)
    return alloc_metrics, max_rss, final_rss


def run_once(repetitions: int) -> Tuple[Optional[Dict[str, int]], Optional[int], Optional[int]]:
    config = render_config(repetitions)
    alloc_metrics, _, _ = run_process(config, {"RUST_MEASURE_ALLOC": "1"})
    _, rss_peak, rss_final = run_process(config, None)
    return alloc_metrics, rss_peak, rss_final


def print_table(rows: List[List[str]]) -> None:
    widths = [max(len(row[col]) for row in rows) for col in range(len(rows[0]))]
    for row in rows:
        print("  ".join(cell.rjust(widths[idx]) for idx, cell in enumerate(row)))


def cleanup_intermediate_files() -> None:
    for filename in [
        WORKDIR / "input.toml",
        WORKDIR / "input_concatenated.fq",
        WORKDIR / "input_concatenated.fq.zst",
        RAW_FASTQ_PATH,
    ]:
        try:
            filename.unlink()
        except FileNotFoundError:
            pass


def main() -> None:
    ensure_binary()
    rows: List[List[str]] = [[
        "rep",
        "alloc_bytes_max",
        "alloc_bytes_current",
        "max_rss_bytes",
        "final_rss_bytes",
        "bytes_max_per_rep",
    ]]

    try:
        for rep in REPS:
            alloc, rss_peak, rss_final = run_once(rep)
            bytes_max = alloc.get("bytes_max") if alloc else None
            bytes_current = alloc.get("bytes_current") if alloc else None
            per_rep = f"{bytes_max / rep:.2f}" if bytes_max is not None else "NA"
            rows.append([
                str(rep),
                str(bytes_max) if bytes_max is not None else "NA",
                str(bytes_current) if bytes_current is not None else "NA",
                str(rss_peak) if rss_peak is not None else "NA",
                str(rss_final) if rss_final is not None else "NA",
                per_rep,
            ])
    finally:
        cleanup_intermediate_files()

    print_table(rows)


if __name__ == "__main__":
    main()
