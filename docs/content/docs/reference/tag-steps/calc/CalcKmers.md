---
weight: 100
---

# CalcKmers

Count the number of kmers from a read that match those in a database built from reference sequences.

```toml
[[step]]
    action = "CalcKmers"
    out_label = "mytag"
    segment = "read1"  # Any of your input segments, or 'All'
    files = ['reference.fa', 'database.fq']
    count_reverse_complement = true # whether to also include each revcomp of a kmer in the database
    k = 21
    min_count = 2  # optional, defaults to 1
```

This transformation:
1. Builds a kmer database from the specified sequence files (all [input formatws]({{< relref "docs/reference/input-section.md" >}}))
2. Extracts all kmers of length `k` from the reference sequences
3. Filters kmers by `min_count` (minimum occurrences in the reference to be included)
4. For each read, counts how many of its kmers appear in the database
5. Creates a numeric tag with the kmer match count

## Parameters

- **out_label**: Tag name to store the kmer count
- **segment**: Which segment to quantify (read1, read2, index1, index2, or 'All')
- **files**: List of sequence files to build the kmer database from
- **count_reverse_complement**: (alias: "canonical") Whether to use canonical kmers (orientation-agnostic matching)
- **k**: Kmer length
- **min_count**: Minimum number of times a kmer must appear in the reference files to be included in the database (default: 1)

## Understanding count_reverse_complement

When `count_reverse_complement = true` (also aliased as `canonical = true`):

**Database building:**
- Stores only the **canonical form** of each kmer (the lexicographically smaller of forward and reverse complement)
- Example: For kmer `AAAC` (forward) and `GTTT` (reverse complement), only `AAAC` is stored (since `AAAC` < `GTTT`)
- Both orientations increment the same counter, so counts accumulate correctly
- **Memory usage**: ~50% reduction compared to storing both orientations separately

**Query behavior:**
- Query kmers are converted to canonical form before database lookup
- Matches reads in **both orientations** (forward and reverse complement)
- A read with sequence `GTTT` will match if the database contains `AAAC` (its canonical form)

**Use case**: Contamination detection, adapter removal, or any application where sequence orientation is unknown or irrelevant

When `count_reverse_complement = false`:

**Database building:**
- Stores kmers exactly as they appear in the reference
- Only forward orientation is indexed

**Query behavior:**
- Queries match only the exact orientation present in the database
- A read with `GTTT` will NOT match a database entry for `AAAC`

**Use case**: Directional sequencing protocols, strand-specific analysis, or when orientation matters

## min_count Filtering

The `min_count` parameter filters kmers that appear fewer than N times in the reference:

- With `count_reverse_complement = true`: Counts from both orientations are combined before filtering
  - Example: If `AAAC` appears 3 times and `GTTT` (its revcomp) appears 2 times, the canonical kmer has count = 5
- With `count_reverse_complement = false`: Only counts from the exact kmer as it appears
  - Example: `AAAC` with 3 occurrences and `GTTT` with 2 occurrences are counted separately

## Use Cases

- **Contamination detection**: Quantify or filter reads matching known contaminant sequences (e.g., PhiX, E. coli)
- **Quality control**: Count kmers from adapter or primer sequences
- **Species identification**: Measure presence of species-specific kmers
- **Off-target detection**: Identify reads from unintended species in capture experiments

## Performance Considerations

- **Memory usage**: For k=21 and PhiX genome (~5kb), expect ~2,500 unique canonical kmers vs ~5,000 non-canonical kmers
- **Recommendation**: Use `count_reverse_complement = true` for most applications to reduce memory usage by ~50%
- **k value selection**:
  - Small k (15-21): More sensitive, faster, less specific
  - Medium k (21-31): Balanced sensitivity and specificity (recommended for most uses)
  - Large k (31+): More specific, slower, less sensitive

## Notes

- Only kmers with valid DNA bases (A, C, G, T) are counted; kmers containing N or other ambiguous bases are skipped
- Kmer matching is case-insensitive
- The canonical kmer for palindromic sequences (e.g., `ACGT` ⇔ `ACGT`) is the sequence itself

