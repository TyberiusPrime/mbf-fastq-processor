# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mbf_fastq_processor is a Rust-based FASTQ processing tool that filters, samples, slices, demultiplexes, 
and performs various analyses on FASTQ files. 

It uses TOML configuration files to define processing pipelines with multiple transformation steps. 
Hints on the TOML configuration can be found in docs/content/docs/reference/llm-guide.md.

The project emphasizes correctness, flexibility, speed, and reproducible results.

## Build System & Commands

This project uses both Nix and Cargo build systems:

### Primary Development (Nix-based)
- **Build**: `cargo build` - Builds the main binary
- **Build**: `cargo build --release` - Builds the main binary in fast release mode.
- **Test**: `cargo test` - Runs all tests
- **Test-cases** - after adding new test cases run `dev/_update_tests.py` followed by `cargo test`. 
    Do not go into folders and run tests 'manually'
- **Update cookbooks** - after adding or modifying cookbooks run `dev/update_generated.sh`  to regenerate the embedded cookbook data

To view test outputs, run `cargo test` and inspect the 'actual' folder in the (failed) test case directory.

- **Check**: `cargo check`
- **Lint**: `cargo clippy --all-targets -- -D clippy::pedantic`
- **Build statically-linked**: `nix build .#mbf-fastq-processor_other_linux` - Creates portable Linux binary
- **Coverage**: `python3 dev/coverage.py` - Generate code coverage reports

## Core tenets

- Explicit is better than implicit. Configuration defaults should be minimal, make the user specify what she wants.

## Architecture

### Core Components
- **mbf-fastq-processor/src/main.rs**: CLI entry point that parses TOML config and orchestrates processing
- **mbf-fastq-processor/src/lib.rs**: Core library with processing pipeline, writer abstractions, and main processing logic
- **mbf-fastq-processor/src/io and io.rs **: FASTQ file I/O handling, supports multiple formats (raw, gzip, zstd), parallel reading
- **mbf-fastq-processor/src/transformations and transformations.rs**: Pipeline step implementations (filtering, trimming, reporting, etc.)
- **mbf-fastq-processor/src/config and config.rs**: TOML parsing and validation
- **mbf-fastq-processor/src/demultiplex.rs**: Demultiplexing logic for separating samples
- **mbf-fastq-processor/src/dna.rs**: DNA sequence utilities and operations

## Development tools
    all dev scripts go into the `dev` directory.

### Processing Model
The tool uses a pipeline architecture where:
1. Input files are read in parallel 
2. Each `[[step]]` in the TOML config becomes a transformation in the pipeline
3. Data flows through transformations sequentially
4. Writers handle output in multiple formats (raw, gzip, zstd)

### Test Structure
- **test_cases/**: Directory containing test FASTQ files and expected outputs. These are run with the CLI's 'verify' command
- **mbf-fastq-processor/tests/test_runner** - Supporting infrastructure for the test_cases.
- **mbf-fastq-processor/tests/integration_tests.rs** - Tests that validate other CLI commands
- **mbf-fastq-processor/tests/template_and_documentation_verification.rs** - Tests that verify the documentation contains everything it should
- **mbf-fastq-processor/tests/parser** - Test cases explicitly for the FASTQ parser
*
- **cookbooks/**: User-facing example cookbooks with documentation

## Cookbooks

The `cookbooks/` directory contains complete, user-friendly examples demonstrating common use cases.
These cookbooks are embedded into the application and can be accessed via the CLI.

### CLI Commands
- `mbf-fastq-processor process <input.toml>` - Run the pipeline
- `mbf-fastq-processor validate <input.toml>` - Verify the parsing & check the config of a toml
- `mbf-fastq-processor verify <input.toml>` - Verify that the output is identical to the files in the current directory
- `mbf-fastq-processor template` - Output a complete configuration template (or a subset)
- `mbf-fastq-processor list-steps` - List all available transformation steps
- `mbf-fastq-processor cookbook` - List all available cookbooks
- `mbf-fastq-processor cookbook <number>` - Display a specific cookbook with its README and configuration

### Cookbook Structure
Each cookbook is a self-contained directory with:
- **README.md**: Detailed explanation of the use case and how to use the cookbook
- **input/**: Sample input files (small, representative FASTQ data)
- **reference_output/**: Expected output files for verification
- **input.toml**: The pipeline configuration file

### Adding a New Cookbook
1. Create a new directory in `cookbooks/` with a descriptive name (e.g., `05-adapter-trimming`)
2. Add the four required components (README.md, input/, reference_output/, input.toml)
3. Run `dev/update_generated.sh` (or `python3 dev/_update_cookbooks.py`) to generate embedded cookbook data and test cases
4. Run the cookbook to generate reference output
5. Move outputs to `reference_output/`
6. Update `cookbooks/README.md` to list the new cookbook


## Configuration System
The project uses TOML files for configuration with sections:
- `[input]`: Define input files (read1, read2, index1, index2)
- `[[step]]`: Pipeline transformation steps (can have multiple)
- `[output]`: Output format and file naming

## Key Dependencies
- **bio**: Bioinformatics utilities
- **crossbeam**: Parallel processing and channels
- **niffler**: File format detection and compression
- **zstd**: Fast compression
- **serde/toml**: Configuration parsing
- **anyhow**: Error handling


## Version control
The project uses jujutsu (jj) for version control. 
Common commands include: 
 - **commit**:  `jj commit -m "message"`

Commit after every significant change, ideally after completing a feature or fixing a bug.


