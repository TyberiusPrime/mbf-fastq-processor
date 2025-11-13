# Workspace Structure Summary

## Quick Overview

The proposed workspace splits `mbf-fastq-processor` into **5 focused crates**:

```
┌─────────────────────────────────────────────────────────────┐
│                    mbf-fastq-processor                       │
│                   (CLI + Pipeline + Main)                    │
│                        ~2,000 LoC                            │
└─────────────────────────────────────────────────────────────┘
                             ▲
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────────────┐   ┌────────────────┐   ┌──────────────┐
│  mbf-config   │   │mbf-transforms  │   │   mbf-io     │
│ (TOML parse)  │   │ (All filters,  │   │ (File I/O,   │
│  ~800 LoC     │   │  trimmers,     │   │  parsers,    │
└───────────────┘   │  demux, etc.)  │   │  output)     │
        │           │  ~5,000 LoC    │   │  ~1,500 LoC  │
        │           └────────────────┘   └──────────────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             ▼
                    ┌────────────────┐
                    │   mbf-core     │
                    │  (FastQRead,   │
                    │   DNA utils)   │
                    │   ~1,000 LoC   │
                    └────────────────┘
```

## Files Created

### Workspace Configuration
- `Cargo.toml.workspace` - Workspace root with shared dependencies
- `Cargo.toml.main` - Updated main crate configuration
- `WORKSPACE_MIGRATION.md` - Detailed migration guide (this file)

### Crate Structure
```
crates/
├── mbf-core/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs (skeleton with exports)
│
├── mbf-config/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs (skeleton with Config struct)
│
├── mbf-io/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs (skeleton with re-exports)
│
└── mbf-transformations/
    ├── Cargo.toml
    └── src/
        └── lib.rs (skeleton with basic types)
```

## What Each Crate Contains

### 🔷 mbf-core (Foundation)
**What:** Core data structures
**Files to move:**
- `src/io/reads.rs` → `crates/mbf-core/src/reads.rs`
- `src/dna.rs` → `crates/mbf-core/src/dna.rs`

**Key types:**
- `FastQElement`, `FastQRead`, `Position`
- `Anchor`, `Hits`, `TagValue`
- `WrappedFastQRead`, `WrappedFastQReadMut`

**Why:** Zero dependencies on other parts = fastest recompilation

### ⚙️ mbf-config (Configuration)
**What:** TOML parsing and validation
**Files to move:**
- `src/config.rs` → `crates/mbf-config/src/lib.rs`
- `src/config/*.rs` → `crates/mbf-config/src/`

**Key types:**
- `Config`, `InputConfig`, `OutputConfig`
- `Segment`, `SegmentIndex`, `Options`

**Why:** Config changes don't trigger transformation recompilation

### 📁 mbf-io (Input/Output)
**What:** File I/O and parsing
**Files to move:**
- `src/io/fileformats.rs`
- `src/io/input.rs`
- `src/io/output.rs`
- `src/io/parsers.rs`
- Parts of `src/output.rs`

**Key types:**
- `InputFile`, `OutputWriter`
- `FastQParser`, `BamParser`

**Why:** I/O is stable, isolate from transformation churn

### 🔄 mbf-transformations (Processing Logic)
**What:** All transformation steps
**Files to move:**
- `src/transformations.rs` → `crates/mbf-transformations/src/lib.rs`
- All submodules in `src/transformations/`
- `src/demultiplex.rs`

**Key types:**
- `Transformation` enum
- All filter, trim, tag, report steps
- Demultiplexing logic

**Why:** Most development happens here, keep it isolated

### 🎯 mbf-fastq-processor (Main Binary)
**What:** CLI and orchestration
**Files remaining in src/:**
- `main.rs` - CLI entry point
- `lib.rs` - Public API (updated with workspace imports)
- `pipeline.rs` - Pipeline orchestration
- `interactive.rs` - Interactive mode
- `cookbooks.rs` - Cookbook management
- `list_steps.rs` - CLI helpers
- `documentation.rs` - Docs generation

**Why:** Glue code, compiles quickly

## Compilation Benefits

### Current (Monolithic)
```
Change transformation → Recompile ~9,000 LoC → 30-60 seconds
Change config         → Recompile ~9,000 LoC → 30-60 seconds
Change I/O            → Recompile ~9,000 LoC → 30-60 seconds
```

### After (Workspace)
```
Change transformation → Recompile ~5,000 + ~2,000 LoC → 10-20 seconds
Change config         → Recompile ~800 + ~2,000 LoC   → 5-10 seconds
Change I/O            → Recompile ~1,500 + ~2,000 LoC → 5-10 seconds
Change core           → Recompile everything          → ~60 seconds
```

**Core changes are rare, transformation changes are common!**

## Migration Checklist

To activate this workspace, you'll need to:

1. **Backup current state**
   ```bash
   cp Cargo.toml Cargo.toml.original
   ```

2. **Activate workspace**
   ```bash
   mv Cargo.toml.workspace Cargo.toml
   mv Cargo.toml.main src/Cargo.toml  # If needed
   ```

3. **Move files to crates** (see WORKSPACE_MIGRATION.md Phase 2-5)
   - Start with mbf-core (smallest, no dependencies)
   - Then mbf-config (depends only on mbf-core)
   - Then mbf-io (depends on mbf-core + mbf-config)
   - Then mbf-transformations (depends on all above)
   - Finally update main binary

4. **Test each phase**
   ```bash
   cargo build -p mbf-core           # After Phase 2
   cargo build -p mbf-config         # After Phase 3
   cargo build -p mbf-io             # After Phase 4
   cargo build -p mbf-transformations # After Phase 5
   cargo build                        # After Phase 6
   cargo test                         # After Phase 6
   ```

5. **Verify everything works**
   ```bash
   cargo test
   cargo build --release
   ./target/release/mbf-fastq-processor --version
   ```

## Key Design Decisions

### ✅ Clear Dependency Flow
Each crate has well-defined dependencies, no circular deps

### ✅ Workspace Dependencies
All versions managed in workspace root, ensuring consistency

### ✅ Minimal Main Binary
Keep orchestration separate from business logic

### ✅ Testable Units
Each crate can be tested independently

### ✅ Future-Proof
Structure allows adding more crates if needed (e.g., mbf-cli, mbf-reports)

## Rollback

If something goes wrong:
```bash
mv Cargo.toml.original Cargo.toml
rm -rf crates/
cargo clean
cargo build
```

## Questions for Review

1. **Granularity:** Is this split fine enough, or too fine?
2. **Core crate:** Should we split DNA utils from read types?
3. **Transformations:** Could we split filters/edits into separate crates?
4. **Visibility:** Should any crates be published to crates.io?
5. **Migration:** All-at-once or incremental per-crate?

## Next Actions

Ready to proceed? Options:

**Option A: Full Migration**
Execute all phases in WORKSPACE_MIGRATION.md

**Option B: Test Drive**
Implement just mbf-core to validate the approach

**Option C: Refinement**
Discuss and refine the structure before implementation

**Option D: Status Quo**
Keep current structure, revisit later
