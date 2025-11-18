# SIMD Instruction Quick Reference

When checking if your code is vectorized, look for these instruction patterns in the assembly output.

## X86-64 SIMD Instruction Sets (Ordered by Capability)

### SSE2 (128-bit, processes 16 bytes at once)
```asm
movdqa   xmm0, xmmword ptr [rdi]     # Move aligned 16 bytes
movdqu   xmm0, xmmword ptr [rdi]     # Move unaligned 16 bytes
pcmpeqb  xmm0, xmm1                   # Compare 16 bytes for equality
pxor     xmm0, xmm1                   # XOR 16 bytes
paddb    xmm0, xmm1                   # Add 16 bytes
```

### AVX (256-bit, processes 32 bytes at once)
```asm
vmovdqa  ymm0, ymmword ptr [rdi]     # Move aligned 32 bytes
vmovdqu  ymm0, ymmword ptr [rdi]     # Move unaligned 32 bytes
vpcmpeqb ymm0, ymm1, ymm2            # Compare 32 bytes for equality
vpxor    ymm0, ymm1, ymm2            # XOR 32 bytes
```

### AVX2 (Enhanced 256-bit integer operations)
```asm
vpmovzxbw  ymm0, xmmword ptr [rdi]  # Zero-extend bytes to words
vpmovmskb  eax, ymm0                 # Extract comparison mask
vpshufb    ymm0, ymm1, ymm2          # Byte shuffle
```

### AVX-512 (512-bit, processes 64 bytes at once)
```asm
vmovdqu64  zmm0, zmmword ptr [rdi]  # Move 64 bytes
vpcmpb     k1, zmm0, zmm1, 0        # Compare 64 bytes with mask
```

## How to Identify Vectorization Level

| Instruction Prefix | Width | Throughput | What It Means |
|-------------------|-------|------------|---------------|
| No prefix (scalar) | 1-8 bytes | 1-2 bytes/cycle | ❌ **Not vectorized** |
| `movdq*`, `p*` | 16 bytes | 16 bytes/cycle | ✅ **SSE2 vectorized** |
| `vmovdq*`, `vp*` (xmm) | 16 bytes | 16-32 bytes/cycle | ✅ **AVX vectorized** |
| `vmovdq*`, `vp*` (ymm) | 32 bytes | 32-64 bytes/cycle | ✅✅ **AVX2 vectorized** |
| `vmovdqu*`, `vp*` (zmm) | 64 bytes | 64-128 bytes/cycle | ✅✅✅ **AVX-512 vectorized** |

## Common Patterns in Vectorized Code

### Pattern Matching Loop (What We Want)
```asm
.loop:
    vmovdqu    ymm0, ymmword ptr [rdi + rcx]    # Load 32 pattern bytes
    vmovdqu    ymm1, ymmword ptr [rsi + rcx]    # Load 32 text bytes
    vpcmpeqb   ymm2, ymm0, ymm1                 # Compare all 32 bytes
    vpmovmskb  eax, ymm2                         # Extract results to register
    cmp        eax, -1                           # Check if all matched
    jne        .mismatch_found                   # Handle mismatches
    add        rcx, 32                           # Next 32 bytes
    cmp        rcx, rdx                          # Check if done
    jb         .loop                             # Continue if not done
```

### Scalar Code (What We Don't Want)
```asm
.loop:
    movzx      eax, byte ptr [rdi + rcx]        # Load 1 byte from pattern
    movzx      edx, byte ptr [rsi + rcx]        # Load 1 byte from text
    cmp        al, dl                            # Compare 1 byte
    jne        .check_iupac                      # Branch for mismatch
    inc        rcx                               # Next byte
    cmp        rcx, r8                           # Check if done
    jb         .loop                             # Continue
```

## Real Example from Optimized Code

Here's what good vectorization looks like for byte comparison:

```asm
example::compare_bytes:
    # Setup
    xor     eax, eax          # dist = 0
    xor     ecx, ecx          # index = 0

.LBB0_1:
    # Load and compare 16 bytes at a time
    movdqu  xmm0, xmmword ptr [rsi + rcx]    # Load 16 bytes from pattern
    movdqu  xmm1, xmmword ptr [rdi + rcx]    # Load 16 bytes from text
    pcmpeqb xmm0, xmm1                        # Compare 16 bytes in parallel
    pmovmskb edx, xmm0                        # Extract comparison mask
    not     edx                               # Invert (1 = mismatch)
    popcnt  edx, edx                          # Count mismatches
    add     eax, edx                          # Add to total
    add     rcx, 16                           # Next 16 bytes
    cmp     rcx, r8                           # Done?
    jb      .LBB0_1                           # Loop if not

    ret
```

This processes **16 bytes per iteration** instead of 1!

## Checking Your Binary

```bash
# Quick check: count SIMD instructions
objdump -d target/release/mbf-fastq-processor | grep -E "vp|movdq" | wc -l

# If the count is > 0, you have some vectorization!

# Detailed check: see which functions use SIMD
objdump -d target/release/mbf-fastq-processor | \
  grep -B 20 "vmovdqu\|vpcmpeqb" | \
  grep "^[0-9a-f]* <" | \
  sort -u
```

## Performance Impact

| Code Type | Bytes/Cycle | Relative Speed |
|-----------|-------------|----------------|
| Scalar (1 byte/iter) | 1-2 | 1× (baseline) |
| SSE2 (16 bytes/iter) | 8-16 | **8-16×** |
| AVX2 (32 bytes/iter) | 16-32 | **16-32×** |
| AVX-512 (64 bytes/iter) | 32-64 | **32-64×** |

*Actual speedup depends on memory bandwidth, CPU architecture, and code complexity.*

## What Prevents Vectorization?

1. **Complex branches inside loops**
   ```rust
   for i in 0..len {
       if some_complex_condition(data[i]) {  // ❌ Hard to vectorize
           do_something();
       } else {
           do_something_else();
       }
   }
   ```

2. **Function calls in hot loop**
   ```rust
   for i in 0..len {
       result += expensive_function(data[i]);  // ❌ Unless inlined
   }
   ```

3. **Non-contiguous memory access**
   ```rust
   for i in 0..len {
       result += data[indices[i]];  // ❌ Random access pattern
   }
   ```

4. **Data dependencies between iterations**
   ```rust
   for i in 1..len {
       data[i] = data[i] + data[i-1];  // ❌ Depends on previous iteration
   }
   ```

## Our IUPAC Code's Vectorization Potential

✅ **Good patterns in our code:**
- Simple loop over contiguous memory (`zip` iterator)
- Same operation on each element (comparison)
- No function calls in the hot path (matches! is inlined)
- Predictable memory access
- Early exit is in *outer* loop, not inner comparison loop

✅ **Expected result:**
The inner comparison loop should vectorize well with SSE2/AVX2, processing 16-32 bytes per iteration instead of 1 byte per iteration.
