# FastQ Processor Cookbooks

This directory contains practical, real-world examples of using mbf-fastq-processor for common bioinformatics tasks.

## What are Cookbooks?

Each cookbook is a complete, self-contained example that demonstrates how to solve a specific problem or perform a common task. Unlike test cases (which focus on correctness), cookbooks focus on teaching and practical application.

**New to mbf-fastq-processor?** Check out the [Complete Workflow Tutorial](../docs/WORKFLOW_TUTORIAL.md) for a hands-on guide that walks through the entire process from initial data exploration to final verification, demonstrating how to use interactive mode and reports effectively.

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

### Protocol-Specific

3. **[lexogen-quantseq](./03-lexogen-quantseq/)** - Process Lexogen QuantSeq 3' RNA-seq data (UMI + adapter trimming)

## Contributing

When creating a new cookbook:
1. Create a new directory with a descriptive name
2. Include all four required components (README.md, input/, reference_output/, input.toml)
3. Make the README clear and educational
4. Use small, representative input files (not full datasets)
5. Update this README with a link to your cookbook
