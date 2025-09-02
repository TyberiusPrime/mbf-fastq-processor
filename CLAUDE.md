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
- **Test cases**: ``cargo run --release --bin mbf-fastq-processor-test-runner  - Runs input output test cases
- **Check**: `cargo check`
- **Lint**: `cargo clippy --all-targets -- -D clippy::pedantic`
- **Build statically-linked**: `nix build .#mbf-fastq-processor_other_linux` - Creates portable Linux binary

## Architecture

### Core Components
- **src/main.rs**: CLI entry point that parses TOML config and orchestrates processing
- **src/lib.rs**: Core library with processing pipeline, writer abstractions, and main processing logic
- **src/io.rs**: FastQ file I/O handling, supports multiple formats (raw, gzip, zstd), parallel reading
- **src/transformations.rs**: Pipeline step implementations (filtering, trimming, reporting, etc.)
- **src/config/**: TOML configuration parsing and validation
- **src/demultiplex.rs**: Demultiplexing logic for separating samples
- **src/dna.rs**: DNA sequence utilities and operations

### Processing Model
The tool uses a pipeline architecture where:
1. Input files are read in parallel using crossbeam channels
2. Each `[[step]]` in the TOML config becomes a transformation in the pipeline
3. Data flows through transformations sequentially
4. Writers handle output in multiple formats (raw, gzip, zstd)

### Test Structure
- **tests/integration_tests.rs**: Integration tests using TOML configs
- **test_cases/**: Directory containing test FastQ files and expected outputs
- **src/bin/mbf-fastq-processor-test-runner.rs**: Custom test runner binary

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
