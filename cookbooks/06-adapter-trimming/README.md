# Cookbook 06: Adapter Trimming with PolyA Tail Removal

## Use Case

You have RNA-seq data that contains:
- **PolyA tails**: Stretches of A bases at the 3' end (or polyT at 5' for reverse strand)
- **Sequencing adapters**: Illumina or other adapter sequences that need removal before alignment

These artifacts can interfere with alignment and downstream analysis if not removed.

## What This Pipeline Does

This cookbook demonstrates a complete adapter and polyA trimming workflow:

1. **Extract PolyA Tail**: Uses `ExtractPolyTail` to find polyA/T stretches
2. **Trim PolyA**: Uses `TrimAtTag` to remove the polyA tail and everything after it
3. **Extract Adapter**: Uses `ExtractIUPAC` to find Illumina adapter sequences
4. **Trim Adapter**: Uses `TrimAtTag` to remove adapter contamination
5. **Filter Short Reads**: Uses `CalcLength` and `FilterByNumericTag` to remove reads that became too short after trimming

## Understanding PolyA/T Tails

PolyA tails are natural features of mRNA:
- **Biological**: mRNA molecules have polyA tails added during transcription
- **Sequencing artifact**: If the read extends past the transcript end, it captures the polyA tail
- **Impact**: Can interfere with alignment if not removed
- **PolyT**: Reverse strand sequences show polyT instead of polyA


## Input Files

- `input/rna_sample_R1.fq` - RNA-seq reads with polyA tails and adapters

## Output Files

- `output_read1.fq` - Trimmed reads with polyA tails and adapters removed

## Expected Results

With the provided sample data:
- **Input:** 8 reads with various combinations of polyA tails and adapters
- **Output:** Trimmed reads with polyA and adapters removed
- Reads that became too short (< 25bp) after trimming are filtered out

## Workflow Details

**Example read transformation:**

Adapter
```
AGATCGGAAGAGC
```

**Before:**
```
@READ1
ACTGACTGACTGACTGAAAAAAAAAAGATCGGAAGAGCACACGTCTGAACTCCAGTCAC
              ^^^^^^^^^^^^^             
              PolyA ↑
                         ^^^^^^^^^^^^
                          Adapter ↑
```

**After polyA trimming:**
```
ACTGACTGACTGACTG
```

## Customization

Adjust parameters based on your data:

**PolyA Detection:**
- `min_length`: Minimum polyA length 
- `max_mismatch_rate`: Allow some misread (non-A) bases in the polyA tail

**Adapter Sequences:**

See [adapters sequences ]({{< relref "docs/reference/adapters.md" >}}) for common adapters.
- Use `max_mismatches` to allow for sequencing errors in adapter


**Length Filtering:**
- `min_value`: Minimum read length to keep (adjust based on alignment requirements)
- For RNA-seq: typically 25-50bp minimum
- For miRNA: might keep shorter reads (18-22bp)

## When to Use This

- RNA-seq data before alignment
- Any protocol where reads may extend past the insert (polyA capture)
- When adapter contamination is detected in quality reports
- Before transcriptome assembly or quantification

## Alternative Approaches

This cookbook uses a two-step approach (extract → trim). You can also use:
- `ExtractLongestPolyX`: Finds the longest stretch of any repeated nucleotide
- `ExtractAnchor`: More flexible pattern matching with orientation
- Multiple `TrimAtTag` steps for different adapter types

## Downstream Analysis

After trimming:
1. **Alignment** to reference genome (STAR, HISAT2)
2. **Quantification** of gene/transcript expression
3. **Quality control** to verify adapter removal (FastQC, MultiQC)
