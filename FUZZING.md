# Fuzzing Guide for mbf-fastq-processor

This document describes the fuzzing infrastructure for mbf-fastq-processor and how to use it.

## What is Fuzzing?

Fuzzing is an automated testing technique that feeds random or malformed input to a program to discover bugs, crashes, memory leaks, and security vulnerabilities. It's particularly valuable for parsers and input handling code, which are common sources of security issues.

## Why Fuzz mbf-fastq-processor?

This project processes user-provided TOML configuration files and various biological data formats (FastQ, FASTA, BAM). These are critical attack surfaces:

1. **TOML Configuration Parser**: Malformed configs could crash the tool or cause unexpected behavior
2. **FastQ Parser**: Malformed FastQ files could lead to buffer overflows, infinite loops, or panics
3. **Custom Deserializers**: DNA sequences, barcodes, and IUPAC codes have custom validation logic that needs testing

Fuzzing helps ensure the tool handles invalid or malicious input gracefully.

## Fuzz Targets

We have three main fuzz targets located in `mbf-fastq-processor/fuzz/fuzz_targets/`:

### 1. `fuzz_config_parser.rs`

**What it tests**: The TOML configuration parser using `eserde::toml::from_str::<Config>()`

**Why it's important**: This is the main entry point for user input. A bug here could allow:
- Denial of service via infinite loops or excessive memory allocation
- Code execution if the parser has memory safety issues
- Configuration injection attacks

**Coverage**:
- TOML syntax parsing
- Configuration structure validation
- Error handling in the parser

### 2. `fuzz_fastq_parser.rs`

**What it tests**: The FastQ file format parser (`mbf_fastq_processor::io::apply_to_read_sequences`)

**Why it's important**: FastQ files are the primary data input. Bugs could lead to:
- Buffer overflows when handling malformed records
- Infinite loops on unusual line endings or truncated files
- Panics on unexpected quality score encodings

**Coverage**:
- FastQ 4-line format parsing
- Buffer management and boundary conditions
- Compression format detection (via niffler)
- Line ending handling (Unix vs Windows)

### 3. `fuzz_custom_deserializers.rs`

**What it tests**: Custom deserialization logic for domain-specific types:
- DNA sequence validation (A, T, C, G, N)
- IUPAC code validation (extended nucleotide alphabet)
- Barcode configuration parsing
- Segment configuration parsing

**Why it's important**: These validators have complex logic and could have edge cases:
- DNA sequence validation bypasses
- Invalid characters in barcodes causing panics
- Regex injection in tag names or segments

**Coverage**:
- `Input` struct deserialization
- `Barcodes` struct with IUPAC validation
- `Transformation` deserialization for segment references

## Installation

### 1. Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

**Requirements**:
- Nightly Rust toolchain (cargo-fuzz requires it)
- Linux or macOS (Windows via WSL)

### 2. Install Nightly Toolchain

```bash
rustup install nightly
```

## Running Fuzz Tests

All commands should be run from the **repository root** (`/home/user/mbf-fastq-processor`).

### Quick Start: Fuzz All Targets

Run each target for 60 seconds to do a quick smoke test:

```bash
# Fuzz the config parser
cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=60

# Fuzz the FastQ parser
cargo +nightly fuzz run fuzz_fastq_parser -- -max_total_time=60

# Fuzz the custom deserializers
cargo +nightly fuzz run fuzz_custom_deserializers -- -max_total_time=60
```

### Deep Fuzzing: Extended Runs

For thorough testing, run for longer periods (hours or overnight):

```bash
# Run for 1 hour (3600 seconds)
cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=3600

# Run indefinitely (Ctrl+C to stop)
cargo +nightly fuzz run fuzz_fastq_parser
```

### Parallel Fuzzing

Speed up fuzzing by running multiple jobs in parallel:

```bash
# Run with 4 parallel workers
cargo +nightly fuzz run fuzz_config_parser -- -jobs=4

# Run with as many workers as CPU cores
cargo +nightly fuzz run fuzz_config_parser -- -jobs=$(nproc)
```

### Using a Corpus

Fuzzing works better with a seed corpus (example inputs):

```bash
# Create corpus directory
mkdir -p mbf-fastq-processor/fuzz/corpus/fuzz_config_parser

# Add sample TOML files
cp test_cases/*/input.toml mbf-fastq-processor/fuzz/corpus/fuzz_config_parser/

# Run fuzzer (it will use the corpus automatically)
cargo +nightly fuzz run fuzz_config_parser
```

For FastQ files:

```bash
mkdir -p mbf-fastq-processor/fuzz/corpus/fuzz_fastq_parser
cp test_cases/*/input/*.fastq mbf-fastq-processor/fuzz/corpus/fuzz_fastq_parser/
cargo +nightly fuzz run fuzz_fastq_parser
```

## Interpreting Results

### Success

If fuzzing runs without finding issues, you'll see output like:

```
#1000000: cov: 1234 ft: 5678 corp: 100/12Kb exec/s: 50000
```

- `cov`: Total code coverage (edges covered)
- `ft`: Features (unique code paths)
- `corp`: Corpus size (number of interesting inputs found)
- `exec/s`: Executions per second

### Crash Found

If a crash is found:

