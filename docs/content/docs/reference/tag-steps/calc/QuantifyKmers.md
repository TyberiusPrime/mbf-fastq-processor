---
weight: 100
---

# QuantifyKmers

Count the number of kmers from a read that match those in a database built from reference sequences.

```toml
[[step]]
    action = "QuantifyKmers"
    label = "kmer_count"
    segment = "read1"  # Any of your input segments, or 'All'
    files = ['reference.fa', 'database.fq']
    k = 21
    min_count = 2  # optional, defaults to 1
```

This transformation:
1. Builds a kmer database from the specified sequence files (FASTA/FASTQ, compressed or uncompressed)
2. Extracts all kmers of length `k` from the reference sequences
3. Filters kmers by `min_count` (minimum occurrences in the reference to be included)
4. For each read, counts how many of its kmers appear in the database
5. Creates a numeric tag with the kmer match count

## Parameters

- **label**: Tag name to store the kmer count
- **segment**: Which segment to quantify (read1, read2, index1, index2, or 'All')
- **files**: List of sequence files to build the kmer database from
- **k**: Kmer length
- **min_count**: Minimum number of times a kmer must appear in the reference files to be included in the database (default: 1)

## Use Cases

- **Contamination detection**: Quantify reads matching known contaminant sequences
- **Quality control**: Count kmers from adapter or primer sequences
- **Species identification**: Measure presence of species-specific kmers
- **Filter by kmer content**: Use with `FilterByNumericTag` to keep/remove reads based on kmer matches

## Example

Filter reads that contain contaminant kmers:

```toml
[[step]]
    action = "QuantifyKmers"
    label = "contaminant_kmers"
    segment = "read1"
    files = ['contaminants.fa']
    k = 31
    min_count = 1

[[step]]
    action = "FilterByNumericTag"
    label = "contaminant_kmers"
    min_value = 1
    keep_or_remove = "remove"  # Remove reads with any contaminant kmers
```

## Notes

- Only valid DNA bases (A, C, G, T) are counted; kmers containing N or other ambiguous bases are skipped
- Kmer matching is case-insensitive
- The database is built once during configuration and reused for all reads

## Reverse Complement Considerations

**Current Implementation**: This transformation currently counts only forward-strand kmers. A kmer and its reverse complement are treated as distinct.

**Future Enhancement**: The bioinformatics best practice is to use "canonical kmers" where a kmer and its reverse complement are considered equivalent (counting them as the same kmer). This is because:
- DNA is double-stranded, so genomic features can appear on either strand
- Canonical counting prevents double-counting the same genomic location
- Most kmer tools use canonical kmers by default

For now, if you need to detect sequences regardless of orientation, you should include both the forward and reverse complement sequences in your reference files. A future update will add a `canonical` parameter (defaulting to `true`) to enable automatic reverse complement matching.
