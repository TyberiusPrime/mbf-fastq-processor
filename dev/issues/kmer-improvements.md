# Kmer Counting Improvements

## Summary
Analysis of the current kmer counting implementation in `src/transformations/calc/kmers.rs` reveals several opportunities for correctness fixes, performance optimizations, and feature enhancements.

## Critical Bugs

### 1. Canonical Kmer Handling is Incorrect
**Location**: `kmers.rs:149-153`

**Current behavior**:
```rust
if canonical {
    let revcomp = crate::dna::reverse_complement(&kmer);
    *kmer_counts.entry(revcomp).or_insert(0) += 1;
}
*kmer_counts.entry(kmer).or_insert(0) += 1;
```

**Problem**: When `count_reverse_complement=true`, the code adds BOTH the forward kmer and its reverse complement to the database. This is incorrect!

**Expected behavior**: Canonical kmer counting should store only the lexicographically smaller of the two orientations (or consistently choose forward < revcomp). The count should represent both orientations.

**Impact**:
- Database is 2x larger than needed
- Queries will find matches twice when they shouldn't
- The `min_count` filter behaves unexpectedly

**Fix**:
```rust
if canonical {
    let revcomp = crate::dna::reverse_complement(&kmer);
    let canonical_kmer = if kmer <= revcomp { kmer } else { revcomp };
    *kmer_counts.entry(canonical_kmer).or_insert(0) += 1;
} else {
    *kmer_counts.entry(kmer).or_insert(0) += 1;
}
```

### 2. Query Counting Should Also Use Canonical Form
**Location**: `kmers.rs:170-192`

**Problem**: When querying reads (`count_kmers_in_database`), the code only checks the forward kmer. If `canonical=true` was used during database building, query kmers should also be converted to canonical form before lookup.

**Fix**:
```rust
pub fn count_kmers_in_database(
    sequence: &[u8],
    k: usize,
    kmer_db: &HashMap<Vec<u8>, usize>,
    canonical: bool,  // Add this parameter
) -> usize {
    if sequence.len() < k {
        return 0;
    }

    let mut count = 0;
    for i in 0..=(sequence.len() - k) {
        let kmer: Vec<u8> = sequence[i..i + k]
            .iter()
            .map(|&b| b.to_ascii_uppercase())
            .collect();

        let lookup_kmer = if canonical {
            let revcomp = crate::dna::reverse_complement(&kmer);
            if kmer <= revcomp { kmer } else { revcomp }
        } else {
            kmer
        };

        if kmer_db.contains_key(&lookup_kmer) {
            count += 1;
        }
    }

    count
}
```

## Performance Optimizations

### 3. Use Integer Encoding for Kmers (High Impact)
**Current**: Vec<u8> keys in HashMap
**Proposed**: u64/u128 bit-packed encoding for k ≤ 32

**Benefits**:
- 4-8x smaller memory footprint
- Faster hashing (integer vs. slice hashing)
- Faster comparisons
- Enables SIMD-accelerated kmer extraction

**Implementation**:
```rust
// Encode kmer as 2-bit packed integer
// A=00, C=01, G=10, T=11
fn encode_kmer(kmer: &[u8]) -> Option<u64> {
    if kmer.len() > 32 {
        return None;  // Fall back to Vec<u8> for k > 32
    }

    let mut encoded = 0u64;
    for &base in kmer {
        encoded <<= 2;
        encoded |= match base.to_ascii_uppercase() {
            b'A' => 0,
            b'C' => 1,
            b'G' => 2,
            b'T' => 3,
            _ => return None,  // Invalid base
        };
    }
    Some(encoded)
}

// Fast reverse complement on encoded kmer
fn reverse_complement_encoded(mut kmer: u64, k: usize) -> u64 {
    // Complement: XOR with 0b01...01 (swaps A<->T, C<->G)
    kmer ^= (1u64 << (2 * k)) - 1;

    // Reverse: bit manipulation trick
    // ... (implementation omitted for brevity)

    kmer
}
```

**Alternative**: Use the `bio` crate's kmer implementation or `needletail` which already has optimized kmer handling.

### 4. Avoid Repeated Allocations
**Problem**: Every kmer creates a new Vec<u8> allocation

**Current hotspots**:
- `kmers.rs:142-145` (database building)
- `kmers.rs:181-184` (query counting)

**Fix**: Reuse a single buffer:
```rust
let mut kmer_buf = vec![0u8; k];
for i in 0..=(seq.len() - k) {
    kmer_buf.copy_from_slice(&seq[i..i + k]);
    for byte in &mut kmer_buf {
        *byte = byte.to_ascii_uppercase();
    }
    // Use kmer_buf...
}
```

Or with integer encoding, avoid allocation entirely.

### 5. Skip Invalid Kmers Earlier
**Problem**: Code validates kmers contain only ATCG after allocation and uppercase conversion

**Fix**: Fast validation before processing:
```rust
// Check if slice contains only valid DNA bases (no N, etc.)
#[inline]
fn is_valid_dna(seq: &[u8]) -> bool {
    seq.iter().all(|&b| matches!(b | 0x20, b'a' | b'c' | b'g' | b't'))
}

// In the loop:
if !is_valid_dna(&seq[i..i + k]) {
    continue;
}
```

### 6. Parallel Database Building
**Problem**: Large reference genomes processed sequentially

**Opportunity**: Use rayon to parallelize kmer extraction per sequence:
```rust
use rayon::prelude::*;

let kmer_counts: HashMap<Vec<u8>, usize> = files
    .par_iter()
    .map(|file_path| {
        // Extract kmers from this file
        let mut local_counts = HashMap::new();
        // ... extraction logic ...
        local_counts
    })
    .reduce(HashMap::new, |mut acc, counts| {
        // Merge hashmaps
        for (kmer, count) in counts {
            *acc.entry(kmer).or_insert(0) += count;
        }
        acc
    });
```

