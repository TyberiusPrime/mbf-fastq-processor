# Workspace Migration Guide

This document describes the proposed workspace structure for splitting mbf-fastq-processor into multiple crates to reduce incremental compile times.

## Structure Overview

The codebase is split into 5 crates with clear separation of concerns:

```
mbf-fastq-processor/
├── Cargo.toml (workspace root)
├── src/ (main binary crate)
│   ├── main.rs (CLI entry point)
│   ├── pipeline.rs (pipeline orchestration)
│   ├── interactive.rs (interactive mode)
│   ├── cookbooks.rs (cookbook management)
│   ├── list_steps.rs (CLI helpers)
│   └── documentation.rs (docs generation)
└── crates/
    ├── mbf-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── reads.rs (FastQElement, FastQRead, Position)
    │       └── dna.rs (Anchor, Hits, TagValue, DNA utilities)
    │
    ├── mbf-config/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── input.rs (InputConfig)
    │       ├── output.rs (OutputConfig)
    │       ├── segments.rs (Segment types)
    │       ├── options.rs (Options)
    │       └── deser.rs (Deserialization utilities)
    │
    ├── mbf-io/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── fileformats.rs (format detection)
    │       ├── input.rs (file opening, parallel reading)
    │       ├── output.rs (compression, BAM output)
    │       └── parsers.rs (FastQ/BAM parsing)
    │
    └── mbf-transformations/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── filters.rs (quality, length, content filtering)
            ├── edits.rs (trimming, cutting)
            ├── demultiplex.rs (demultiplexing logic)
            ├── tag.rs (tagging operations)
            ├── extract.rs (extraction steps)
            ├── reports.rs (reporting transformations)
            ├── validation.rs (validation steps)
            ├── calc.rs (calculations)
            ├── convert.rs (format conversions)
            ├── hamming_correct.rs (error correction)
            └── prelude.rs (common types)
```

## Dependency Graph

```
mbf-core
  ↓
  ├─→ mbf-config
  │     ↓
  └─→ mbf-io
        ↓
    mbf-transformations
        ↓
    mbf-fastq-processor (main binary)
```

## Crate Descriptions

### mbf-core
**Purpose:** Core data structures with minimal dependencies

**Contents:**
- `FastQElement` - Zero-copy or owned sequence component
- `FastQRead` - Complete read representation
- `Position` - Range tracking
- DNA utilities (Anchor, Hits, TagValue)

**Dependencies:** anyhow, bstr, memchr

**Why separate:** These types are used everywhere. Keeping them isolated with minimal dependencies means changes to I/O, config, or transformations won't trigger recompilation of this foundational code.

### mbf-config
**Purpose:** TOML configuration parsing and validation

**Contents:**
- Config structs (InputConfig, OutputConfig, Options)
- Segment definitions
- Deserialization utilities
- Validation logic

**Dependencies:** mbf-core, serde ecosystem, toml, eserde

**Why separate:** Configuration changes are frequent during development. Isolating config parsing means transformation logic doesn't recompile when config structures change.

### mbf-io
**Purpose:** All file I/O operations

**Contents:**
- File format detection
- FastQ/BAM/FASTA parsing
- Compressed I/O (gzip, zstd)
- Parallel file reading
- Output formatting and compression

**Dependencies:** mbf-core, mbf-config, bio, niffler, noodles, compression libs

**Why separate:** I/O code is relatively stable. Separating it means changes to transformation logic don't trigger recompilation of I/O code.

### mbf-transformations
**Purpose:** All transformation step implementations

**Contents:**
- Filtering (quality, length, duplicates, content)
- Trimming and editing
- Demultiplexing
- Tagging and extraction
- Validation steps
- Reporting
- Calculations and conversions

**Dependencies:** mbf-core, mbf-config, regex, rand, cuckoo filters, etc.

**Why separate:** This is where most development happens. Isolating transformations means adding/modifying steps only recompiles this crate and the main binary, not core/config/io.

### mbf-fastq-processor (main)
**Purpose:** CLI and pipeline orchestration

**Contents:**
- CLI argument parsing
- Pipeline orchestration
- Interactive mode
- Cookbook management
- Thread management

**Dependencies:** All workspace crates, clap

**Why separate:** This is the glue layer. It's small and compiles quickly.

## Benefits

### Reduced Compile Times
- **Incremental builds:** Changing a transformation only recompiles `mbf-transformations` and the main binary
- **Parallel compilation:** Cargo can compile independent crates in parallel
- **Smaller units:** Each crate compiles faster than the monolithic version

