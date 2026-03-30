# Cookbook 09: Fastp-Equivalent Workflow

## Use Case

You want to replicate the default behavior of [fastp](https://github.com/OpenGene/fastp/) — a popular all-in-one FASTQ preprocessor — using a configurable pipeline. This is useful when you need reproducible, step-by-step control over each filtering stage, or want to extend the workflow beyond what fastp offers.

## What This Pipeline Does

This cookbook replicates fastp's default single-end processing pipeline:

1. **PolyG Trimming**: Uses `ExtractPolyTail` + `TrimAtTag` to remove polyG tails (Illumina NextSeq/NovaSeq artifact)
2. **Adapter Trimming**: Uses `ExtractIUPAC` + `TrimAtTag` to remove the Illumina TruSeq R1 adapter
3. **N-base Filtering**: Uses `CalcNCount` + `FilterByNumericTag` to remove reads with too many ambiguous bases (`--n_base_limit 5`)
4. **Quality Filtering**: Uses `CalcQualifiedBases` + `FilterByNumericTag` to remove reads with too many low-quality bases (`--qualified_quality_phred 15`, `--unqualified_percent_limit 40`)
5. **Length Filtering**: Uses `CalcLength` + `FilterByNumericTag` to remove reads shorter than 15bp (`--length_required 15`)

## Fastp Defaults Replicated

| Fastp parameter | Value | Pipeline step |
|---|---|---|
| `--poly_g_min_len` | 10 | `ExtractPolyTail min_length = 10` |
| `--adapter_sequence` | `AGATCGGAAGAGCACACGTCTGAACTCCAGTCA` | `ExtractIUPAC query = ...` |
| `--n_base_limit` | 5 | `FilterByNumericTag max_value = 6` |
| `--qualified_quality_phred` | 15 (Phred) → 48 (ASCII) | `CalcQualifiedBases threshold = 48` |
| `--unqualified_percent_limit` | 40% → keep if ≥ 60% qualified | `FilterByNumericTag min_value = 0.60` |
| `--length_required` | 15 | `FilterByNumericTag min_value = 15` |

**Note on quality scores**: Quality values in FASTQ files are ASCII-encoded. Phred Q15 corresponds to ASCII character 48 (`15 + 33 = 48`).

## Fastp Considerations

**PolyG trimming** is automatically enabled in fastp when the read header starts with instrument prefixes indicating NextSeq or NovaSeq data (e.g., `@NS`, `@FS`, `@MN`, `@NB`, etc.). In this cookbook we enable it explicitly.

**Adapter detection** in fastp is automatic but requires several thousand reads. Here we specify the adapter sequence directly: `AGATCGGAAGAGCACACGTCTGAACTCCAGTCA` (Illumina TruSeq R1). See the [adapters reference]({{< relref "docs/reference/adapters.md" >}}) for other common sequences.

**Filtering order**: fastp applies trimming (polyG, adapter) before calculating quality metrics. This pipeline follows the same order.

## Input Files

- `R1.fq` — Single-end FASTQ reads

## Output Files

- `output_r1.fq` — Processed reads after all trimming and filtering steps

## Expected Results

With the provided sample data:
- Reads with polyG tails have tails removed
- Reads containing the TruSeq adapter are trimmed at the adapter site
- Reads with more than 5 N bases are removed
- Reads where fewer than 60% of bases meet Q15 are removed
- Reads shorter than 15bp (including those shortened by trimming) are removed

## Customization

**For paired-end data**, add `r2` to the input and apply matching steps to `segment = 'read2'`.

**Adapter sequences**: Replace the `query` value in the `ExtractIUPAC` step with your adapter. For common adapters see the [adapters reference]({{< relref "docs/reference/adapters.md" >}}).

**Stricter quality filtering:**
```toml
min_value = 0.80  # keep reads with ≥80% bases above threshold
```

**Longer minimum length** (e.g., for alignment):
```toml
min_value = 25
```

## When to Use This

- When you want to match fastp's default preprocessing for comparison or reproducibility
- As a starting point for custom workflows based on fastp's defaults
- When you need to inspect or audit each filtering step independently

## Downstream Analysis

After preprocessing:
1. **Alignment** to reference genome (BWA, STAR, Bowtie2)
2. **Quantification** of gene expression
3. **Variant calling** with improved accuracy from cleaner input data
