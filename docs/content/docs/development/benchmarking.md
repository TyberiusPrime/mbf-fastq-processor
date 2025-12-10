---
weight: 130
---

# Performance Benchmarking

mbf-fastq-processor includes a comprehensive benchmarking suite for measuring and analyzing the performance of individual transformation steps.

## Overview

The benchmarking system uses:
- **Criterion**: Industry-standard Rust benchmarking framework
- **Benchmark mode**: Built-in mode that focuses on step performance while minimizing I/O overhead
- **Automated analysis**: Python tools for comprehensive performance analysis and visualization

## Quick Start

### Running Benchmarks

```bash
# Run all step benchmarks
cargo bench --bench simple_benchmarks

# Run with custom settings
cargo bench --bench simple_benchmarks -- --sample-size 20 --warm-up-time 2
```


## Benchmark Architecture

### Benchmark Mode

The processor includes a special benchmark mode that:
- Disables regular output
- Runs in a temporary directory 
- Repeats the block of molecules until `molecule_count` is exceeded
- Focuses on step performance rather than I/O

Example configuration:
```toml
[input]
    read1 = "test_cases/sample_data/large_sample.fq.gz"

[benchmark]
    enable = true
    molecule_count = 100_000
    quiet = false # defaults to false

[[step]]
    action = "CalcLength"
    out_label = "length"
    segment = "read1"

[[step]]
    action = "ForgetAllTags"  # Consume the tag

[output]
    prefix = "benchmark_output"
    format = "FASTQ"
```


## Adding New Benchmarks

### 1. Define the Benchmark

Add your step to `mbf-fastq-processor/benches/simple_benchmarks.rs`$



