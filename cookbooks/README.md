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
3. **[quality-filtering](./03-quality-filtering/)** - Filter reads by quality scores

### Demultiplexing

4. **[simple-demultiplex](./04-simple-demultiplex/)** - Demultiplex samples using barcode sequences

### Advanced

5. **[complete-pipeline](./05-complete-pipeline/)** - Full pipeline combining multiple operations

## Contributing

When creating a new cookbook:
1. Create a new directory with a descriptive name
2. Include all four required components (README.md, input/, reference_output/, input.toml)
3. Make the README clear and educational
4. Use small, representative input files (not full datasets)
5. Update this README with a link to your cookbook
