# Cookbook 02: UMI Extraction

## Use Case

You have sequencing data with Unique Molecular Identifiers (UMIs) embedded in the reads. UMIs are short random barcodes added during library preparation that allow you to:
- Identify and remove PCR duplicates
- Distinguish true biological duplicates from amplification artifacts
- Improve accuracy in quantitative analyses (RNA-seq, ATAC-seq, etc.)

## What This Pipeline Does

1. Reads input FastQ file with UMIs at the start of read1
2. Extracts the UMI sequence (first 8 bases) and creates a tag
3. Stores the UMI in the read comment (FASTQ header)
4. Removes the UMI bases from the read sequence (so they don't interfere with alignment)
5. Outputs modified reads with UMI preserved in the header

## Input Files

- `input/sample_R1.fq` - Reads with 8bp UMI at the start

## Output Files

- `output_R1.fq` - Reads with UMI in comment, UMI bases removed from sequence

## Configuration Highlights

```toml
[[step]]
    # Extract UMI from positions 0-7 (8 bases)
    action = 'ExtractRegions'
    label = 'umi'
    regions = [{source = 'read1', start = 0, length = 8, anchor="Start"}]

[[step]]
    # Store UMI in the FASTQ comment
    action = 'StoreTagInComment'
    label = 'umi'

[[step]]
    # Remove the UMI bases from the read
    action = 'CutStart'
    target = 'Read1'
    n = 8
```

## Workflow Details

**Before processing:**
```
@READ1
ATCGATCGACTGTACTGTACTGTACTGTACTG
+
IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII
```

**After processing:**
```
@READ1 umi:ATCGATCG
ACTGTACTGTACTGTACTGTACTG
+
IIIIIIIIIIIIIIIIIIIIIIII
```

The UMI `ATCGATCG` is now in the comment and removed from the sequence.

## When to Use This

- Single-cell RNA-seq with UMIs
- ATAC-seq with UMI-based deduplication
- Any protocol using unique molecular identifiers
- Before alignment when you need to preserve UMIs for downstream duplicate marking