## Feature Enhancements

### 7. Add Minimizer Support
**Use case**: More memory-efficient representation for large genomes

**Benefit**: 10-20x memory reduction with minimal accuracy loss for contamination detection

**Example API**:
```toml
[[step]]
    action = "CalcKmers"
    out_label = "phix"
    k = 21
    use_minimizers = true
    window_size = 10  # Extract 1 kmer per 10bp window
```

### 8. Add Kmer Frequency Weighting
**Problem**: Current implementation treats all kmers equally (presence/absence)

**Enhancement**: Use kmer frequency from database for scoring:
```rust
// Instead of counting matches, sum the frequencies
let mut score = 0;
for kmer in read_kmers {
    if let Some(&frequency) = kmer_db.get(&kmer) {
        score += frequency;  // Weight by how common this kmer is
    }
}
```

**Use case**: Distinguish highly-conserved regions from repetitive sequences.

### 9. Add Kmer Density/Coverage Metric
**Current**: Count of matching kmers
**Enhancement**: Fraction of read covered by matching kmers

```toml
[[step]]
    action = "CalcKmers"
    out_label = "phix_coverage"
    metric = "coverage"  # vs "count" (default)
```

**Calculation**:
```rust
let covered_bases = count_covered_bases(sequence, k, kmer_db);
let coverage = covered_bases as f64 / sequence.len() as f64;
```

### 10. Quality-Aware Kmer Extraction
**Problem**: Low-quality bases create spurious kmers

**Enhancement**: Skip kmers containing bases below quality threshold:
```toml
[[step]]
    action = "CalcKmers"
    min_base_quality = 20  # Skip kmers with any base < Q20
```

### 11. Add Kmer Database Caching
**Problem**: Same database rebuilt on every run

**Enhancement**:
```rust
// Save preprocessed database
#[derive(Serialize, Deserialize)]
struct KmerDatabase {
    k: usize,
    canonical: bool,
    kmers: HashMap<Vec<u8>, usize>,
    created: SystemTime,
    source_files: Vec<(String, FileHash)>,
}

// Check if cached database is valid
if cache_is_valid(cache_path, &config.files) {
    db = load_cached_database(cache_path)?;
} else {
    db = build_kmer_database(...)?;
    save_cached_database(cache_path, &db)?;
}
```

### 12. Support for Multiple Kmer Sizes
**Use case**: Multi-scale contamination detection

```toml
[[step]]
    action = "CalcKmers"
    out_label = "phix"
    k_values = [15, 21, 31]  # Generate 3 tags: phix_k15, phix_k21, phix_k31
```

## Code Quality Improvements

### 13. Add Comprehensive Unit Tests
**Missing tests**:
- Canonical kmer handling (forward vs reverse)
- Behavior with sequences containing N
- Edge cases (k=1, k=read length, k>read length)
- min_count filtering edge cases
- Performance benchmarks

### 14. Improve Documentation
**Add to CalcKmers.md**:
- Explanation of canonical kmers and when to use them
- Guidance on choosing k (trade-offs between specificity and sensitivity)
- Memory usage estimates
- Examples showing min_count filtering behavior
- Performance characteristics for different k values

### 15. Add Input Validation
**Missing checks**:
```rust
// Validate k is reasonable
if k > 32 {
    eprintln!("Warning: k={k} is large. Consider k≤32 for better performance");
}
if k < 15 {
    eprintln!("Warning: k={k} may have many false positives. Consider k≥15");
}

// Warn about memory usage
let estimated_kmers = estimate_unique_kmers(&files, k)?;
let estimated_mb = estimated_kmers * std::mem::size_of::<(Vec<u8>, usize)>() / 1_000_000;
if estimated_mb > 1000 {
    eprintln!("Warning: Estimated database size: {estimated_mb}MB");
}
```

## Implementation Priority

**High Priority** (Correctness):
1. Fix canonical kmer bug (#1, #2)
2. Add unit tests for canonical behavior (#13)

**Medium Priority** (Performance):
3. Integer encoding for k≤32 (#3)
4. Avoid repeated allocations (#4)
5. Skip invalid kmers earlier (#5)

**Low Priority** (Features):
6. Kmer frequency weighting (#8)
7. Coverage metric (#9)
8. Database caching (#11)
9. Improved documentation (#14)

**Future Enhancements**:
10. Minimizers (#7)
11. Quality-aware extraction (#10)
12. Multiple k values (#12)
13. Parallel database building (#6)

## Benchmarking Recommendations

Create benchmarks for:
1. Database building with various k and genome sizes
2. Query performance with different database sizes
3. Memory usage profiling
4. Comparison with canonical vs non-canonical

Example using criterion:
```rust
#[bench]
fn bench_build_phix_database(b: &mut Bencher) {
    b.iter(|| {
        build_kmer_database(&["phix.fa"], 31, 1, true)
    });
}
```

## Alternative Approaches to Consider

1. **Use existing kmer libraries**:
   - `needletail` has optimized kmer iteration
   - `bio::alphabets::dna::kmer` provides canonical kmer support

2. **Bloom filters**: For presence/absence queries, a counting Bloom filter uses ~10x less memory

3. **Sketching algorithms**: MinHash or HyperLogLog for approximate similarity

## References

- [BBDuk documentation](https://jgi.doe.gov/data-and-tools/software-tools/bbtools/bb-tools-user-guide/bbduk-guide/)
- [khmer: k-mer counting & filtering](https://github.com/dib-lab/khmer)
- [Minimap2 minimizers](https://github.com/lh3/minimap2)
