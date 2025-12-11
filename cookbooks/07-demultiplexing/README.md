# Cookbook 07: Demultiplexing by Inline Barcode

## Use Case

You have pooled sequencing data from multiple samples that were tagged with unique barcode sequences during library preparation
and have not been demuliplexed by your sequencing facility. 

You need to:
- Extract the barcode(s) from each read
- Correct sequencing errors in barcodes
- Separate reads into individual files per sample

This is common in multiplexed sequencing runs to maximize sequencing efficiency and reduce costs.

## What This Pipeline Does

This cookbook demonstrates a complete demultiplexing workflow:

1. **Extract Barcode**: Uses `ExtractRegion` to extract inline barcode from the start of reads (fixed position)
2. **Correct Errors**: Uses `HammingCorrect` to fix single-base errors in barcodes
3. **Remove Barcode from Sequence**: Uses `CutStart` to trim the barcode bases from reads
4. **Demultiplex**: Uses `Demultiplex` to split reads into separate files per sample
5. **Generate Report**: Creates summary statistics for each sample

## Understanding Barcodes

**Inline barcodes** are short DNA sequences (4-12bp) added to the start or end of reads:
- **Purpose**: Uniquely identify which sample each read came from
- **Location**: Typically at the 5' end of read1 or in a separate index read (we support any number of segments!)
- **Errors**: Sequencing errors can cause misassignment; error correction is helpful
- **Hamming distance**: Number of positions at which sequences differ
  - Hamming distance = 1: One base different (e.g., ATCG vs ACCG)
  - Good barcode sets have Hamming distance ≥ 3 for robust error correction

## Input Files

- `input/pooled_R1.fq` - Pooled reads from multiple samples with inline barcodes

## Output Files

- `output_sample1_read1.fq` - Reads belonging to sample1
- `output_sample2_read1.fq` - Reads belonging to sample2
- `output_sample3_read1.fq` - Reads belonging to sample3
- `output_sample4_read1.fq` - Reads belonging to sample4
- `output_no-barcode_read1.fq` - Reads with unrecognized barcodes

## Expected Results

With the provided sample data:
- **Input:** 12 reads (from 4 samples plus some with errors)
- **Output:** Separate files for each sample, with barcode sequences removed
- Barcodes with 1 error are corrected to the nearest valid barcode
- Reads with >1 error /no match go to the unmatched file

## Barcode Design Considerations

When designing barcodes:
1. **Hamming distance ≥ 3**: Allows single-error correction
2. **Balanced GC content**: Improves sequencing quality
3. **Avoid homopolymers**: AAAA, TTTT, etc. cause sequencing errors
4. **Distinct patterns**: Avoid similar-looking barcodes

**Example good barcode set (6bp, Hamming ≥ 3):**
- ATCACG
- CGATGT
- TTAGGC
- TGACCA
- ACAGTG
- GCCAAT

## Customization

Adjust parameters based on your experimental design:

**Barcode Location:**
Examples
- Start of read1: `segment = 'read1', start = 0, anchor='Start'`
- End of read1: `segment = 'read1', start = -6, anchor='End'` (for 6bp barcode)
- Separate index read: `segment = 'index1'`

**Error Correction:**
- `max_hamming_distance = 0`: No error correction, leave off the HammingCorrect step
- `max_hamming_distance = 1`: Correct single-base errors (recommended)
- `max_hamming_distance = 2`: Correct two errors (requires Hamming ≥ 5 barcode set)

**Unmatched Reads:**
- `output_unmatched = true`: Save unmatched reads for QC
- output_unmatched = false`: Discard unmatched reads
- `on_no_match = 'remove'`: Set tags that do not match to 'missing' (usefull for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}})
- `on_no_match = 'empty'`: Set tags that do not match to "" (but keep the location data)
- `on_no_match = 'keep'`: Keep the non-matching barcode. Combine with 
[`tag_histogram in report]({{< relref "docs/reference/report-steps/Report.md" >}}) to 
find used-but-undocumented barcodes.

## When to Use This

- Multiplexed sequencing runs with inline barcodes
- Single-cell experiments with cell barcodes
- Pooled CRISPR screens with guide barcodes
- Any application where multiple samples are sequenced together

## Alternative Approaches

**Index reads instead of inline barcodes:**
If barcodes are in a separate index file rather than inline:
```toml
[[step]]
    action = 'ExtractRegion'
    segment = 'index1'  # Use index read instead
    start = 0
    length = 8
    out_label = 'barcode'
```

**Dual indexing:**
For higher multiplexing, use two index reads:
```toml
# Extract from index1
[[step]]
    action = 'ExtractRegion'
    segment = 'index1'
    start = 0
    length = 8
    out_label = 'i7'

# Extract from index2
[[step]]
    action = 'ExtractRegion'
    segment = 'index2'
    start = 0
    length = 8
    out_label = 'i5'

# Concatenate barcodes
[[step]]
    action = 'ConcatTags'
    in_labels = ['i7', 'i5']
    out_label = 'barcode'
    separator = '_'

# Then demultiplex on concatenated barcode
```

## Downstream Analysis

After demultiplexing:
1. **Quality control** per sample ([Report]({{< relref "docs/reference/report-steps/Report.md" >}}))
2. **Alignment** to reference genome
3. **Sample-specific analysis** (variant calling, expression quantification)
4. **Combine results** across samples for comparative analysis

## Quality Control

Check demultiplexing quality by examining:
- **Reads per sample**: Should be roughly balanced (unless intentionally unequal)
- **Unmatched rate**: High rates (>10%) suggest barcode design or sequencing issues
- **Error correction rate**: Monitor how many barcodes required correction
