+++
+++

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

## When to Use This

- First analysis of new sequencing data
- Quality control before committing to expensive downstream analysis
- Comparing data quality across different sequencing runs
- Identifying potential issues (adapter contamination, quality drop-off, etc.)



## Download

[Download 01-basic-quality-report.tar.gz](../../../../../cookbooks/01-basic-quality-report.tar.gz) for a complete, runnable example including expected output files.

## Configuration File

```toml
[input]
    # Single-end reads for this example
    # For paired-end data, you would also include: read2 = 'input/sample_R2.fq'
    read1 = 'input/sample_R1.fq'

[[step]]
    # Generate a comprehensive quality report
    action = 'Report'
    label = 'initial'

    # Count total number of reads
    count = true

    # Analyze base quality scores and GC content
    base_statistics = true

    # Count duplicate reads (identical sequences)
    duplicate_count_per_read = true

    # Analyze the distribution of read lengths
    length_distribution = true

[output]
    # Output prefix for all files
    prefix = 'output'

    # Generate both HTML and JSON reports
    report_html = true
    report_json = true

    # Output format (FASTQ = uncompressed FASTQ format)
    format = "FASTQ"
```
