// Paste this into https://godbolt.org/ to see vectorization in action
// Set compiler flags: -C opt-level=3 -C target-cpu=native

// Simplified version of our IUPAC matching for visualization
#[inline(never)]  // Prevent inlining so we can see the assembly
pub fn iupac_hamming_distance_simple(pattern: &[u8], text: &[u8]) -> usize {
    let mut dist = 0;

    for (a, b) in pattern.iter().zip(text.iter()) {
        // Quick path for exact match
        if a == b {
            continue;
        }

        // Check IUPAC compatibility
        let is_match = matches!(
            (*a, *b),
            // Case insensitive matches
            (b'A', b'a') | (b'a', b'A')
            | (b'C', b'c') | (b'c', b'C')
            | (b'G', b'g') | (b'g', b'G')
            | (b'T', b't') | (b't', b'T')
            // IUPAC codes
            | (b'R', b'A' | b'G' | b'a' | b'g')
            | (b'Y', b'C' | b'T' | b'c' | b't')
            | (b'N', _)  // N matches anything
        );

        if !is_match {
            dist += 1;
        }
    }

    dist
}

// Test function to generate some assembly
pub fn test_function() -> usize {
    let pattern = b"ATCGNRYSWKM";
    let text = b"ATCGAAAGGGTTCCCAAAGGGTTCCCAAAGGGTTCCC";

    iupac_hamming_distance_simple(pattern, &text[0..pattern.len()])
}

// What to look for in the assembly output:
//
// GOOD (Vectorized):
//   vmovdqu   ymm0, ymmword ptr [...]     # AVX2: load 32 bytes
//   vpcmpeqb  ymm1, ymm0, ymm2            # AVX2: compare 32 bytes in parallel
//   vptest    ymm0, ymm1                  # AVX2: test results
//
// OKAY (Partially vectorized):
//   movdqu    xmm0, xmmword ptr [...]     # SSE2: load 16 bytes
//   pcmpeqb   xmm1, xmm0                  # SSE2: compare 16 bytes
//
// NOT IDEAL (Scalar):
//   movzx     eax, byte ptr [...]         # Load 1 byte
//   cmp       al, ...                      # Compare 1 byte
//   je        .L...                        # Branch per byte