### Better Code Organization
- **Clear boundaries:** Each crate has a well-defined purpose
- **Explicit dependencies:** Import paths make dependencies obvious
- **Reusability:** Core types could be used by other tools

### Improved Testing
- **Focused tests:** Each crate can have its own test suite
- **Faster test iteration:** Run tests for only the crate you're modifying
- **Better coverage tracking:** Per-crate coverage metrics

## Migration Steps

### Phase 1: Create Workspace Structure (Completed)
- [x] Create `crates/` directory structure
- [x] Write individual `Cargo.toml` files
- [x] Create workspace root `Cargo.toml`
- [x] Write skeleton `lib.rs` for each crate

### Phase 2: Move Core Types
1. Copy `src/io/reads.rs` → `crates/mbf-core/src/reads.rs`
2. Copy `src/dna.rs` → `crates/mbf-core/src/dna.rs`
3. Update imports in both files
4. Test compilation: `cargo build -p mbf-core`

### Phase 3: Move Config
1. Copy entire `src/config/` → `crates/mbf-config/src/`
2. Update `src/config.rs` → `crates/mbf-config/src/lib.rs`
3. Fix imports to use `mbf_core::`
4. Test compilation: `cargo build -p mbf-config`

### Phase 4: Move I/O
1. Copy `src/io/` files → `crates/mbf-io/src/`
2. Update imports to use `mbf_core::` and `mbf_config::`
3. Move output-related code from `src/output.rs`
4. Test compilation: `cargo build -p mbf-io`

### Phase 5: Move Transformations
1. Copy transformation modules → `crates/mbf-transformations/src/`
2. Copy `src/demultiplex.rs` → `crates/mbf-transformations/src/demultiplex.rs`
3. Update all imports
4. Test compilation: `cargo build -p mbf-transformations`

### Phase 6: Update Main Binary
1. Update `src/lib.rs` to import from workspace crates
2. Update `src/main.rs`, `src/pipeline.rs`, etc.
3. Move `output.rs` orchestration code to main crate
4. Test full compilation: `cargo build`

### Phase 7: Testing & Validation
1. Run full test suite: `cargo test`
2. Run cookbook tests: `python3 dev/update_cookbook_tests.py && nix build .#test`
3. Check coverage: `python3 dev/coverage.py`
4. Test interactive mode
5. Build release binary: `cargo build --release`

### Phase 8: Documentation & CI
1. Update CLAUDE.md with new structure
2. Update CI workflows for workspace
3. Update README.md
4. Document import patterns

## Import Patterns

### Before (monolithic)
```rust
use crate::io::{FastQRead, FastQElement};
use crate::config::Config;
use crate::transformations::Transformation;
```

### After (workspace)
```rust
use mbf_core::{FastQRead, FastQElement};
use mbf_config::Config;
use mbf_transformations::Transformation;
```

## Build Commands

### Build everything
```bash
cargo build
cargo build --release
```

### Build specific crate
```bash
cargo build -p mbf-core
cargo build -p mbf-transformations
```

### Run tests
```bash
cargo test                    # All tests
cargo test -p mbf-core       # Core tests only
```

### Check compilation
```bash
cargo check                   # Check all
cargo check -p mbf-io        # Check I/O only
```

## Rollback Plan

If the migration encounters issues:

1. All original files are preserved in `src/`
2. New workspace files are in `crates/`
3. Original `Cargo.toml` is preserved as `Cargo.toml.original`
4. Simply restore original `Cargo.toml` and delete `crates/` directory

## Performance Expectations

### Compile Time Improvements (estimated)

**Clean build:** Similar to current (~2-3 minutes)
- More parallelization may offset overhead

**Incremental build (change in transformations):**
- Current: ~30-60 seconds (recompiles most of the crate)
- Expected: ~10-20 seconds (only transformations + main binary)

**Incremental build (change in I/O):**
- Current: ~30-60 seconds
- Expected: ~5-10 seconds (only I/O + main binary, transformations unaffected)

**Test iteration:**
- Current: Compile all + run tests
- Expected: Compile changed crate + run relevant tests

## Next Steps

1. Review this proposal
2. Decide on migration approach (all-at-once vs. phased)
3. Execute migration phases
4. Update documentation and CI
5. Monitor compile time improvements

## Questions?

- Should we move more code into the crates initially?
- Do we need even finer-grained separation?
- Should cookbooks be a separate crate?
- Do we want to expose some crates for external use?
