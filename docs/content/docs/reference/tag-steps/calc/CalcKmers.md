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
    filename = ['reference.fa', 'database.fq'] # Path (string) or list of such
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
- **count_reverse_complement**: (alias: "canonical") Whether to include reverse complements of kmers in the database  ('canonical kmers')
- **k**: Kmer length
- **min_count**: Minimum number of times a kmer must appear in the reference files to be included in the database (default: 1). Sum of forward and reverse complement counts if `count_reverse_complement` is true.

## Use Cases

- **Contamination detection**: Quantify or filter reads matching known contaminant sequences
- **Quality control**: Count kmers from adapter or primer sequences
- **Species identification**: Measure presence of species-specific kmers

## Notes

- Only kmers with only valid DNA bases (A, C, G, T) are counted; kmers containing N or other ambiguous bases are skipped
- Kmer matching is case-insensitive

