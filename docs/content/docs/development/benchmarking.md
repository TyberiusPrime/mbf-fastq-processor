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

### Analyzing Results

```bash
# Comprehensive analysis with visualizations
cd dev
python3 enhanced_benchmark_analysis.py
```

## Benchmark Architecture

### Benchmark Mode

The processor includes a special benchmark mode that:
- Disables regular output
- Runs in a temporary directory  
- Repeats the first molecule block until `molecule_count` is exceeded
- Focuses on step performance rather than I/O

Example configuration:
```toml
[input]
    read1 = "test_cases/sample_data/large_sample.fq.gz"

[benchmark]
    enable = true
    molecule_count = 100_000

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

### Step Categories

Benchmarks are organized by function:

| Category | Steps | Typical Performance |
|----------|-------|-------------------|
| **Filtering** | `Head`, `FilterEmpty` | 110-111ms |
| **Quality Control** | `Progress`, `Report` | 113-249ms |
| **Calculation** | `CalcLength`, `CalcGCContent` | 110-113ms |
| **Sequence Manipulation** | `CutStart`, `ReverseComplement` | 108-116ms |

## Performance Analysis

### Current Baseline Performance

Based on 100,000 molecules from a 14MB FASTQ file:

```
Step                Performance    Category
----                -----------    --------
CutStart           107.5ms (1.00x)  Sequence Manipulation
CalcLength         110.3ms (1.03x)  Calculation  
FilterEmpty        110.5ms (1.03x)  Filtering
Head               111.3ms (1.04x)  Filtering
Progress           112.6ms (1.05x)  Quality Control
CalcGCContent      112.9ms (1.05x)  Calculation
ReverseComplement  116.2ms (1.08x)  Sequence Manipulation
Report             248.7ms (2.31x)  Quality Control ⚠️
```

### Performance Insights

- **Most operations** perform consistently at ~107-116ms
- **Report step** is the primary optimization target (2.3x slower)
- **Quality Control** category shows the highest variance
- **Filtering operations** are among the fastest and most predictable

## Adding New Benchmarks

### 1. Define the Benchmark

Add your step to `mbf-fastq-processor/benches/simple_benchmarks.rs`:

```rust
let steps = vec![
    // ... existing steps ...
    ("YourNewStep", r#"param1 = "value"
    param2 = 42
    segment = "read1""#),
];
```

### 2. Handle Tag Dependencies

Ensure proper tag management:
```rust
// For steps that create tags - automatically handled
("CalcLength", r#"out_label = "length"
    segment = "read1""#),  // ForgetAllTags added automatically

// For steps that need existing tags - create dependency
("FilterByNumericTag", r#"in_label = "existing_tag"
    min_value = 50"#),
```

### 3. Update Documentation

Add expected performance characteristics to:
- `dev/benchmarks/README.md`
- This documentation section
- Performance baselines

## Optimization Workflow

### 1. Identify Targets

```bash
# Run analysis to identify slow steps
python3 dev/benchmarks/enhanced_benchmark_analysis.py
```

Look for:
- Steps with >1.2x relative performance
- High variance in timing
- Category outliers

### 2. Profile Individual Steps

```bash
# Create isolated benchmark for specific step
cargo bench --bench simple_benchmarks YourStep
```

### 3. Detailed Profiling

For deeper analysis:
```bash
# Generate flamegraph for specific step
cargo flamegraph --bench simple_benchmarks -- YourStep

# Memory profiling
cargo bench --bench simple_benchmarks YourStep -- --memory-profile
```

### 4. Validate Improvements

```bash
# Before/after comparison
git checkout baseline_branch
cargo bench --bench simple_benchmarks > baseline.txt

git checkout optimized_branch  
cargo bench --bench simple_benchmarks > optimized.txt

# Compare results
python3 dev/benchmarks/compare_results.py baseline.txt optimized.txt
```

## Continuous Integration

### Performance Regression Detection

```bash
# In CI pipeline
cargo bench --bench simple_benchmarks -- --output-format json > results.json

# Check for regressions (implement as needed)
python3 dev/benchmarks/check_regressions.py results.json
```

### Performance Tracking

Track performance over time:
- Store benchmark results with commit hashes
- Generate performance trend reports
- Alert on significant regressions (>10% slowdown)

## Troubleshooting

### Common Issues

**File not found errors**
```bash
# Ensure running from project root
pwd  # Should show /path/to/mbf-fastq-processor
cargo bench --bench simple_benchmarks
```

**Compilation failures**
- Check that benchmark mode syntax is compatible
- Verify all dependencies in `Cargo.toml`
- Ensure step configurations are valid

**Inconsistent results**
- Run on idle system for consistent results
- Increase sample size for more stable measurements
- Check for background processes affecting performance

### Performance Analysis Tips

- **Focus on categories**: Look for entire categories performing poorly
- **Check error ranges**: High variance may indicate algorithmic issues
- **Compare relative performance**: Absolute times vary by system
- **Profile incrementally**: Start with broad categories, drill down to specific optimizations

## Future Enhancements

Planned benchmarking improvements:
- Memory allocation profiling
- Multi-threaded performance analysis  
- Scaling behavior with different molecule counts
- Comparison with alternative implementations
- Integration with CI for automated performance tracking