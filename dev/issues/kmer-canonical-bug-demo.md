# Canonical Kmer Bug Demonstration

## The Bug

The current implementation incorrectly handles canonical kmers by adding BOTH the forward and reverse complement to the database, rather than just the canonical (lexicographically smaller) version.

## Demonstration

### Setup
Create a simple test with a kmer "AAAC" (forward) and its reverse complement "GTTT":

**Reference sequence**: `AAAC` (contains forward kmer)
**Query sequence**: `GTTT` (contains reverse complement kmer)

### Expected Behavior (Correct)

With `count_reverse_complement = true`:
1. **Database building**:
   - Extract kmer "AAAC" from reference
   - Compute revcomp: "GTTT"
   - Compare: "AAAC" < "GTTT" lexicographically
   - Store canonical kmer "AAAC" with count = 1
   - Database contains: {"AAAC": 1}

2. **Query**:
   - Extract kmer "GTTT" from query
   - Convert to canonical: min("GTTT", "AAAC") = "AAAC"
   - Lookup "AAAC" in database: **Found** ✓
   - Result: 1 match

### Current Behavior (Buggy)

With `count_reverse_complement = true`:
1. **Database building**:
   - Extract kmer "AAAC" from reference
   - Compute revcomp: "GTTT"
   - Add revcomp: {"GTTT": 1}
   - Add forward: {"GTTT": 1, "AAAC": 1}  ❌ Both added!
   - Database contains: {"AAAC": 1, "GTTT": 1}

2. **Query**:
   - Extract kmer "GTTT" from query
   - Lookup "GTTT" in database (no canonicalization): **Found** ✓
   - Result: 1 match

### Why This Seems to "Work"

The bug appears to work correctly because:
- Database has both orientations, so queries in either direction succeed
- However, this causes several problems:

## Problems Caused by the Bug

### Problem 1: Database is 2x Larger Than Necessary

```
Reference: AAACGTTT (contains both AAAC and GTTT)
k = 4, canonical = true

Current (buggy):
Database = {
    "AAAC": 1, "GTTT": 1,  # Forward kmer + its revcomp
    "AACG": 1, "CGTT": 1,  # Forward kmer + its revcomp
    "ACGT": 1, "ACGT": 1,  # Palindrome counted once (forward == revcomp)
    "CGTT": 1, "AACG": 1,  # Revcomp of previous + its forward
    "GTTT": 1, "AAAC": 1,  # Forward kmer + its revcomp
}
After deduplication: 6 entries

Correct:
Database = {
    "AAAC": 2,  # AAAC and GTTT both canonicalize to AAAC
    "AACG": 2,  # AACG and CGTT both canonicalize to AACG
    "ACGT": 1,  # Palindrome
}
3 entries (50% reduction)
```

### Problem 2: min_count Filtering Behaves Incorrectly

```toml
[[step]]
    action = "CalcKmers"
    k = 21
    canonical = true
    min_count = 5  # Keep only kmers appearing ≥5 times
```

**Scenario**: A kmer appears 3 times in forward orientation and 3 times in reverse complement (6 total)

**Current (buggy)**:
- Forward kmer: count = 3 (filtered out, < 5) ❌
- Reverse complement: count = 3 (filtered out, < 5) ❌
- Both are removed even though combined count = 6

**Correct**:
- Canonical kmer: count = 6 (kept, ≥ 5) ✓

### Problem 3: Documentation is Misleading

The documentation says:
> "min_count: Minimum number of times a kmer must appear in the reference files to be included in the database (default: 1). Sum of forward and reverse complement counts if count_reverse_complement is true."

This suggests `min_count` should consider the sum of both orientations, but the current implementation doesn't do this.

## Test Case to Verify the Bug

```rust
#[test]
fn test_canonical_kmer_bug() {
    use std::collections::HashMap;
    use super::build_kmer_database;

    // Create a temporary file with a simple sequence
    let fasta_content = ">test\nAAAACGTT\n";
    let temp_file = "/tmp/test_kmer.fa";
    std::fs::write(temp_file, fasta_content).unwrap();

    let db = build_kmer_database(
        &[temp_file.to_string()],
        4,  // k = 4
        1,  // min_count = 1
        true,  // canonical = true
    ).unwrap();

    // With the bug, we get duplicates (forward + revcomp)
    println!("Database size: {}", db.len());
    for (kmer, count) in &db {
        println!("{}: {}", String::from_utf8_lossy(kmer), count);
    }

    // AAAA and TTTT are revcomps - should have ONE entry with count=1
    // AAAC and GTTT are revcomps - should have ONE entry with count=1
    // AACG and CGTT are revcomps - should have ONE entry with count=1
    // ACGT is a palindrome - should have ONE entry with count=1
    // CGTT and AACG already counted

    // Expected: 4 unique canonical kmers
    // Current buggy behavior: ~8 entries (both orientations)

    // Check if AAAA and TTTT are both present (they shouldn't be with canonical)
    let has_aaaa = db.contains_key(b"AAAA");
    let has_tttt = db.contains_key(b"TTTT");

    if has_aaaa && has_tttt {
        panic!("Bug confirmed: Both AAAA and TTTT are in database (should only have one canonical form)");
    }
}
```

## How to Fix

See the fixes in `kmer-improvements.md` (#1 and #2).

The key changes:
1. When building database with canonical=true, only store the lexicographically smaller orientation
2. When querying with canonical=true, convert query kmers to canonical form before lookup
3. Ensure counts are accumulated correctly for both orientations
