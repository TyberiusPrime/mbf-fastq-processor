# Cookbook 01: Basic Quality Report

## Use Case

You have FastQ files from a sequencing run and want to generate comprehensive quality reports to assess:
- Read quality scores
- Base composition
- Read length distribution
- Duplicate read counts

This is typically the first step in any sequencing data analysis to understand data quality before downstream processing.

## What This Pipeline Does

1. Reads input FastQ file(s)
2. Generates a comprehensive quality report including:
   - Base quality statistics
   - Base distribution across positions
   - Read length distribution
   - Duplicate read counting
3. Outputs reports in both HTML (human-readable) and JSON (machine-readable) formats
4. Passes through all reads unchanged (no filtering)

## Input Files

- `input/sample_R1.fq` - Forward reads (Read 1) from paired-end sequencing

## Output Files

- `output_R1.fq` - Passed-through reads (identical to input)
- `output.report_initial.html` - HTML quality report
- `output.report_initial.json` - JSON quality report with detailed statistics

## Configuration Highlights

```toml
[[step]]
    action = 'Report'
    label = 'initial'
    count = true
    base_statistics = true
    duplicate_count_per_read = true
    length_distribution = true
```

The `Report` step generates comprehensive quality metrics. The `label` parameter determines the output filename suffix.

## When to Use This

- First analysis of new sequencing data
- Quality control before committing to expensive downstream analysis
- Comparing data quality across different sequencing runs
- Identifying potential issues (adapter contamination, quality drop-off, etc.)

## Running This Cookbook

```bash
cd cookbooks/01-basic-quality-report
mbf-fastq-processor process input.toml
```

Then open `output.report_initial.html` in your web browser to view the quality report.
