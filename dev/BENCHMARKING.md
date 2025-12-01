# Benchmarking Guide

This document describes how to benchmark individual transformation steps in mbf-fastq-processor.

## Overview

The benchmarking system allows you to profile the performance of individual transformation steps. This is useful for:
- Understanding which steps are most computationally expensive
- Comparing the relative performance of different steps
- Optimizing pipeline configurations
- Identifying performance regressions

## Quick Start

### 1. Using the CLI Benchmark Command

The simplest way to benchmark steps is using the `benchmark` subcommand:

```bash
# Benchmark all steps in a config file
cargo build --release
./target/release/mbf-fastq-processor benchmark config.toml

# Specify number of reads to process
./target/release/mbf-fastq-processor benchmark config.toml --read-count 500000
```

Example config file:
```toml
[input]
interleaved = ["test_cases/sample_data/paired_end/input_read1.fq"]

[[step]]
action = "CutStart"
segment = "read1"
length = 5

[[step]]
action = "CalcGCContent"
segment = "read1"
tag = "gc"

[output]
prefix = "out"
```

### 2. Using the Hyperfine Benchmarking Script

For comprehensive benchmarking of all step types with statistical analysis:

```bash
# Install hyperfine if not already installed
cargo install hyperfine
# or on Ubuntu: sudo apt install hyperfine

# Run the benchmark suite
python3 dev/benchmark_steps.py

# Customize parameters
python3 dev/benchmark_steps.py --read-count 200000 --runs 10 --warmup 2

# Skip rebuilding if binary is already built
python3 dev/benchmark_steps.py --skip-build
```

This will:
- Build the release binary (unless `--skip-build` is specified)
- Create minimal benchmark configs for each step type
- Run hyperfine to benchmark each step
- Generate results in JSON and Markdown formats

### Options

**`benchmark_steps.py` options:**
- `--read-count N`: Number of reads to process per step (default: 100000)
- `--warmup N`: Number of warmup runs before benchmarking (default: 1)
- `--runs N`: Number of benchmark runs per step (default: 5)
- `--skip-build`: Skip building the binary (use existing release build)

**CLI benchmark options:**
- `--read-count N` or `-n N`: Number of reads to process (default: 1000000)

## How It Works

### Benchmark Function

The benchmark function (`src/benchmark.rs`):

1. **Reads a sample block** from the input files
2. **Clones the block** as many times as needed to reach the target read count
3. **Processes all blocks** through the specified step
4. **Measures time** and calculates throughput (reads/second)

This approach ensures:
- **Isolation**: Each step is profiled independently
- **Repeatability**: Same input data for consistent results
- **Focus**: Only the step's logic is measured, not I/O overhead

### Hyperfine Integration

The `benchmark_steps.py` script uses [hyperfine](https://github.com/sharkdp/hyperfine) to:
- Run multiple iterations for statistical accuracy
- Compare step performance side-by-side
- Export results in JSON and Markdown formats
- Detect outliers and calculate confidence intervals

## Interpreting Results

### CLI Output

```
Step                           |        Reads |       Time |         Reads/s
--------------------------------------------------------------------------------
CutStart                       |      1000000 |      0.234 s |       4273504 reads/s
CalcGCContent                  |      1000000 |      0.567 s |       1763668 reads/s
```

This shows:
- **Step name**: The transformation being benchmarked
- **Reads**: Number of reads processed
- **Time**: Wall clock time taken
- **Reads/s**: Throughput in reads per second

### Hyperfine Output

Hyperfine provides statistical summaries including:
- **Mean time**: Average execution time across runs
- **Range**: Min and max times observed
- **Std dev**: Standard deviation (lower is better for consistency)
- **Relative performance**: How much faster/slower compared to the fastest step

Example output:
```
Benchmark 1: CutStart
  Time (mean ± σ):     234.5 ms ±   3.2 ms    [User: 220.1 ms, System: 14.4 ms]
  Range (min … max):   230.1 ms … 239.8 ms    5 runs

Summary
  'CutStart' ran
    1.42 ± 0.02 times faster than 'CalcGCContent'
    2.15 ± 0.03 times faster than 'CalcComplexity'
```

## Benchmark Results Files

After running `benchmark_steps.py`, two files are generated:

### benchmark_results.json

Machine-readable JSON format containing:
- Command executed for each step
- Mean, min, max, median, stddev times
- All individual measurements
- System information

Use for:
- Automated performance tracking
- CI/CD integration
- Historical comparison

### benchmark_results.md

Human-readable Markdown table:
```markdown
| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `CutStart` | 234.5 ± 3.2 | 230.1 | 239.8 | 1.00 |
| `CalcGCContent` | 333.2 ± 4.5 | 327.8 | 340.1 | 1.42 ± 0.02 |
```

Use for:
- Documentation
- Pull request descriptions
- Performance reports

## Performance Tips

### Optimizing Read Count

- **Too low** (<10,000): Results may be dominated by initialization overhead
- **Too high** (>10M): Takes longer to run without adding much statistical value
- **Recommended**: 100,000 - 1,000,000 reads for most steps

### Benchmark Environment

For reliable results:
- Close other applications
- Use release builds (`cargo build --release`)
- Run on consistent hardware
- Consider CPU thermal throttling for long benchmarks
- Run multiple iterations (hyperfine does this automatically)

### Comparing Results

When comparing benchmarks:
- Use the same input data
- Use the same read count
- Run on the same machine
- Consider relative performance, not just absolute times
- Focus on steps that take >5% of total pipeline time

## Example: Profiling a Slow Pipeline

If your pipeline is slow:

1. **Benchmark all steps** in your config:
   ```bash
   ./target/release/mbf-fastq-processor benchmark my_pipeline.toml
   ```

2. **Identify bottlenecks**: Steps with lowest reads/s are candidates for optimization

3. **Compare alternatives**: If a step is slow, try alternative approaches:
   - Different parameters
   - Simpler transformations
   - Removing unnecessary steps

4. **Verify improvements**: Re-run benchmark to confirm optimizations help

## Adding New Steps to Benchmark Suite

To add a new step to `benchmark_steps.py`:

1. Add an entry to the `STEPS` list:
   ```python
   ("MyNewStep", '[input]\ninterleaved = ["{input}"]\n\n[[step]]\naction = "MyNewStep"\n... parameters ...\n\n[output]\nprefix = "out"'),
   ```

2. Run the benchmark suite to include your step:
   ```bash
   python3 dev/benchmark_steps.py
   ```

## CI/CD Integration

You can integrate benchmarking into CI/CD:

```bash
# In your CI pipeline
python3 dev/benchmark_steps.py --runs 3 --read-count 50000

# Check for performance regressions
# Compare benchmark_results.json with previous runs
```

## Troubleshooting

### "Binary not found" error
```bash
cargo build --release
```

### "No reads found in input files"
Check that your input files exist and contain valid FASTQ data.

### Hyperfine not found
```bash
cargo install hyperfine
```

### Inconsistent results
- Ensure no other heavy processes are running
- Increase `--runs` for more stable statistics
- Check CPU temperature/throttling
