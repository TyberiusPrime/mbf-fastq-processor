## Project Overview

mbf_fastq_processor is a Rust-based FASTQ processing tool that filters, samples, slices, demultiplexes, 
and performs various analyses on FASTQ files. 

It uses TOML configuration files to define processing pipelines with multiple transformation steps. 
Hints on the TOML configuration can be found in docs/content/docs/reference/llm-guide.md.

The project emphasizes correctness, flexibility, speed, and reproducible results.

# Repository Guidelines

## Project Structure & Module Organization

The Rust workspace lives under `mbf-fastq-processor/src/`. `src/main.rs` hosts
the CLI entry point and delegates to the library in `mbf-fastq-processorsrc/lib.rs`, while
specialized pipelines sit in modules like `mbf-fastq-processor/src/transformations.rs`,
`mbf-fastq-processor/src/demultiplex.rs`, and the configs under `mbf-fastq-processor/src/config/`.  Integration assets
reside in `test_cases/` (fixture FASTQs, expected outputs) and are orchestrated
by harnesses under `mbf-fastq-processor/tests/`. 

Developer utilities and scripts live in `dev/`,
documentation drafts in `docs/`, and benchmark harnesses in `benchmarks/`.

## Build, Test, and Development Commands

Use Cargo for day-to-day work: `cargo build` for debug builds, `cargo build
--release` for optimized artifacts, and `cargo run -- <config.toml>` to
exercise the CLI locally. `cargo check` gives a fast type check, while `cargo
clippy --all-targets -- -D clippy::pedantic` enforces lint compliance. When
fixtures change, run `dev/update_tests.py` before executing `cargo test` to
regenerate derived files; add `-- --ignored` to cover long-running cases.
Coverage reports come from `python3 dev/coverage.py --summary` or `--html`. A
reproducible toolchain is available through `nix develop`. Run cargo through
nix if you receive 'unknown command: cargo' errors.

To run an individual test, cd to the directory and run `cargo run -- verify`

## Configuration language.

The tests are all in our TOML configuration format.
Read `docs/content/docs/reference/llm-guide.md` for an overview.

## Coding Style & Naming Conventions

Follow rustfmt defaults (run `cargo fmt`!). Prefer 4-space indentation, snake_case
for modules/functions, CamelCase for types, and descriptive test names like
`test_valid_demultiplex_template`. Keep public APIs documented with `///` doc
comments and favor explicit error handling via `anyhow::Result`. Run `clippy -D
clippy: pedantic` before submitting.

Each transformation step goes into it's own separate file.

## Testing Guidelines

Most tests are integration tests in `test_cases`.
The meta-tests are in `mbf-fastq-processor/tests` 
and verify that the testing / verification function works as expected,
as well as many other aspects of the repository.

Do not bother to clean up 'actual' folders in test cases, they're in .gitignore anyway.

## Commit & Pull Request Guidelines

Write concise, sentence-style commit subjects (e.g., `Verify barcodes are
disjoint`) and keep related changes together. PRs should describe the
motivation, outline validation steps (`cargo test`, coverage runs), and link
issues or research notes. Attach screenshots or sample command outputs when
behavior changes, and request review when clippy and tests are clean. Use
jujutsu (jj) for version control, following its branching and merging
conventions.

## AI Planning Notes

Store agent plans in `dev/ai_plans/<name-of-agent>/<next-number>-short-description.md` alongside the corresponding numbered history.