```
==12345==ERROR: AddressSanitizer: heap-buffer-overflow
artifact_prefix='./fuzz/artifacts/'; Test unit written to ./fuzz/artifacts/crash-abc123
```

The crashing input is saved to `mbf-fastq-processor/fuzz/artifacts/fuzz_<target>/crash-*`

### Reproducing a Crash

```bash
# Run the specific crashing input
cargo +nightly fuzz run fuzz_config_parser mbf-fastq-processor/fuzz/artifacts/fuzz_config_parser/crash-abc123

# Or examine it manually
cat mbf-fastq-processor/fuzz/artifacts/fuzz_config_parser/crash-abc123
```

### Debugging a Crash

1. **Examine the crash artifact** to understand what input caused the issue
2. **Run with debug symbols** to get a better stack trace
3. **Add a test case** in `tests/` that reproduces the issue
4. **Fix the bug** in the parser or validator
5. **Re-run the fuzzer** to confirm the fix

## Best Practices

### 1. Run Fuzzing Regularly

- Run fuzzing in CI/CD (see "Continuous Fuzzing" below)
- Fuzz for at least 1 hour before each release
- Fuzz overnight when adding new parser features

### 2. Maintain a Corpus

- Keep interesting inputs in the corpus directories
- Add regression test cases from bugs found
- Share corpus with the team

### 3. Coverage-Guided Fuzzing

Libfuzzer (used by cargo-fuzz) automatically:
- Prioritizes inputs that increase code coverage
- Mutates inputs to explore new code paths
- Saves interesting inputs to the corpus

Help it by:
- Providing diverse seed inputs
- Running for longer periods
- Using sanitizers (enabled by default)

### 4. Minimize Crash Cases

If you find a crash, minimize it to the smallest input:

```bash
cargo +nightly fuzz tmin fuzz_config_parser mbf-fastq-processor/fuzz/artifacts/fuzz_config_parser/crash-abc123
```

This makes debugging easier.

## Continuous Fuzzing

### GitHub Actions Integration

Add fuzzing to CI with a quick smoke test:

```yaml
- name: Run fuzzing smoke test
  run: |
    cargo install cargo-fuzz
    cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=60 -runs=10000
    cargo +nightly fuzz run fuzz_fastq_parser -- -max_total_time=60 -runs=10000
    cargo +nightly fuzz run fuzz_custom_deserializers -- -max_total_time=60 -runs=10000
```

### OSS-Fuzz Integration (Recommended)

For continuous fuzzing at scale, consider integrating with [OSS-Fuzz](https://github.com/google/oss-fuzz):
- Free for open-source projects
- Runs 24/7 on Google infrastructure
- Automatic bug reporting

## Advanced Options

### Memory Limits

Prevent the fuzzer from using too much memory:

```bash
cargo +nightly fuzz run fuzz_config_parser -- -rss_limit_mb=2048
```

### Timeout Detection

Find inputs that cause hangs:

```bash
cargo +nightly fuzz run fuzz_config_parser -- -timeout=10
```

### Dictionary-Based Fuzzing

Provide a dictionary of tokens (useful for TOML fuzzing):

```bash
echo -e '"key"\n"value"\n"[section]"\n"="' > fuzz.dict
cargo +nightly fuzz run fuzz_config_parser -- -dict=fuzz.dict
```

### Custom Sanitizers

Enable additional sanitizers:

```bash
# Address sanitizer (default)
cargo +nightly fuzz run fuzz_config_parser

# Memory sanitizer (stricter, slower)
RUSTFLAGS="-Zsanitizer=memory" cargo +nightly fuzz run fuzz_config_parser
```

## Limitations

### What Fuzzing Can't Find

- **Logic bugs**: Fuzzing finds crashes, not incorrect behavior
- **Race conditions**: Single-threaded fuzzing won't find concurrency bugs
- **Semantic errors**: Won't find issues like "incorrect quality score calculation"

Use fuzzing alongside:
- Unit tests for logic
- Integration tests for workflows
- Manual testing for usability

### Performance Considerations

Fuzzing with file I/O (like `fuzz_fastq_parser`) is slower because:
- Each iteration creates a temporary file
- I/O operations are expensive

To speed up:
- Use in-memory parsing if possible
- Run with multiple jobs: `-jobs=$(nproc)`
- Use faster storage (tmpfs/ramdisk)

## Troubleshooting

### "error: no such command: `fuzz`"

Install cargo-fuzz: `cargo install cargo-fuzz`

### "error: requires nightly toolchain"

Use `cargo +nightly`: `cargo +nightly fuzz run ...`

### Fuzzer is slow (< 1000 exec/s)

- Reduce the size of seed inputs
- Check if the code has expensive operations in the hot path
- Use `-jobs=` for parallelism

### Out of memory

- Set RSS limit: `-rss_limit_mb=2048`
- Reduce corpus size
- Close other applications

## Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/)

## Contributing

When adding new parsers or input handling:

1. **Create a fuzz target** in `mbf-fastq-processor/fuzz/fuzz_targets/`
2. **Add it to `Cargo.toml`** as a `[[bin]]` entry
3. **Document it** in this file
4. **Add seed inputs** to the corpus
5. **Run it** for at least 1 hour before submitting a PR

## License

The fuzzing infrastructure follows the same MIT license as the main project.
