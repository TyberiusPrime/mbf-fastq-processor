# Checking IUPAC Matching Vectorization

This guide shows how to verify if the Rust compiler is auto-vectorizing the IUPAC pattern matching code.

## Method 1: Using cargo-show-asm (Easiest)

```bash
# Install the tool
cargo install cargo-show-asm

# View assembly for the hot path function
cargo asm --lib --release mbf_fastq_processor::dna::iupac_hamming_distance_with_limit

# Look for SIMD instructions like:
# - movdqa, movdqu (SSE2 - 128-bit moves)
# - vpmovzx, vpcmpeqb (AVX2 - 256-bit operations)
# - vmovdqu, vpaddb (AVX - vector operations)
```

### What to Look For in Assembly

**✅ Good signs (vectorized):**
```asm
vmovdqu   ymm0, ymmword ptr [rdi + rax]    # Load 32 bytes at once (AVX2)
vpcmpeqb  ymm1, ymm0, ymm2                  # Compare 32 bytes in parallel
vpmovmskb eax, ymm1                         # Extract comparison results
```

**❌ Bad signs (scalar):**
```asm
movzx   eax, byte ptr [rdi + rcx]          # Load 1 byte at a time
cmp     al, byte ptr [rsi + rcx]            # Compare 1 byte at a time
je      .LBB                                # Branch per byte
```

## Method 2: Using Compiler Explorer (Godbolt)

1. Go to https://godbolt.org/
2. Select Rust as language
3. Paste this simplified version of the hot loop:

```rust
#[inline(never)]
pub fn iupac_distance_check(pattern: &[u8], text: &[u8]) -> usize {
    let mut dist = 0;

    for (a, b) in pattern.iter().zip(text.iter()) {
        if a == b {
            continue;
        }

        let is_match = matches!(
            (a, b),
            (b'A', b'a') | (b'a', b'A')
            | (b'C', b'c') | (b'c', b'C')
            | (b'G', b'g') | (b'g', b'G')
            | (b'T', b't') | (b't', b'T')
            | (b'N', _)
        );

        if !is_match {
            dist += 1;
        }
    }
    dist
}
```

4. Add compiler flags: `-C opt-level=3 -C target-cpu=native`
5. Look for vector instructions in the output

## Method 3: Using LLVM-MCA (Performance Analysis)

```bash
# Build in release mode
cargo build --release

# Extract the function
cargo asm --lib --release mbf_fastq_processor::dna::iupac_hamming_distance_with_limit > /tmp/iupac.asm

# Analyze with llvm-mca (if available)
llvm-mca /tmp/iupac.asm

# Look for:
# - Iterations: shows throughput per iteration
# - IPC (Instructions Per Cycle): higher is better
# - Resource pressure: SIMD units (FP0, FP1) should show usage
```

## Method 4: Runtime Performance Check

Create a benchmark to measure actual performance:

```rust
// In benches/iupac_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_iupac_matching(c: &mut Criterion) {
    let pattern = b"ATCGNRYSWKM";
    let text = b"ATCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    c.bench_function("iupac_hamming_distance", |b| {
        b.iter(|| {
            mbf_fastq_processor::dna::iupac_hamming_distance(
                black_box(pattern),
                black_box(text)
            )
        })
    });
}

criterion_group!(benches, bench_iupac_matching);
criterion_main!(benches);
```

```bash
cargo bench --bench iupac_bench
```

## Method 5: Check Object Code with objdump

```bash
# Build release binary
cargo build --release

# Disassemble the function
objdump -d target/release/libmbf_fastq_processor.rlib | \
  grep -A 100 "iupac_hamming_distance_with_limit"

# Or use llvm-objdump for better output
llvm-objdump -d --demangle target/release/libmbf_fastq_processor.rlib | \
  grep -A 100 "iupac_hamming_distance_with_limit"
```

## Method 6: Performance Counters with perf (Linux)

```bash
# Build and run a test
cargo build --release --tests

# Run with perf to count SIMD instructions
perf stat -e instructions,cycles,\
fp_arith_inst_retired.scalar_single,\
fp_arith_inst_retired.128b_packed_single,\
fp_arith_inst_retired.256b_packed_single \
cargo test --release test_find_iupac

# Look for non-zero counts in 128b or 256b packed operations
```

## What Affects Auto-Vectorization

### ✅ Helps Vectorization
- Simple loops with predictable iteration count
- No early returns inside the loop (our early exit is fine - it's in the outer loop)
- Operations on slices/arrays with known or bounded length
- Data-parallel operations (same operation on each element)
- Using `#[inline]` on hot functions

### ❌ Prevents Vectorization
- Complex control flow inside loops
- Function calls inside loops (unless inlined)
- Non-contiguous memory access
- Variable-length patterns
- Heavy use of branches per iteration

## Our Code's Vectorization Potential

### Strong Points
```rust
// This loop is VERY vectorization-friendly:
for (a, b) in iupac_reference.iter().zip(atcg_query.iter()) {
    if a == b { continue; }  // Simple comparison

    let is_match = matches!(...);  // Compiled to lookup table

    if !is_match { dist += 1; }
}
```

**Why it vectorizes well:**
1. Simple iteration over contiguous memory
2. The `matches!` macro becomes a lookup table or simple comparisons
3. No function calls in the hot path
4. Predictable memory access pattern
5. The compiler can unroll this loop

### Potential Improvement

If auto-vectorization isn't happening, you can use explicit SIMD:

```rust
#[cfg(target_feature = "avx2")]
unsafe fn iupac_distance_avx2(pattern: &[u8], text: &[u8]) -> usize {
    use std::arch::x86_64::*;

    // Process 32 bytes at a time with AVX2
    // ...
}
```

But try auto-vectorization first! Modern LLVM is quite good at it.

## Quick Test on Your Machine

```bash
# 1. Build with full optimizations and CPU-specific instructions
RUSTFLAGS="-C target-cpu=native" cargo build --release

# 2. Check the generated code
cargo asm --lib --release mbf_fastq_processor::dna::iupac_hamming_distance_with_limit

# 3. Look for these instruction prefixes:
#    v*    (AVX/AVX2 - 256-bit SIMD)
#    p*    (SSE - 128-bit packed operations)
#    movdq (SSE2 - 128-bit moves)
```

## Expected Results

With the current implementation and `-C opt-level=3`, you should see:
- **Partial vectorization** of the comparison loop (likely 4-8 bytes at a time with SSE2)
- **Full vectorization** if using `-C target-cpu=native` on a modern CPU (AVX2)
- **Scalar code** for the outer search loop (as expected - it has early returns)

The inner `iupac_hamming_distance_with_limit()` loop is the target for vectorization, and it's structured perfectly for it!
