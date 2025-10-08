status: open
# Overrepresented Sequence Detection

- **Algorithm**:
  1. Skip x reads for baseline
  2. Count 12-mers (2^24 possibilities) for next n reads
  3. For following n×x reads, calculate max occurrence using k-mer table
  4. Apply enrichment threshold filtering
  5. Calculate enrichment based on actual counts
  6. Remove sequences that are prefixes of others
- **Output**: Report overrepresented sequences with enrichment statistics
- **Problems**: Difficult to validate
- **Other ideas**: How does FASTQC do it?
