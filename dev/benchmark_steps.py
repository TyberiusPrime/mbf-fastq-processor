#!/usr/bin/env python3
"""
Benchmark all transformation steps using hyperfine.

This script creates minimal benchmark configs for each step type and runs
hyperfine to compare their relative performance.
"""

import subprocess
import tempfile
import os
from pathlib import Path
import json

# Get the project root directory
PROJECT_ROOT = Path(__file__).parent.parent
TEST_DATA = PROJECT_ROOT / "test_cases" / "sample_data" / "paired_end" / "input_read1_1.fq"

# List of steps to benchmark with their minimal configurations
# Note: Use read1 = [...] not interleaved = [...] for single-file inputs
STEPS = [
    # Edits
    ("CutStart", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "CutStart"\nsegment = "read1"\nn = 5\n\n[output]\nprefix = "out"'),
    ("CutEnd", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "CutEnd"\nsegment = "read1"\nn = 5\n\n[output]\nprefix = "out"'),
    ("Truncate", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "Truncate"\nsegment = "read1"\nn = 50\n\n[output]\nprefix = "out"'),
    ("ReverseComplement", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "ReverseComplement"\nsegment = "read1"\n\n[output]\nprefix = "out"'),
    ("LowercaseSequence", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "LowercaseSequence"\nsegment = "read1"\n\n[output]\nprefix = "out"'),
    ("UppercaseSequence", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "UppercaseSequence"\nsegment = "read1"\n\n[output]\nprefix = "out"'),

    # Filters
    ("Head", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "Head"\nn = 500\n\n[output]\nprefix = "out"'),
    ("Skip", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "Skip"\nn = 100\n\n[output]\nprefix = "out"'),
    ("FilterEmpty", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "FilterEmpty"\nsegment = "read1"\n\n[output]\nprefix = "out"'),
    ("FilterSample", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "FilterSample"\nprobability = 0.5\nseed = 42\n\n[output]\nprefix = "out"'),

    # Validation
    ("ValidateSeq", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "ValidateSeq"\nsegment = "read1"\n\n[output]\nprefix = "out"'),
    ("ValidateQuality", '[input]\nread1 = ["{input}"]\n\n[[step]]\naction = "ValidateQuality"\nsegment = "read1"\n\n[output]\nprefix = "out"'),
]


def check_hyperfine():
    """Check if hyperfine is installed."""
    try:
        subprocess.run(["hyperfine", "--version"], capture_output=True, check=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def build_binary():
    """Build the binary in release mode."""
    print("Building binary in release mode...")
    subprocess.run(["cargo", "build", "--release"], cwd=PROJECT_ROOT, check=True)
    binary = PROJECT_ROOT / "target" / "release" / "mbf-fastq-processor"
    if not binary.exists():
        raise RuntimeError(f"Binary not found at {binary}")
    return str(binary)


def run_benchmarks(binary_path, read_count=100000, warmup=1, runs=5):
    """Run benchmarks using hyperfine."""

    if not check_hyperfine():
        print("Error: hyperfine is not installed.")
        print("Install it with: cargo install hyperfine")
        print("Or on Ubuntu: sudo apt install hyperfine")
        return

    # Create a temporary directory for configs
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        # Create config files
        config_files = []
        for step_name, config_template in STEPS:
            config_content = config_template.format(input=str(TEST_DATA))
            config_file = tmpdir / f"{step_name}.toml"
            config_file.write_text(config_content)
            config_files.append((step_name, config_file))

        # Build hyperfine commands
        commands = []
        names = []
        for step_name, config_file in config_files:
            cmd = f"{binary_path} benchmark {config_file} --read-count {read_count}"
            commands.extend(["--command-name", step_name, cmd])
            names.append(step_name)

        # Run hyperfine
        hyperfine_cmd = [
            "hyperfine",
            "--warmup", str(warmup),
            "--runs", str(runs),
            "--export-json", "benchmark_results.json",
            "--export-markdown", "benchmark_results.md",
        ] + commands

        print(f"\nRunning benchmarks with {read_count} reads per step...")
        print(f"Warmup runs: {warmup}, Benchmark runs: {runs}")
        print()

        subprocess.run(hyperfine_cmd, cwd=PROJECT_ROOT)

        print("\n" + "="*80)
        print("Benchmark results saved to:")
        print(f"  - benchmark_results.json")
        print(f"  - benchmark_results.md")
        print("="*80)


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Benchmark transformation steps")
    parser.add_argument("--read-count", type=int, default=100000,
                        help="Number of reads to process per step (default: 100000)")
    parser.add_argument("--warmup", type=int, default=1,
                        help="Number of warmup runs (default: 1)")
    parser.add_argument("--runs", type=int, default=5,
                        help="Number of benchmark runs (default: 5)")
    parser.add_argument("--skip-build", action="store_true",
                        help="Skip building the binary")

    args = parser.parse_args()

    if args.skip_build:
        binary_path = str(PROJECT_ROOT / "target" / "release" / "mbf-fastq-processor")
    else:
        binary_path = build_binary()

    run_benchmarks(binary_path, args.read_count, args.warmup, args.runs)


if __name__ == "__main__":
    main()
