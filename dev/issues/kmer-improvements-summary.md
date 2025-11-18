# Kmer Counting Improvements - Executive Summary

## TL;DR

The current kmer implementation is **functionally correct** but has:
1. **Confusing terminology** ("canonical" doesn't mean what it typically means in bioinformatics)
2. **Memory inefficiency** (2x larger database than necessary)
3. **Performance opportunities** (allocations, no SIMD, no integer encoding)
4. **Missing features** (minimizers, quality-aware extraction, caching)

## Current Implementation Analysis

### How It Works Now

With `count_reverse_complement = true`, the code:
1. For each kmer in the reference, adds **both** the forward and reverse complement to the database
2. When querying reads, looks up each kmer as-is (no canonicalization)
3. Matches succeed regardless of read orientation

**Example**:
- Reference: `AAAC` → Database: `{AAAC: 1, GTTT: 1}`
- Query `AAAC` → Match! (forward)
- Query `GTTT` → Match! (reverse complement)

This is **correct** for contamination detection where you want to catch sequences in both orientations.

### The Terminology Problem

In bioinformatics, "canonical kmers" typically means:
- Store only ONE orientation per kmer (the lexicographically smaller)
- Canonicalize queries before lookup
- Example: For kmer `AAAC` / `GTTT`, store only `AAAC`

But this implementation uses "canonical" to mean "bidirectional" or "orientation-agnostic".

**Recommendation**: Rename parameter to avoid confusion:
```toml
# Current (confusing):
count_reverse_complement = true  # alias: "canonical"

# Better:
orientation = "both"  # Options: "forward", "both", "canonical"
```

## Priority Rankings

### Priority 1: Clarify Intent (No Code Change Required)

**Action**: Update documentation to clarify that `count_reverse_complement=true` adds both orientations.

**File**: `docs/content/docs/reference/tag-steps/calc/CalcKmers.md`

Add:
```markdown
## Understanding count_reverse_complement

When `count_reverse_complement = true`:
- **Database building**: Both forward and reverse complement of each kmer are stored
- **Memory usage**: Approximately 2x the number of unique kmers
- **Query behavior**: Matches reads in both orientations
- **Use case**: Contamination detection where sequence can appear in either strand

When `count_reverse_complement = false`:
- **Database building**: Only forward kmers are stored
- **Memory usage**: Number of unique kmers as they appear
- **Query behavior**: Matches only forward orientation
- **Use case**: Directional sequencing protocols or strand-specific analysis
```

### Priority 2: Performance Optimization (High Impact, Medium Effort)

**Goal**: Reduce memory usage by 50% and improve query speed

**Approach**: True canonical kmer implementation
```rust
// Canonical kmer: always store the lexicographically smaller orientation
fn canonical_kmer(kmer: &[u8]) -> Vec<u8> {
    let revcomp = reverse_complement(kmer);
    if kmer <= revcomp.as_slice() {
        kmer.to_vec()
    } else {
        revcomp
    }
}
```

**Database building**:
```rust
if count_reverse_complement {
    let canonical = canonical_kmer(&kmer);
    *kmer_counts.entry(canonical).or_insert(0) += 1;
} else {
    *kmer_counts.entry(kmer).or_insert(0) += 1;
}
```

**Query**:
```rust
let lookup_kmer = if count_reverse_complement {
    canonical_kmer(&kmer)
} else {
    kmer
};
if kmer_db.contains_key(&lookup_kmer) { ... }
```

**Benefits**:
- 50% memory reduction (stores 1 kmer instead of 2)
- Same correctness guarantees
- Faster database building (half the insertions)
- Slightly slower queries (need to canonicalize), but cache-friendly

**Benchmark Before/After**:
```
PhiX genome (5.4kb), k=30, canonical=true:
- Current: ~5000 kmers × 2 = 10,000 entries
- Optimized: ~5000 canonical kmers = 5,000 entries
- Memory savings: ~50%
```

### Priority 3: Integer Encoding (High Impact, High Effort)

For k ≤ 32, encode kmers as u64 (2 bits per base):

**Benefits**:
- 75-85% memory reduction (u64 vs Vec<u8> + heap allocation)
- 3-5x faster hashing
- Enables SIMD operations

**Implementation**:
```rust
enum KmerKey {
    Encoded(u64),      // For k ≤ 32
    Bytes(Vec<u8>),    // For k > 32
}

type KmerDb = HashMap<KmerKey, usize>;
```

**Benchmark Estimate** (k=21, PhiX):
- Current: ~40KB (10K entries × 4 bytes/entry overhead + Vec allocations)
- With encoding: ~8KB (5K entries × 16 bytes/entry)
- Memory savings: ~80%

### Priority 4: Feature Additions

#### 4a. Coverage Metric
Instead of raw count, report fraction of read covered:

```toml
[[step]]
    action = "CalcKmers"
    metric = "coverage"  # vs "count" (default)
```

**Use case**: Distinguish reads with scattered matches vs uniformly covered reads.

#### 4b. Kmer Frequency Weighting
Weight matches by kmer rarity:

```toml
[[step]]
    action = "CalcKmers"
    weighting = "frequency"  # vs "boolean" (default)
```

**Calculation**:
```rust
// Instead of: count += 1
// Use: score += kmer_db[kmer] (the frequency in reference)
```

**Use case**: Distinguish conserved regions from repeats.

#### 4c. Database Caching
Save preprocessed databases to avoid re-parsing large references:

```rust
// Auto-cache based on file hash + parameters
let cache_key = hash_files_and_params(&files, k, canonical);
let cache_path = format!(".mbf_cache/kmers_{cache_key}.bin");

if cache_exists_and_valid(cache_path) {
    db = load_from_cache(cache_path)?;
} else {
    db = build_kmer_database(...)?;
    save_to_cache(cache_path, &db)?;
}
```

**Benchmark** (Human genome, k=31):
- First run: ~5 minutes to build database
- Subsequent runs: ~1 second to load from cache

#### 4d. Quality-Aware Extraction
Skip low-quality kmers:

```toml
[[step]]
    action = "CalcKmers"
    min_base_quality = 20  # Skip kmers with any Q<20 base
```

**Benefit**: Reduce false positives from sequencing errors.

## Implementation Roadmap

### Phase 1: Documentation (1-2 hours)
- [ ] Clarify `count_reverse_complement` behavior in docs
- [ ] Add examples showing orientation handling
- [ ] Document memory usage expectations

### Phase 2: Canonical Optimization (4-6 hours)
- [ ] Implement true canonical kmer logic
- [ ] Add unit tests comparing old/new behavior
- [ ] Benchmark memory and speed improvements
- [ ] Update test cases with expected output

### Phase 3: Integer Encoding (8-12 hours)
- [ ] Implement u64 encoding for k≤32
- [ ] Create enum to handle both encoded and byte-based kmers
- [ ] Benchmark improvements
- [ ] Add property tests (encoded vs byte-based should match)

### Phase 4: Feature Additions (2-4 hours each)
- [ ] Coverage metric
- [ ] Frequency weighting
- [ ] Database caching
- [ ] Quality-aware extraction

## Testing Strategy

### Unit Tests Needed
```rust
#[test]
fn test_canonical_vs_bidirectional() {
    // Verify canonical uses 50% memory
}

#[test]
fn test_orientation_matching() {
    // Forward, reverse, and both orientations
}

#[test]
fn test_min_count_with_canonical() {
    // Ensure counts accumulate correctly
}

#[test]
fn test_integer_encoding_correctness() {
    // Encoded and byte-based give same results
}
```

### Integration Tests
- PhiX detection (existing cookbook)
- Large genome (e.g., E. coli chromosome)
- Multiple reference files
- Edge cases: k=1, k=32, k=33, palindromic kmers

### Benchmarks
```bash
# Database building
cargo bench --bench kmer_db_build

# Query performance
cargo bench --bench kmer_query

# Memory profiling
valgrind --tool=massif target/release/mbf-fastq-processor config.toml
```

## References

- [Minimap2 canonical kmers](https://github.com/lh3/minimap2)
- [Jellyfish kmer counter](https://github.com/gmarcais/Jellyfish)
- [BBDuk kmer filtering](https://jgi.doe.gov/data-and-tools/software-tools/bbtools/)

## Questions for Discussion

1. **Breaking change**: Should we change the default behavior to true canonical (memory efficient) and require users to opt-in to bidirectional?

2. **API design**: Should we support multiple orientation modes?
   ```toml
   orientation = "forward"    # Current default (count_reverse_complement=false)
   orientation = "both"       # Current canonical=true
   orientation = "canonical"  # Proposed optimization
   ```

3. **Backwards compatibility**: How to handle existing TOML configs using `canonical = true`?
