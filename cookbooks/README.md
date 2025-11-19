# FastQ Processor Cookbooks

This directory contains practical, real-world examples of using mbf-fastq-processor for common bioinformatics tasks.

## What are Cookbooks?

Each cookbook is a complete, self-contained example that demonstrates how to solve a specific problem or perform a common task. Unlike test cases (which focus on correctness), cookbooks focus on teaching and practical application.

## Structure

Each cookbook contains:
- **README.md**: Explanation of the use case, what the cookbook does, and when to use it
- **input/**: Sample input files (FastQ files, etc.)
- **reference_output/**: Expected output files after processing
- **input.toml**: The pipeline configuration file

## Running a Cookbook

To run any cookbook:

```bash
cd cookbooks/[cookbook-name]
mbf-fastq-processor process input.toml
```

Compare your output with the files in `reference_output/` to verify correct execution.

## Available Cookbooks

### Basic Operations

1. **[basic-quality-report](./01-basic-quality-report/)** - Generate quality reports from FastQ files
2. **[umi-extraction](./02-umi-extraction/)** - Extract UMI (Unique Molecular Identifiers) from reads
5. **[quality-filtering](./05-quality-filtering/)** - Filter reads based on quality scores using expected error
6. **[adapter-trimming](./06-adapter-trimming/)** - Trim adapters and polyA tails from RNA-seq reads
8. **[length-filtering](./08-length-filtering/)** - Filter reads by length and truncate to uniform size

### Protocol-Specific

3. **[lexogen-quantseq](./03-lexogen-quantseq/)** - Process Lexogen QuantSeq 3' RNA-seq data (UMI + adapter trimming)
4. **[phiX-removal](./04-phiX-removal/)** - Remove PhiX spike-in contamination using k-mer counting
7. **[demultiplexing](./07-demultiplexing/)** - Separate pooled samples using inline barcodes with error correction

## Contributing

When creating a new cookbook:
1. Create a new directory with a descriptive name
2. Include all four required components (README.md, input/, reference_output/, input.toml)
3. Make the README clear and educational
4. Use small, representative input files (not full datasets)
5. Update this README with a link to your cookbook
