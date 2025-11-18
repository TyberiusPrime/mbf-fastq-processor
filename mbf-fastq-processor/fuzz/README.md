# Fuzzing Infrastructure

This directory contains fuzz targets for mbf-fastq-processor.

## Quick Start

```bash
# Install cargo-fuzz (one-time setup)
cargo install cargo-fuzz
rustup install nightly

# Run a fuzz target for 60 seconds
cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=60
```

## Available Targets

- **fuzz_config_parser**: Tests TOML configuration parsing
- **fuzz_fastq_parser**: Tests FastQ file format parsing
- **fuzz_custom_deserializers**: Tests custom deserializers for DNA sequences, barcodes, etc.

## Full Documentation

See [FUZZING.md](../../FUZZING.md) in the repository root for complete documentation including:
- Detailed target descriptions
- Advanced usage options
- Continuous fuzzing setup
- Troubleshooting guide

## Directory Structure

```
fuzz/
├── Cargo.toml              # Fuzz target dependencies
├── fuzz_targets/           # Fuzz target implementations
│   ├── fuzz_config_parser.rs
│   ├── fuzz_fastq_parser.rs
│   └── fuzz_custom_deserializers.rs
├── corpus/                 # Seed inputs (create as needed)
│   ├── fuzz_config_parser/
│   ├── fuzz_fastq_parser/
│   └── fuzz_custom_deserializers/
└── artifacts/              # Crash artifacts (git-ignored)
```

## Common Commands

```bash
# Run indefinitely (Ctrl+C to stop)
cargo +nightly fuzz run fuzz_config_parser

# Run with multiple parallel jobs
cargo +nightly fuzz run fuzz_fastq_parser -- -jobs=4

# List all available targets
cargo +nightly fuzz list

# Minimize a crash case
cargo +nightly fuzz tmin fuzz_config_parser artifacts/fuzz_config_parser/crash-*
```
