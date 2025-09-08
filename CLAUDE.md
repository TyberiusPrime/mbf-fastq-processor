# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mbf_fastq_processor is a Rust-based FastQ processing tool that filters, samples, slices, demultiplexes, and performs various analyses on FastQ files. It uses TOML configuration files to define processing pipelines with multiple transformation steps. The project emphasizes correctness, flexibility, speed, and reproducible results.

## Build System & Commands

This project uses both Nix and Cargo build systems:

### Primary Development (Nix-based)
- **Build**: `cargo build` - Builds the main binary
- **Build**: `cargo build --release` - Builds the main binary in fast release mode.
- **Test**: `cargo test` - Runs all tests 
- **Test-cases** - after adding new test cases run `dev/update_tests.py` followed by `cargo test`. Do not go into folders and run tests 'manually'

**CRITICAL TESTING REMINDER**: Always run `dev/update_tests.py` before running any tests with `cargo test` when test cases have been added or modified. This ensures all test artifacts are properly generated.
- **Check**: `cargo check`
- **Lint**: `cargo clippy --all-targets -- -D clippy::pedantic`
- **Build statically-linked**: `nix build .#mbf-fastq-processor_other_linux` - Creates portable Linux binary
- **Coverage**: `python3 dev/coverage.py` - Generate code coverage reports

## Architecture

### Core Components
- **src/main.rs**: CLI entry point that parses TOML config and orchestrates processing
- **src/lib.rs**: Core library with processing pipeline, writer abstractions, and main processing logic
- **src/io.rs**: FastQ file I/O handling, supports multiple formats (raw, gzip, zstd), parallel reading
- **src/transformations.rs**: Pipeline step implementations (filtering, trimming, reporting, etc.)
- **src/config/**: TOML configuration parsing and validation
- **src/demultiplex.rs**: Demultiplexing logic for separating samples
- **src/dna.rs**: DNA sequence utilities and operations

## Dev tools
    all dev scripts go into the `dev` directory.

### Processing Model
The tool uses a pipeline architecture where:
1. Input files are read in parallel using crossbeam channels
2. Each `[[step]]` in the TOML config becomes a transformation in the pipeline
3. Data flows through transformations sequentially
4. Writers handle output in multiple formats (raw, gzip, zstd)

### Test Structure
- **tests/integration_tests.rs**: Integration tests using TOML configs
- **test_cases/**: Directory containing test FastQ files and expected outputs

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

## Code Coverage

The project uses `cargo-llvm-cov` for code coverage measurement. Current coverage: **87.6% regions, 83.0% functions, 92.5% lines** across 252 tests.

### Coverage Commands
- **Quick Summary**: `python3 dev/coverage.py` or `python3 dev/coverage.py --summary`
- **HTML Report**: `python3 dev/coverage.py --html` (saves to `coverage-html/html/index.html`)
- **LCOV Report**: `python3 dev/coverage.py --lcov` (saves to `coverage.lcov`)
- **JSON Report**: `python3 dev/coverage.py --json` (saves to `coverage.json`)
- **All Formats**: `python3 dev/coverage.py --all`
- **Open in Browser**: `python3 dev/coverage.py --html --open`

### Raw cargo-llvm-cov Commands
- **Summary Only**: `cargo llvm-cov test --summary-only`
- **HTML Report**: `cargo llvm-cov test --html --output-dir coverage-html`
- **LCOV Format**: `cargo llvm-cov test --lcov --output-path coverage.lcov`
- **JSON Format**: `cargo llvm-cov test --json --output-path coverage.json`

### Coverage Targets
- Maintain **>85%** line coverage
- Focus on critical business logic and error handling paths
- Test both success and failure scenarios


