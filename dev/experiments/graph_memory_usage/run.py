import os
import subprocess
import time
from pathlib import Path


# note that RSS based measurements are not reliable for the very fast
# runs below 8 or so reps
REPS = [1, 2, 4, 8,9,10, 16, 160, 320]

WORKDIR = Path(__file__).resolve().parent
PROJECT_ROOT = WORKDIR.parents[1]
TARGET_BIN = PROJECT_ROOT / "target" / "release" / "mbf-fastq-processor"


def ensure_binary() -> None:
    subprocess.run(
        [
            "cargo",
            "build",
            "--release",
        ],
        cwd=PROJECT_ROOT,
        check=True,
    )


def render_config(repetitions: int) -> Path:
    input_file = WORKDIR / "input.toml"
    fq_path = (
        #Path('input_read1.fq.gz')
        PROJECT_ROOT / "test_cases"
        / "integration_tests"
        / "inspect_read1"
        / "input_read1.fq.zst"
        #/ "integration_tests/fastp_606/input_r2.fq.gz"
    ).resolve()
    files = [str(fq_path)] * repetitions
    files_literal = "[" + ", ".join(f"'{path}'" for path in files) + "]"
    toml = """
[input]
    read1 = {files}

[options]
    accept_duplicate_files = true
    thread_count = 1
    block_size=1000

[output]
    prefix = 'no_output'
    format = 'None'
""".format(files=files_literal)
    input_file.write_text(toml, encoding="utf-8")
    return input_file, len(toml)


def parse_alloc_metrics(stderr: str) -> dict[str, int] | None:
    for line in stderr.splitlines():
        if line.startswith("alloc: "):
            metrics: dict[str, int] = {}
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


def read_status_metrics(status_path: Path) -> tuple[int | None, int | None]:
    try:
        with status_path.open("r", encoding="utf-8") as status_file:
            vmhwm: int | None = None
            vmrss: int | None = None
            for line in status_file:
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


def run_process(
    config: Path,
    env_override: dict[str, str] | None,
) -> tuple[dict[str, int] | None, int | None, int | None]:
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
    max_rss_bytes: int | None = None
    final_rss_bytes: int | None = None
    while proc.poll() is None:
        vmhwm_bytes, vmrss_bytes = read_status_metrics(status_path)
        if vmhwm_bytes is not None:
            if max_rss_bytes is None or vmhwm_bytes > max_rss_bytes:
                max_rss_bytes = vmhwm_bytes
        if vmrss_bytes is not None:
            final_rss_bytes = vmrss_bytes
        time.sleep(0.001)

    _, stderr = proc.communicate()
    vmhwm_bytes, vmrss_bytes = read_status_metrics(status_path)
    if vmhwm_bytes is not None:
        if max_rss_bytes is None or vmhwm_bytes > max_rss_bytes:
            max_rss_bytes = vmhwm_bytes
    if vmrss_bytes is not None:
        final_rss_bytes = vmrss_bytes

    if proc.returncode != 0:
        raise RuntimeError(f"Command failed: {stderr}")

    alloc_metrics = parse_alloc_metrics(stderr)
    return alloc_metrics, max_rss_bytes, final_rss_bytes


def run_once(repetitions: int) -> tuple[dict[str, int] | None, int | None, int | None]:
    config, input_size = render_config(repetitions)
    alloc_metrics, _, _ = run_process(config, {"RUST_MEASURE_ALLOC": "1"})
    _, rss_peak, rss_final = run_process(config, None)
    return alloc_metrics, rss_peak, rss_final, input_size


def main() -> None:
    ensure_binary()
    rows: list[list[str]] = [[
        "rep",
        "alloc_bytes_max",
        "alloc_bytes_current",
        "max_rss_bytes",
        "final_rss_bytes",
        "bytes_max_per_rep",
        "input_size",
    ]]

    for rep in REPS:
        alloc, rss_peak, rss_final, input_size = run_once(rep)
        bytes_max = alloc.get("bytes_max") if alloc else None
        bytes_current = alloc.get("bytes_current") if alloc else None
        per_rep = f"{bytes_max / rep:.2f}" if bytes_max is not None else "NA"
        rows.append([
            str(rep),
            f"{bytes_max}" if bytes_max is not None else "NA",
            f"{bytes_current}" if bytes_current is not None else "NA",
            f"{rss_peak}" if rss_peak is not None else "NA",
            f"{rss_final}" if rss_final is not None else "NA",
            per_rep,
            str(input_size)
        ])

    col_widths = [max(len(row[idx]) for row in rows) for idx in range(len(rows[0]))]
    for row in rows:
        print("  ".join(value.rjust(col_widths[idx]) for idx, value in enumerate(row)))

    config_path = WORKDIR / "input.toml"
    if config_path.exists():
        config_path.unlink()
        ...


if __name__ == "__main__":
    main()
