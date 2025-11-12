# Cookbook 04: PhiX Removal

## Use Case

You have Illumina PhiX spike-in sequences in your dataset and want to remove those contaminating reads before downstream analysis. PhiX is commonly added as a control to increase base diversity during sequencing runs.

## What This Pipeline Does

This cookbook demonstrates how to identify and remove PhiX contamination using k-mer counting:

1. **Count k-mers**: Uses `CalcKmers` to count how many 30-mers from each read match the PhiX genome
2. **Export data**: Saves k-mer counts to a TSV table for analysis
3. **Filter reads**: Removes reads with high PhiX k-mer counts (≥25 matching k-mers)

## Understanding the Approach

### K-mer Counting

The `CalcKmers` step counts how many k-mers (short subsequences of length k) from each read are present in the PhiX reference genome:

- **k = 30**: Uses 30-base-pair k-mers (longer k-mers = more specific matching)
- **canonical = true**: Counts both forward and reverse complement k-mers
- Reads heavily contaminated with PhiX will have many matching k-mers
- Clean reads will have few or zero matching k-mers

### Analyzing the Data: Histogram and Threshold Selection

After running the initial pipeline with `StoreTagsInTable`, examine the output TSV file to understand your data distribution:

```bash
# View the k-mer histogram with common command-line tools
# get second column, throw away header line, sort numerically, count unique occurrences
cut -f2 output_without_phix_kmer_analysis.tsv | tail -n +2 | sort -n | uniq -c

# alternativly, use awk for steps 1 & 2
awk 'NR>1 {print $2}' output_without_phix_kmer_analysis.tsv | sort -n | uniq -c
```

**Example output interpretation:**
```
  4   0    # 4 reads with 0 PhiX k-mers (clean reads)
  4  32    # 4 reads with 32 PhiX k-mers (PhiX contamination)
```

This bimodal distribution (two clear peaks) makes it easy to choose a threshold. Here, any value between 1-31 would work; we chose **25** as a conservative threshold.

**For larger datasets (first 300-1000 reads recommended):**
1. Run pipeline with `Head` step to sample reads: `[[step]]` with `action = "Head"` and `n = 300`
2. Export the table and analyze the distribution in your preferred tool 
3. Look for a clear separation between clean and contaminated reads
4. Choose a threshold in the "valley" between peaks

## Filtering Approaches

This cookbook demonstrates three equivalent ways to filter PhiX-contaminated reads:

### Approach 1: FilterByNumericTag (Simplest, shown in input.toml)

```toml
[[step]]
    action = 'FilterByNumericTag'
    in_label = "phix_kmer_count"
    min_value = 25
    keep_or_remove = 'remove'  # Remove reads with ≥25 k-mers
```

**Best for:** Direct numeric filtering with a single threshold.

### Approach 2: EvalExpression + Demultiplex (Most Flexible)

```toml
# Create a boolean tag based on k-mer count threshold
[[step]]
    action = "EvalExpression"
    expression = "phix_kmer_count >= 25"
    out_label = "is_phix"
    result_type = "bool"

# Separate clean reads from PhiX-contaminated reads
# For boolean tags, Demultiplex creates files with pattern: {prefix}_{label}={value}_{segment}.fq
[[step]]
    action = "Demultiplex"
    in_label = "is_phix"

# No [barcodes] section needed for boolean demultiplexing!
# Output files will be:
# - output_is_phix=true_read1.fq  (PhiX-contaminated reads)
# - output_is_phix=false_read1.fq (clean reads)
```

**Best for:**
- Separating reads into multiple output files (contaminated vs. clean)
- Complex boolean expressions combining multiple criteria
- When you want to keep both categories for quality control

**Alternative using FilterByTag:**
After `EvalExpression`, you could also use `FilterByTag` to keep/remove reads based on the boolean tag:
```toml
[[step]]
    action = "FilterByTag"
    in_label = "is_phix"
    keep_or_remove = "Remove"  # Removes reads where is_phix = true
```

## Usage

### Using the Standard Approach (FilterByNumericTag)

```bash
# Run the pipeline
cd 04-phiX-removal
mbf-fastq-processor input.toml

# Check the results
head output_without_phix_kmer_analysis.tsv      # View k-mer counts
grep -c "^@" output_without_phix_read1.fq       # Count output reads (should be 4)
```

### Using the Demultiplex Approach (Alternative)

To try the EvalExpression + Demultiplex approach that separates reads into two files:

```bash
# Run the alternative pipeline
mbf-fastq-processor input_demultiplex.toml

# Check the results
head output_kmer_analysis.tsv                     # View k-mer counts
grep -c "^@" output_is_phix=false_read1.fq        # Count clean reads (should be 4)
grep -c "^@" output_is_phix=true_read1.fq         # Count PhiX reads (should be 4, have phiX in name.)
```

## Expected Results

With the provided sample data:
- **Input:** 8 reads (4 PhiX, 4 clean)
- **Output:** 4 clean reads (PhiX reads removed)
- **Table:** Shows k-mer counts for all input reads (32 for PhiX, 0 for clean)

## Customization

Adjust these parameters based on your data:
- **k**: Larger values (e.g., 35) = more specific; smaller values (e.g., 21) = more sensitive
- **min_count**: In `CalcKmers`, filters rare k-mers from the reference (default: 1)
- **threshold**: Adjust `min_value` in `FilterByNumericTag` based on your histogram analysis
