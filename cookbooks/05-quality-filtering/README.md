# Cookbook 05: Quality Filtering

## Use Case

You have sequencing data with varying quality and want to remove low-quality reads before downstream analysis. Poor quality reads can introduce errors in variant calling, assembly, and other analyses.

## What This Pipeline Does

This cookbook demonstrates quality-based filtering using expected error calculation:

1. **Calculate Expected Errors**: Uses `CalcExpectedError` to compute the expected number of base call errors per read based on quality scores
2. **Filter Low-Quality Reads**: Uses `FilterByNumericTag` to remove reads exceeding an error threshold
3. **Generate Reports**: Creates quality reports before and after filtering to show improvement

## Understanding Expected Error

Expected error (EE) is a more nuanced quality metric than average quality score:

- **Formula**: EE = sum of error probabilities across all bases
- **Example**: A read with quality scores Q30, Q30, Q20, Q30 has EE ≈ 0.001 + 0.001 + 0.01 + 0.001 = 0.013
- **Interpretation**: Lower EE = higher confidence read
- **Threshold**: Common threshold is EE ≤ 1.0 (expect ≤1 error per read)

Quality scores and error probabilities:
- Q20 = 1% error rate (0.01 probability)
- Q30 = 0.1% error rate (0.001 probability)
- Q40 = 0.01% error rate (0.0001 probability)

## Configuration Highlights

```toml
[[step]]
    action = 'Report'
    label = 'initial'
    n = 10000  # Sample first 10k reads for speed

[[step]]
    action = 'CalcExpectedError'
    segment = 'read1'
    out_label = 'expected_error'

[[step]]
    action = 'FilterByNumericTag'
    in_label = "expected_error"
    max_value = 1.0  # Keep reads with EE ≤ 1.0
    keep_or_remove = 'keep'

[[step]]
    action = 'Report'
    label = 'filtered'
    n = 10000
```

## Expected Results

With the provided sample data:
- **Input:** 10 reads (5 high-quality, 5 low-quality)
- **Output:** 5 high-quality reads (low-quality reads removed)
- **Reports:** Before/after quality comparison showing improved quality metrics

## Customization

Adjust the filtering threshold based on your application:
- **Strict filtering (EE ≤ 0.5)**: For applications requiring highest accuracy (variant calling, metagenomics)
- **Standard filtering (EE ≤ 1.0)**: General-purpose filtering (shown in this cookbook)
- **Relaxed filtering (EE ≤ 2.0)**: When read depth is more important than individual read accuracy

You can also filter by other quality metrics:
- `CalcQualifiedBases`: Count bases above a quality threshold
- `FilterByNumericTag` with `min_value`: Keep reads with enough high-quality bases

## When to Use This

- After initial quality assessment (see Cookbook 01)
- Before alignment or assembly
- When downstream tools are sensitive to sequencing errors
- To reduce computational burden by removing unreliable data

## Downstream Analysis

After quality filtering:
1. **Alignment** to reference genome (BWA, Bowtie2, STAR)
2. **Variant calling** with higher confidence
3. **Assembly** with cleaner input data
4. **Quantification** with reduced noise
