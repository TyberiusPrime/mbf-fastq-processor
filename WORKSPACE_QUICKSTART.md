# Workspace Quick Reference

## 📋 Overview

Split mbf-fastq-processor into 5 crates for **faster incremental builds**:

```
mbf-core → mbf-config
           mbf-io
         → mbf-transformations
         → mbf-fastq-processor (main)
```

## 🚀 Quick Start

### Using the migration script:
```bash
# Backup and activate workspace
./dev/migrate_to_workspace.sh backup
./dev/migrate_to_workspace.sh activate

# Migrate each crate
./dev/migrate_to_workspace.sh core
./dev/migrate_to_workspace.sh config
./dev/migrate_to_workspace.sh io
./dev/migrate_to_workspace.sh transformations
./dev/migrate_to_workspace.sh main

# Test
./dev/migrate_to_workspace.sh test

# If something goes wrong
./dev/migrate_to_workspace.sh rollback
```

### Manual migration:
```bash
# 1. Backup
cp Cargo.toml Cargo.toml.original

# 2. Activate workspace
cp Cargo.toml.workspace Cargo.toml

# 3. Migrate files (see WORKSPACE_MIGRATION.md for details)
# Copy files from src/ to crates/*/src/

# 4. Test each crate
cargo build -p mbf-core
cargo build -p mbf-config
cargo build -p mbf-io
cargo build -p mbf-transformations
cargo build

# 5. Full test
cargo test
```

## 📦 Crate Responsibilities

| Crate | Purpose | Key Types | LoC | Deps |
|-------|---------|-----------|-----|------|
| **mbf-core** | Data structures | `FastQRead`, `FastQElement`, DNA utils | ~1K | Minimal |
| **mbf-config** | TOML parsing | `Config`, `InputConfig`, `OutputConfig` | ~800 | core, serde |
| **mbf-io** | File I/O | Parsers, readers, writers | ~1.5K | core, config |
| **mbf-transformations** | Processing | All filters, trimmers, demux | ~5K | core, config |
| **mbf-fastq-processor** | CLI + orchestration | Pipeline, CLI, interactive | ~2K | All above |

## 🔄 Import Changes

### Before (monolithic):
```rust
use crate::io::{FastQRead, FastQElement};
use crate::config::Config;
use crate::transformations::Transformation;
use crate::dna::Anchor;
```

### After (workspace):
```rust
use mbf_core::{FastQRead, FastQElement, Anchor};
use mbf_config::Config;
use mbf_transformations::Transformation;
```

## 🛠️ Build Commands

```bash
# Build everything
cargo build
cargo build --release

# Build specific crate
cargo build -p mbf-core
cargo build -p mbf-transformations

# Run tests
cargo test                      # All tests
cargo test -p mbf-core         # Core only
cargo test -p mbf-transformations  # Transformations only

# Check without building
cargo check
cargo check -p mbf-transformations

# Clean build
cargo clean
cargo build
```

## ⚡ Speed Improvements

| Change Location | Before | After | Improvement |
|----------------|--------|-------|-------------|
| Transformation | 30-60s | 10-20s | **2-3x faster** |
| Config | 30-60s | 5-10s | **5-6x faster** |
| I/O | 30-60s | 5-10s | **5-6x faster** |
| Core types | 30-60s | ~60s | Same (rare) |

## 📁 File Migration Map

### mbf-core
```
src/io/reads.rs    → crates/mbf-core/src/reads.rs
src/dna.rs         → crates/mbf-core/src/dna.rs
```

### mbf-config
```
src/config.rs      → crates/mbf-config/src/lib.rs
src/config/*.rs    → crates/mbf-config/src/*.rs
```

### mbf-io
```
src/io/fileformats.rs → crates/mbf-io/src/fileformats.rs
src/io/input.rs       → crates/mbf-io/src/input.rs
src/io/output.rs      → crates/mbf-io/src/output.rs
src/io/parsers.rs     → crates/mbf-io/src/parsers.rs
```

### mbf-transformations
```
src/transformations.rs → crates/mbf-transformations/src/lib.rs
src/transformations/*  → crates/mbf-transformations/src/*
src/demultiplex.rs     → crates/mbf-transformations/src/demultiplex.rs
```

### mbf-fastq-processor (stays in src/)
```
src/main.rs           (update imports)
src/lib.rs            (update imports)
src/pipeline.rs       (update imports)
src/interactive.rs    (update imports)
src/cookbooks.rs
src/list_steps.rs
src/documentation.rs
```

## ⚠️ Common Issues

### Issue: `cannot find type FastQRead`
**Fix:** Add `use mbf_core::FastQRead;`

### Issue: `no such external crate: crate`
**Fix:** Change `use crate::X` to `use mbf_X::Y` or `use super::X`

### Issue: Module not found
**Fix:** Ensure module is declared in crate's `lib.rs`:
```rust
pub mod reads;
pub mod dna;
```

### Issue: Circular dependency
**Fix:** Check dependency graph - must flow: core → config/io → transformations → main

## 🧪 Testing Strategy

```bash
# 1. Test incrementally as you migrate each crate
cargo build -p mbf-core && echo "✓ Core OK"
cargo build -p mbf-config && echo "✓ Config OK"
cargo build -p mbf-io && echo "✓ IO OK"
cargo build -p mbf-transformations && echo "✓ Transformations OK"
cargo build && echo "✓ Full build OK"

# 2. Run unit tests
cargo test -p mbf-core
cargo test -p mbf-transformations

# 3. Run integration tests
cargo test

# 4. Run cookbook tests (if needed)
python3 dev/update_cookbook_tests.py
nix build .#test

# 5. Check coverage
python3 dev/coverage.py --summary
```

## 🔙 Rollback

If migration fails:
```bash
# Option 1: Use script
./dev/migrate_to_workspace.sh rollback
rm -rf crates/
cargo clean

# Option 2: Manual
cp Cargo.toml.original Cargo.toml
rm -rf crates/
cargo clean
cargo build
```

## 📚 Documentation

- **WORKSPACE_SUMMARY.md** - High-level overview with diagrams
- **WORKSPACE_MIGRATION.md** - Detailed migration guide with 8 phases
- **WORKSPACE_QUICKSTART.md** - This file (quick reference)
- **dev/migrate_to_workspace.sh** - Automated migration helper

## ✅ Success Criteria

Migration is successful when:
- [ ] All crates compile: `cargo build`
- [ ] All tests pass: `cargo test`
- [ ] Interactive mode works: `./target/debug/mbf-fastq-processor interactive test.toml`
- [ ] Release builds: `cargo build --release`
- [ ] Coverage stable: `python3 dev/coverage.py`
- [ ] Incremental builds are faster (measure with `time cargo build` after changes)

## 💡 Tips

1. **Migrate incrementally** - Do one crate at a time, test, commit
2. **Start with core** - It has the fewest dependencies
3. **Use cargo check** - Faster than full builds during migration
4. **IDE support** - Rust Analyzer should work once structure is correct
5. **Commit often** - Easy to revert if something breaks

## 🎯 Next Steps

1. Read **WORKSPACE_SUMMARY.md** for the big picture
2. Review **WORKSPACE_MIGRATION.md** for detailed steps
3. Decide: Use script or manual migration?
4. Start with Phase 1: Backup and activate workspace
5. Proceed phase by phase, testing each step

## 📞 Questions?

- Why these 5 crates? See WORKSPACE_SUMMARY.md "Key Design Decisions"
- How to handle X? See WORKSPACE_MIGRATION.md "Phase N"
- Script failed? Check logs and try manual steps
- Want finer granularity? Consider splitting transformations further
