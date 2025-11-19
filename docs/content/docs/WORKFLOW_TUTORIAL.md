---
weight: 25
title: "Complete Workflow Tutorial"
bookToc: true
---

# Complete Workflow Tutorial: From First Reads to Final Processing

This tutorial demonstrates the complete workflow of using mbf-fastq-processor to develop and refine a processing pipeline. Unlike the cookbooks (which show specific configurations), this tutorial walks you through the iterative process of:

1. **Exploring** your data to understand its structure
2. **Generating reports** to identify quality issues and contaminants
3. **Finding** adapters, polyA tails, and constant regions
4. **Developing** your pipeline interactively with rapid feedback
5. **Verifying** the final results

## Table of Contents

- [The Scenario](#the-scenario)
- [Step 1: Initial Data Exploration](#step-1-initial-data-exploration)
- [Step 2: Generate an Initial Quality Report](#step-2-generate-an-initial-quality-report)
- [Step 3: Analyzing the Report](#step-3-analyzing-the-report)
- [Step 4: Interactive Pipeline Development](#step-4-interactive-pipeline-development)
- [Step 5: Full Processing](#step-5-full-processing)
- [Step 6: Final Verification](#step-6-final-verification)
- [Best Practices](#best-practices)

---

## The Scenario

You've received FastQ files from a custom 3' RNA-seq protocol. The library structure includes:
- **First 8 bases**: Random UMI for deduplication
- **Next 6 bases**: Constant region "TACGAT" (from the primer)
- **Remaining sequence**: cDNA from the 3' end of transcripts
- **Potential contamination**: Illumina adapters and polyA tails may be present

Your goal: Clean and process these reads for downstream alignment and quantification.

---

## Step 1: Initial Data Exploration

Before building a processing pipeline, always start by examining a small sample of your data.

### Create an exploration config

Create `explore.toml`:

```toml
[input]
    read1 = 'raw_data/sample_R1.fastq.gz'

# Take just the first 10,000 reads for quick exploration
[[step]]
    action = 'Head'
    n = 10000

# Sample 15 reads randomly to inspect
[[step]]
    action = 'FilterReservoirSample'
    n = 15
    seed = 42

# Display the sampled reads in the console
[[step]]
    action = 'Inspect'
    n = 15

[output]
    # We don't need to save these - just inspect them
    format = 'None'
```

### Run exploration

```bash
mbf-fastq-processor process explore.toml
```

**What you're looking for:**
- Read length distribution
- Visual patterns in the sequences
- Quality score patterns
- Any obvious adapter sequences

**Example output you might see:**

```
Read 1:
@READ_001
ATCGATCGTACGATACTGTACTGTACTGTACAAAAAAAA...
+
IIIIIIIIIIIIIIIIIIIIIIII################...
        ^^^^^^      polyA tail visible
  ^^^^^^
  UMI?

Read 2:
@READ_002
GCTAGCTAGTACGATGCTGATCGATCGATCGATCGGAAGAGCACA...
+
IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII##########...
                                    ^^^^^^^^^^
                                    Illumina adapter?
```

From this quick inspection, you can already see:
- Potential 8bp UMI at the start
- A constant region after the UMI
- PolyA tails in some reads
- Illumina adapter contamination

---

## Step 2: Generate an Initial Quality Report

Now generate a comprehensive report to quantify what you observed.

### Create a report config

Create `initial_report.toml`:

```toml
[input]
    read1 = 'raw_data/sample_R1.fastq.gz'

# Process a reasonable sample for statistics
# For full datasets, you might use 1-5 million reads
[[step]]
    action = 'Head'
    n = 100000

[[step]]
    action = 'Report'
    name = 'initial'

    # Basic statistics
    count = true
    base_statistics = true
    length_distribution = true

    # Duplication analysis
    duplicate_count_per_read = true

    # Look for specific sequences
    count_oligos = [
        'AAAAAA',                                    # PolyA tail
        'TTTTTT',                                    # PolyT tail
        'GATCGGAAGAGCACACGTCTGAACTCCAGTCAC',        # Illumina TruSeq adapter
        'AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT',        # Illumina small RNA adapter
        'TACGAT',                                    # Our expected constant region
    ]

[output]
    prefix = 'initial_report'
    report_html = true
    report_json = true
    format = 'None'  # We only want the report, not the reads
```

### Run the report

```bash
mbf-fastq-processor process initial_report.toml
```

This generates:
- `initial_report.report_initial.html` - Human-readable report
- `initial_report.report_initial.json` - Machine-readable data

---

## Step 3: Analyzing the Report

Open `initial_report.report_initial.html` in your browser. Look for:

### 3.1 Read Quality

**Base quality plot**:
- Do quality scores drop off toward the end? → Consider quality trimming
- Low quality at the start? → May indicate issues with UMI or constant region

**Quality distribution**:
- What percentage of reads have good quality (Q30+)?
- Are there quality issues that would benefit from filtering?

### 3.2 Sequence Content

**Base composition**:
- Check for expected patterns (e.g., bias in first 8bp due to UMI randomness)
- Look for unexpected patterns that might indicate contamination

**Oligo counts** (from your `count_oligos` list):

```
Oligo counts:
- AAAAAA (polyA):        23,456 reads (23.5%)
- TTTTTT (polyT):            34 reads (0.03%)
- GATCGGAAGAGCACACGTCTGAACTCCAGTCAC (Illumina TruSeq):
                          1,234 reads (1.2%)
- AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT (Illumina small RNA):
                              0 reads (0%)
- TACGAT (constant):     98,765 reads (98.8%)
```

**Key findings from this example:**
1. ✓ 98.8% of reads contain our expected constant region "TACGAT"
2. ⚠️ 23.5% have polyA tails (need trimming)
3. ⚠️ 1.2% have Illumina TruSeq adapters (need removal)

### 3.3 Length Distribution

```
Length distribution:
Min: 35bp
Max: 151bp
Mean: 87bp
Median: 92bp

Peak at ~15bp (adapter-only reads) → Remove these
Peak at ~90bp (good reads) → Expected
```

---

## Step 4: Interactive Pipeline Development

Now that you know what to fix, use **interactive mode** to rapidly develop your processing pipeline.

Interactive mode:
- Watches your config file for changes
- Auto-processes a small sample
- Shows results immediately
- Perfect for iterative development

### 4.1 Create your processing config

Create `process.toml`:

```toml
[input]
    read1 = 'raw_data/sample_R1.fastq.gz'

# Step 1: Extract the UMI
[[step]]
    action = 'ExtractRegions'
    out_label = 'umi'
    regions = [{segment = 'read1', start = 0, length = 8}]

# Step 2: Store UMI in comment for later deduplication
[[step]]
    action = 'StoreTagInComment'
    in_label = 'umi'

# Step 3: Find and tag the constant region
[[step]]
    action = 'FindConstantRegion'
    out_label = 'constant_region'
    segment = 'read1'
    sequence = 'TACGAT'
    max_mismatches = 1
    start_search_after = 6  # Start after UMI

# Step 4: Trim everything before and including the constant region
[[step]]
    action = 'TrimAtTag'
    in_label = 'constant_region'
    segment = 'read1'
    keep = 'After'

# Step 5: Remove Illumina adapters
[[step]]
    action = 'FindAndRemove3PrimeAdapter'
    segment = 'read1'
    adapter = 'GATCGGAAGAGCACACGTCTGAACTCCAGTCAC'
    min_overlap = 5
    max_error_rate = 0.1

# Step 6: Trim polyA tails
[[step]]
    action = 'TrimPolyA'
    segment = 'read1'
    min_length = 5
    min_purity = 0.8

# Step 7: Filter out reads that are too short after trimming
[[step]]
    action = 'FilterByLength'
    segment = 'read1'
    min_length = 20
    keep_or_remove = 'Keep'

[output]
    prefix = 'processed'
    format = 'Fastq'
    compression = 'Gzip'
    report_html = true
    report_json = true
```

### 4.2 Start interactive mode

```bash
mbf-fastq-processor interactive process.toml
```

You'll see output like:

```
🔍 Watching: process.toml
📊 Auto-processing with Head(10000) → Sample(15) → Inspect(15)

Processing...
✓ Processed 10,000 reads
✓ Sampled 15 reads for inspection

Sample Output:
─────────────────────────────────────────────
Read 1:
@READ_001 umi:ATCGATCG
ACTGTACTGTACTGTAC
+
IIIIIIIIIIIIIIIII

Read 2:
@READ_002 umi:GCTAGCTA
GCTGATCGATCGATCG
+
IIIIIIIIIIIIIIII
─────────────────────────────────────────────

📈 Quick Stats:
- Input reads: 10,000
- Output reads: 9,234 (92.3% pass)
- Avg length: 62bp (was 87bp before trimming)

Watching for changes... (Ctrl+C to exit)
```

### 4.3 Iterate on your pipeline

Now you can edit `process.toml` and see results immediately!

**Example iteration 1**: The constant region isn't being found in some reads

```toml
# Increase mismatch tolerance
[[step]]
    action = 'FindConstantRegion'
    out_label = 'constant_region'
    segment = 'read1'
    sequence = 'TACGAT'
    max_mismatches = 2  # ← Changed from 1
    start_search_after = 6
```

Save → Results update automatically!

**Example iteration 2**: Too many short reads are passing

```toml
# Increase minimum length
[[step]]
    action = 'FilterByLength'
    segment = 'read1'
    min_length = 30  # ← Changed from 20
    keep_or_remove = 'Keep'
```

Save → Results update automatically!

**Example iteration 3**: Add another adapter you found in the report

```toml
# Add another adapter removal step
[[step]]
    action = 'FindAndRemove3PrimeAdapter'
    segment = 'read1'
    adapter = 'AGATCGGAAGAGC'
    min_overlap = 5
    max_error_rate = 0.1
```

Save → Results update automatically!

### 4.4 When you're happy

Once the sample output looks good:
1. Exit interactive mode (Ctrl+C)
2. Proceed to full processing

---

## Step 5: Full Processing

Now run your refined pipeline on the complete dataset.

### 5.1 Remove the Head step

Edit `process.toml` and remove (or comment out) the `Head` step if you added one:

```toml
[input]
    read1 = 'raw_data/sample_R1.fastq.gz'

# Removed Head step - process all reads

[[step]]
    action = 'ExtractRegions'
    ...
```

### 5.2 Add a final report

Add a final report step at the end to track what you've accomplished:

```toml
# ... all your processing steps ...

# Final quality report to compare with initial
[[step]]
    action = 'Report'
    name = 'final'
    count = true
    base_statistics = true
    length_distribution = true
    duplicate_count_per_read = true

[output]
    prefix = 'processed'
    format = 'Fastq'
    compression = 'Gzip'
    report_html = true
    report_json = true
```

### 5.3 Run full processing

```bash
mbf-fastq-processor process process.toml
```

For large datasets, you can monitor progress:

```bash
mbf-fastq-processor process process.toml --verbose
```

**Expected output:**

```
Reading configuration: process.toml
✓ Config validated
✓ Input files found

Processing pipeline:
  1. ExtractRegions (umi)
  2. StoreTagInComment (umi)
  3. FindConstantRegion (constant_region)
  4. TrimAtTag
  5. FindAndRemove3PrimeAdapter
  6. TrimPolyA
  7. FilterByLength
  8. Report (final)

Processing reads...
[████████████████████████████████] 100% (2.5M reads, 2m 34s)

✓ Complete!

Output files:
- processed_R1.fastq.gz (2.31M reads, 1.2GB)
- processed.report_final.html
- processed.report_final.json
```

---

## Step 6: Final Verification

Compare your initial and final reports to verify the processing worked correctly.

### 6.1 Compare read counts

**Initial report:**
```
Total reads: 2,500,000
```

**Final report:**
```
Total reads: 2,312,345 (92.5% of input)
```

**Interpretation**: Lost 7.5% of reads due to:
- Reads without constant region
- Reads too short after trimming
- Adapter-only reads

### 6.2 Compare length distributions

**Initial:**
```
Mean length: 87bp
Median: 92bp
Range: 35-151bp
```

**Final:**
```
Mean length: 62bp
Median: 65bp
Range: 30-142bp
```

**Interpretation**:
- ✓ Removed ~14bp of UMI + constant region
- ✓ Trimmed adapters and polyA
- ✓ Minimum length filter working (30bp floor)

### 6.3 Compare oligo counts

**Initial:**
```
AAAAAA (polyA):  23.5%
Illumina adapter: 1.2%
```

**Final:**
```
AAAAAA (polyA):   0.02%  ✓ Removed
Illumina adapter: 0.00%  ✓ Removed
```

**Interpretation**: Successfully removed contaminants!

### 6.4 Check base quality

**Initial:**
```
Mean quality: Q34.5
% ≥ Q30: 87.3%
```

**Final:**
```
Mean quality: Q36.2  ✓ Improved (removed low-quality tails)
% ≥ Q30: 92.1%       ✓ Improved
```

### 6.5 Verify UMI preservation

Inspect a few reads from the output:

```bash
zcat processed_R1.fastq.gz | head -20
```

```
@READ_001 umi:ATCGATCG
ACTGTACTGTACTGTAC...
+
IIIIIIIIIIIIIIIII...

@READ_002 umi:GCTAGCTA
GCTGATCGATCGATCG...
+
IIIIIIIIIIIIIIII...
```

✓ UMIs are preserved in the comments!

---

## Step 7: Downstream Analysis

Your cleaned reads are now ready for:

### 7.1 Alignment

```bash
# STAR alignment example
STAR --genomeDir genome_index \
     --readFilesIn processed_R1.fastq.gz \
     --readFilesCommand zcat \
     --outFileNamePrefix aligned_ \
     --outSAMtype BAM SortedByCoordinate
```

### 7.2 UMI-aware deduplication

```bash
# Using UMI-tools (extracts UMI from comment)
umi_tools dedup \
    --stdin=aligned_Aligned.sortedByCoord.out.bam \
    --stdout=deduplicated.bam \
    --extract-umi-method=read_id
```

Or use [mbf-bam-quantifier](https://tyberiusprime.github.io/mbf-bam-quantifier/) which handles UMI deduplication during quantification.

### 7.3 Gene quantification

Proceed with your standard RNA-seq quantification workflow (HTSeq, featureCounts, etc.).

---

## Best Practices

### 1. Always Start with Exploration

- Take a head of 10-50K reads
- Generate a report before processing
- Understand your data before making decisions

### 2. Use Interactive Mode Extensively

- Perfect for pipeline development
- Rapid iteration saves hours of time
- Catch errors early with small samples

### 3. Document Your Decisions

Add comments to your TOML config explaining why you chose specific parameters:

```toml
[[step]]
    action = 'FindConstantRegion'
    sequence = 'TACGAT'
    max_mismatches = 2  # Sequencing errors common in UMI region
    start_search_after = 6  # Skip past the 8bp UMI
```

### 4. Save Your Reports

Keep both initial and final reports for your records:

```bash
mkdir -p reports/
mv initial_report.report_initial.* reports/
mv processed.report_final.* reports/
```

### 5. Validate on Subsets

Before processing terabytes of data, validate your pipeline on:
- A head of the data (first 1M reads)
- A random sample of the data
- Known positive/negative controls if available

### 6. Version Control Your Configs

```bash
git add process.toml
git commit -m "Finalized processing pipeline for Project-XYZ"
```

### 7. Use Checksums for Reproducibility

mbf-fastq-processor supports deterministic processing with seeds:

```toml
[options]
    # Ensures reproducible results for sampling steps
    global_seed = 42
```

### 8. Monitor Resource Usage

For very large datasets:

```bash
# Monitor memory and CPU
time mbf-fastq-processor process large_dataset.toml

# Adjust block size if needed
[options]
    block_size = 100000  # Process 100K reads at a time
```

---

## Troubleshooting

### Problem: Interactive mode shows errors

**Solution**: The auto-inserted `Head(10000)` might still be too large. Create a smaller test file:

```bash
zcat raw_data/sample_R1.fastq.gz | head -40000 > test_subset.fastq
```

Then update your config to use this smaller file during development.

### Problem: Constant region not found in most reads

**Possible causes:**
1. Wrong sequence (double-check protocol)
2. Too strict mismatch tolerance (increase `max_mismatches`)
3. Wrong search region (adjust `start_search_after`)
4. Sequence is reverse-complemented (use `ReverseComplement` step first)

**Debug approach:**
Use `Inspect` to look at raw reads and find the pattern manually.

### Problem: Too many reads filtered out

**Diagnosis:**
Add diagnostic reports between steps:

```toml
[[step]]
    action = 'Report'
    name = 'after_constant_region'
    count = true

[[step]]
    action = 'TrimAtTag'
    ...

[[step]]
    action = 'Report'
    name = 'after_trim'
    count = true
```

Compare counts to see where reads are being lost.

### Problem: Output quality not improving

**Check:**
1. Are you removing the right adapters?
2. Is quality trimming too aggressive?
3. Are polyA tails actually present? (check oligo counts in report)

---

## Advanced Topics

### Paired-End Data

For paired-end data, specify both reads:

```toml
[input]
    read1 = 'sample_R1.fastq.gz'
    read2 = 'sample_R2.fastq.gz'
```

Many steps work on specific segments:

```toml
# Trim polyA only from read1 (3' end)
[[step]]
    action = 'TrimPolyA'
    segment = 'read1'

# Trim adapter only from read2
[[step]]
    action = 'FindAndRemove3PrimeAdapter'
    segment = 'read2'
    adapter = 'AGATCGGAAGAGC'
```

### Demultiplexing

If your data has index barcodes for sample separation:

```toml
[input]
    read1 = 'pooled_R1.fastq.gz'
    index1 = 'pooled_I1.fastq.gz'

# Define barcode sets
[barcodes.sample_ids]
    sequences = [
        'ATCGATCG',
        'GCTAGCTA',
        'TAGCTAGC',
    ]

[[step]]
    action = 'Demultiplex'
    barcode_label = 'sample_ids'
    max_mismatches = 1

[output]
    prefix = 'demux'
    # Creates separate files for each barcode
    demultiplex_output = 'PerBarcode'
```

### Custom Quality Filtering

Filter based on mean quality:

```toml
# Calculate mean quality per read
[[step]]
    action = 'CalcMeanQuality'
    out_label = 'mean_q'
    segment = 'read1'

# Keep only high-quality reads
[[step]]
    action = 'FilterByNumericTag'
    in_label = 'mean_q'
    min_value = 30
    keep_or_remove = 'Keep'
```

---

## Summary

This tutorial demonstrated the complete workflow:

1. ✓ **Explore** - Head + Inspect to see raw data
2. ✓ **Report** - Generate comprehensive QC metrics
3. ✓ **Analyze** - Identify adapters, polyA, quality issues
4. ✓ **Develop** - Use interactive mode for rapid iteration
5. ✓ **Process** - Run full pipeline on complete dataset
6. ✓ **Verify** - Compare initial vs. final reports

**Key takeaways:**
- Always understand your data before processing
- Interactive mode is your best friend for pipeline development
- Reports are essential for both initial QC and final verification
- Document your decisions and parameters
- Validate on subsets before processing large datasets

For more examples of specific processing patterns, see the [cookbooks]({{< relref "/docs/how-to/cookbooks" >}}).

For details on all available transformation steps, run:
```bash
mbf-fastq-processor list-steps
```

Happy processing!
