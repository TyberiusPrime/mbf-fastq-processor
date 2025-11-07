+++
+++

# Cookbook 03: Lexogen QuantSeq Processing

## Use Case

Lexogen QuantSeq is a popular 3' mRNA sequencing protocol optimized for gene expression profiling. The library structure includes:
- **First 8 bases**: UMI (Unique Molecular Identifier) for deduplication
- **Next 6 bases**: Random hexamer primer sequence (needs removal)
- **Remaining sequence**: Actual cDNA from the 3' end of transcripts

This cookbook demonstrates the standard preprocessing for QuantSeq data before alignment.

## What This Pipeline Does

1. Extracts the 8bp UMI from the start of reads
2. Stores the UMI in the read comment (FASTQ header)
3. Removes the first 14 bases total (8bp UMI + 6bp random hexamer)
4. Outputs processed reads ready for alignment

## Input Files

- `input/quantseq_sample.fq` - Raw QuantSeq reads with UMI and random hexamer

## Output Files

- `output_read1.fq` - Processed reads with:
  - UMI stored in comment
  - First 14bp removed
  - Ready for alignment to reference genome

## Workflow Details

**Raw read structure:**
```
@READ1
ATCGATCGTTACGATACTGTACTGTACTGTAC...
^^^^^^^^  ^^^^^^
   UMI    Hexamer  <- These get removed
        ^^^^^^^^^^... <- This stays for alignment
```

**After processing:**
```
@READ1 umi:ATCGATCG
ACTGTACTGTACTGTAC...
```

The UMI is preserved in the comment for downstream deduplication, and the adapter/primer sequences are removed.

## Configuration Highlights

```toml
[[step]]
    # Extract UMI (first 8 bases)
    action = 'ExtractRegions'
    label = 'umi'
    regions = [{segment = 'read1', start = 0, length = 8}]

[[step]]
    # Store UMI in comment for deduplication
    action = 'StoreTagInComment'
    label = 'umi'

[[step]]
    # Remove UMI + random hexamer (14 bases total)
    action = 'CutStart'
    segment = 'read1'
    n = 14
```

## When to Use This

- Processing Lexogen QuantSeq FWD/REV libraries
- Any 3' RNA-seq protocol with UMI + random primer structure
- Before aligning to reference genome for gene expression analysis

## Downstream Analysis

After processing with this cookbook:

1. **Align to reference genome** using STAR, HISAT2, or similar
2. **Assign to genes** using featureCounts or HTSeq
3. **Deduplicate using UMI** with tools like:
   - `umi_tools dedup` (extracts UMI from comment)
   - `fgbio GroupReadsByUmi`
4. **Quantify gene expression** with standard DE tools (DESeq2, edgeR)

## Important Notes

- The 6bp random hexamer introduces sequence bias; UMI-based deduplication helps mitigate this
- QuantSeq reads are strand-specific (typically R2/reverse strand)
- Read lengths will be 14bp shorter after processing
- Quality filtering may be beneficial after trimming (see cookbook 03-quality-filtering)

## Running This Cookbook

```bash
cd cookbooks/03-lexogen-quantseq
mbf-fastq-processor process input.toml
```

## References

- [Lexogen QuantSeq 3' mRNA-Seq Library Prep Kit](https://www.lexogen.com/quantseq-3mrna-sequencing/)
- [UMI-tools documentation](https://umi-tools.readthedocs.io/)


## Download

[Download 03-lexogen-quantseq.tar.gz](../../../../../cookbooks/03-lexogen-quantseq.tar.gz) for a complete, runnable example.

## Configuration File

```toml
[input]
    # QuantSeq produces single-end reads
    read1 = 'input/quantseq_sample.fq'

[[step]]
    # Extract the 8bp UMI from the start of each read
    # QuantSeq uses 8bp random UMI for PCR duplicate identification
    action = 'ExtractRegions'
    label = 'umi'
    regions = [{segment = 'read1', start = 0, length = 8}]

[[step]]
    # Store the UMI in the FASTQ comment
    # This preserves it for downstream deduplication with umi_tools or similar
    action = 'StoreTagInComment'
    label = 'umi'

[[step]]
    # Remove the first 10 bases from reads:
    # - 8bp UMI
    # - 4bp TATA spacer
    # What remains is the actual cDNA sequence for alignment
    action = 'CutStart'
    segment = 'read1'
    n = 10

[output]
    prefix = 'output'
    format = "FASTQ"
```
