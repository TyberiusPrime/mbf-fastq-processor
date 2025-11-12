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
- **Update cookbooks** - after adding or modifying cookbooks run `dev/update_cookbooks.py` to regenerate the embedded cookbook data

**CRITICAL TESTING REMINDER**: Always run `dev/update_tests.py` before running any tests with `cargo test` when test cases have been added or modified. This ensures all test artifacts are properly generated.

**CRITICAL COOKBOOK REMINDER**: Always run `dev/update_cookbooks.py` after adding or modifying cookbooks. A test will fail if the generated cookbook data is out of sync.

To view test outputs, run `cargo test` and inspect the 'actual' folder in the test case directory.

- **Check**: `cargo check`
- **Lint**: `cargo clippy --all-targets -- -D clippy::pedantic`
- **Build statically-linked**: `nix build .#mbf-fastq-processor_other_linux` - Creates portable Linux binary
- **Coverage**: `python3 dev/coverage.py` - Generate code coverage reports
- **Interactive mode**: `./target/debug/mbf-fastq-processor interactive <config.toml>` - Watch a TOML file and show live results for rapid development

### Interactive Mode

The interactive mode provides rapid feedback during pipeline development:
- Watches a TOML configuration file for changes (polls every second)
- Automatically prepends `Head(10000)` and `FilterReservoirSample(15)` steps to limit processing
- Automatically appends an `Inspect(15)` step to display sample results
- Converts input paths to absolute paths
- Sets output to minimal configuration
- Displays results or errors in a formatted output
- Ideal for iterative development and debugging

Usage: `mbf-fastq-processor interactive <config.toml>`

## Core tenants

- Explicit is better than implicit. Configuration defaults should be minimal, make the user specify what she wants.

## Architecture

### Core Components
- **src/main.rs**: CLI entry point that parses TOML config and orchestrates processing
- **src/lib.rs**: Core library with processing pipeline, writer abstractions, and main processing logic
- **src/interactive.rs**: Interactive mode for rapid development and testing
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
- **tests/cookbook_tests.rs**: Cookbook tests (auto-generated, run separately)
- **cookbooks/**: User-facing example cookbooks with documentation

## Cookbooks

The `cookbooks/` directory contains complete, user-friendly examples demonstrating common use cases. These cookbooks are embedded into the application and can be accessed via the CLI.

### CLI Commands
- `mbf-fastq-processor cookbook` - List all available cookbooks
- `mbf-fastq-processor cookbook <number>` - Display a specific cookbook with its README and configuration
- `mbf-fastq-processor list-steps` - List all available transformation steps

### Cookbook Structure
Each cookbook is a self-contained directory with:
- **README.md**: Detailed explanation of the use case and how to use the cookbook
- **input/**: Sample input files (small, representative FastQ data)
- **reference_output/**: Expected output files for verification
- **input.toml**: The pipeline configuration file

### Adding a New Cookbook
1. Create a new directory in `cookbooks/` with a descriptive name (e.g., `05-adapter-trimming`)
2. Add the four required components (README.md, input/, reference_output/, input.toml)
3. Run `python3 dev/update_cookbooks.py` to generate embedded cookbook data
4. Run `python3 dev/update_cookbook_tests.py` to generate test cases
5. Run the cookbook to generate reference output
6. Move outputs to `reference_output/`
7. Update `cookbooks/README.md` to list the new cookbook

### Cookbook Tests
- Cookbook tests use the same `test_runner` infrastructure as regular test cases
- Tests are gated behind `#[cfg(feature = "cookbook_tests")]`
- Run only in CI/PR workflows and via `nix build .#test`
- **DO NOT run on normal `cargo test`** to keep fast iteration times
- Regenerate tests with: `python3 dev/update_cookbook_tests.py`

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


