# Cookbook 08: Read Length Filtering and Truncation

## Use Case

You have sequencing data with variable read lengths and need to:
- Remove reads that are too short (may align poorly or represent artifacts)
- Remove reads that are too long (may indicate technical issues)
- Truncate all reads to a uniform length (required by some downstream tools)

Read length filtering is important for:
- Quality control after adapter trimming
- Preparing data for tools that require uniform read lengths
- Removing degraded or artifactual sequences

## What This Pipeline Does

This cookbook demonstrates comprehensive read length management:

1. **Calculate Read Length**: Uses `CalcLength` to tag each read with its length
2. **Filter by Minimum Length**: Uses `FilterByNumericTag` to remove short reads
3. **Filter by Maximum Length**: Uses `FilterByNumericTag` to remove long reads
4. **Truncate to Uniform Length**: Uses `Truncate` to trim all reads to the same size
5. **Generate Reports**: Creates before/after statistics

## Understanding Read Length

**Why read length matters:**
- **Too short**: May align to multiple locations (multimapping)
- **Too long**: May indicate incomplete adapter trimming or concatenated sequences
- **Variable length**: Some tools (e.g., older aligners) require uniform lengths
- **Optimal range**: Depends on application (typically 25-150bp for RNA-seq)

**Common scenarios:**
- **After adapter trimming**: Reads become variable length; filter out very short ones
- **Amplicon sequencing**: Expected length range is tight (e.g., 250-280bp)
- **Small RNA**: Keep short reads (18-30bp) while filtering longer contamination
- **Assembly**: Longer reads generally better, but quality matters more

## Configuration Highlights

```toml
[[step]]
    # Calculate the length of each read
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'length'

[[step]]
    # Remove reads shorter than 30bp
    # Short reads often align poorly or represent artifacts
    action = 'FilterByNumericTag'
    in_label = 'length'
    min_value = 30
    keep_or_remove = 'keep'

[[step]]
    # Remove reads longer than 150bp
    # Unusually long reads may indicate technical issues
    action = 'FilterByNumericTag'
    in_label = 'length'
    max_value = 150
    keep_or_remove = 'keep'

[[step]]
    # Truncate all remaining reads to exactly 100bp
    # Some tools require uniform read lengths
    action = 'Truncate'
    segment = 'read1'
    length = 100
```

## Input Files

- `input/variable_length_R1.fq` - Reads with varying lengths (20-160bp)

## Output Files

- `output_read1.fq` - Reads filtered to 30-150bp range and truncated to 100bp

## Expected Results

With the provided sample data:
- **Input:** 10 reads with lengths ranging from 20 to 160bp
- **After min filter:** Removes reads < 30bp (e.g., 20bp, 25bp reads removed)
- **After max filter:** Removes reads > 150bp (e.g., 160bp read removed)
- **After truncate:** All remaining reads are exactly 100bp

## Workflow Details

**Example transformations:**

| Read ID | Original Length | After Min Filter | After Max Filter | After Truncate |
|---------|----------------|------------------|------------------|----------------|
| READ1   | 25bp          | Removed          | -                | -              |
| READ2   | 40bp          | Kept             | Kept             | → 40bp (kept) |
| READ3   | 100bp         | Kept             | Kept             | → 100bp       |
| READ4   | 120bp         | Kept             | Kept             | → 100bp       |
| READ5   | 160bp         | Kept             | Removed          | -              |

**Note on Truncate behavior:**
- If read is longer than target: trims to target length
- If read is shorter than target: keeps original length (does not pad)
- To enforce exact length, filter first: `min_value = target_length`

## Customization

Adjust parameters based on your application:

**RNA-seq (general):**
```toml
min_value = 25  # Minimum for reliable alignment
max_value = 200 # Filter abnormally long reads
# No truncation usually needed
```

**Amplicon sequencing (expected 250bp):**
```toml
min_value = 240  # Tight range around expected
max_value = 260
# Truncate to 250 for uniformity
```

**Small RNA-seq (miRNA):**
```toml
min_value = 18   # Shortest mature miRNA
max_value = 30   # Longest miRNA + some tolerance
# No truncation
```

**ChIP-seq or ATAC-seq:**
```toml
min_value = 25
max_value = 150
# Optional truncation to reduce file size
```

**Uniform length required:**
```toml
# First filter to ensure all reads are at least target length
min_value = 100
# Then truncate to exact length
[[step]]
    action = 'Truncate'
    segment = 'read1'
    length = 100
```

## When to Use This

- **After adapter trimming** to remove reads that became too short
- **Quality control** to filter abnormally long/short reads
- **Before tools** that require uniform read lengths
- **Amplicon analysis** to enforce expected size range
- **Small RNA analysis** to select specific size classes

## Alternative Approaches

**Using CutStart/CutEnd instead of Truncate:**
```toml
# Remove first 10bp and last 10bp
[[step]]
    action = 'CutStart'
    segment = 'read1'
    n = 10

[[step]]
    action = 'CutEnd'
    segment = 'read1'
    n = 10
```

**Filtering paired-end reads by combined length:**
```toml
# Calculate both read lengths
[[step]]
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'len1'

[[step]]
    action = 'CalcLength'
    segment = 'read2'
    out_label = 'len2'

# Use EvalExpression to filter based on combined length
[[step]]
    action = 'EvalExpression'
    expression = 'len1 + len2 >= 50'
    out_label = 'long_enough'
    result_type = 'bool'

[[step]]
    action = 'FilterByTag'
    in_label = 'long_enough'
    keep_or_remove = 'keep'
```

## Downstream Analysis

After length filtering:
1. **Verify length distribution** with quality control tools (FastQC)
2. **Alignment** to reference genome (should have better mapping rates)
3. **Quantification** or other downstream analysis
4. **Compare** results with/without filtering to assess impact

## Quality Metrics

Monitor these metrics after length filtering:
- **Percentage reads retained**: Should retain most reads (>80% typical)
- **Mean read length**: Should match expected for your protocol
- **Mapping rate**: Often improves after filtering too-short reads
- **Alignment quality**: Fewer multimapping reads
