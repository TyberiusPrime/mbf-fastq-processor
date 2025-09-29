# MBF FastQ Processor TODO

## Paper Ideas

### FastP Reproducibility Issues

- **Objective**: Demonstrate that fastp produces non-reproducible results
- **Impact**: Shows a key advantage of our tool's deterministic approach
- **Implementation**: Create test cases that expose fastp's non-deterministic behavior
https://github.com/OpenGene/fastp/issues/562
https://github.com/OpenGene/fastp/issues/552
https://github.com/OpenGene/fastp/issues/506
https://github.com/OpenGene/fastp/issues/379

### PE to SE with Overlap Analysis Comparison

- **Objective**: Compare our overlap detection with fastp's implementation
- **Technical Details**:
  - fastp uses simple offset checking for overlap detection with parameters:
    - `overlap_len_require` (default 30)
    - `overlap_diff_limit` (default 5)
    - `overlap_diff_percent_limit` (default 20%)
  - Our approach: Modified Smith-Waterman algorithm from rust-bio
- **Expected Outcome**: Show we're both more accurate and faster
- **Requirements**: Need test datasets for evaluation
https://github.com/OpenGene/fastp/issues/31

### Insert Size Histogram Analysis

- **Objective**: Implement fastp-style overlapping reads processing statistics
- **Current Status**: Once we have the merging capability, just need the statistics collection
- **Value**: Provides users with library preparation quality metrics

## Code Changes

# Quality

    Figure out the quality story. Is it 'whatever's in the file?'
    is in decoded phred, if so which format is the default and how does the user specify the right one,
    (autodetect?)


### New Transformations/Features

#### AnnotateBamWithTags

- **Purpose**: Add tag annotations to BAM files during processing
- **Dependencies**: Need to define tag format specification
- **Contra**: Might be better of as a separate tool reading tsv tables.

#### RewriteTag

- **Purpose**: Modify existing tags using regex patterns
- **Implementation**: Add regex-based tag transformation capability
- **Use Case**: Standardize tag formats or extract information from existing tags

#### StoreTagInSequence Optimization

- **Problem**: Currently discards all tag locations when growing/shrinking sequences
- **Solution**: Preserve relevant tag locations during sequence modifications
- **Benefit**: Better tag location tracking throughout pipeline


### Performance & Output

#### Compression Investigation

- **Issue**: Slow decompression performance on ERR13885883
  - Current: ~44.7s (43.07s without output)
  - Recompressed gz: 44.7s (42.39s)
  - zstd: 43.53s (24s)
- **Investigation**: Compare with fastp performance
- **Potential Solution**: Explore `gzp` crate for parallel Gzip writing

#### Parallel Decompression

- **Research**: Investigate `gzp` crate for parallel Gzip operations
- **Limitation**: Gzip format may not be amenable to parallel reading
- **Alternative**: Focus on parallel writing optimizations (little gain...)

### Quality Control & Reporting

#### Advanced Quality Metrics

- **Reads with expected error rate < 1%** (approximately Q20 average)
- **Reads with expected error rate < 0.1%** (approximately Q30 average)
- **Base quality score histograms** (FastQC-style but improved visualization)
- **Sequence length distribution histograms**

#### Overrepresented Sequence Detection

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

#### Duplication Analysis

- **Feature**: Duplication distribution reporting (frequency of duplicates)
- **Reference**: Compare with fastp's approach (samples ~1 in 20 reads up to 10k)

## Miscellaneous

### Research & Benchmarking

- **Benchmark Suite**: Comprehensive comparison against fastp, fasterq, seqstats
- **Quality Encoding Support**: Add support for solexa, illumina-1.3+, illumina-1.5+, illumina-1.8+ encodings
  (I don't even know where the differences are)

### Documentation & Standards

- **Adapter Sequence Research**:
  - Study cutadapt algorithms and adapter types
  - Reference Illumina adapter sequences document
  - Understand adapter-based trimming mechanisms

### Advanced Features (Lower Priority)

- **Order Shuffling**: Implement read order randomization (long range is difficult to implement)
- **True Duplicate Collapse**: Remove identical sequences (=same name. dubious utility)
- **High K-mer Read Removal**: Multi-pass normalization (reference: fasten_normalize)
- **Progress Display**: Modify Progress to avoid new line each iteration

### External Tool Integration

- **SeqKit Comparison**: Review seqkit usage patterns for feature ideas
- **SeqStats Analysis**: Study seqstats for additional statistics to implement
- **Niffler Integration**: Explore niffler for compression writer improvements

### Separator Configuration

- **Question**: Do we need separators on `ExtractRegions`, or is store-in-comment sufficient?
- **Related**: Should `ExtractAnchor` be renamed to `ExtractRelativeToTag`?

### Read Overlap Detection (BD Rhapsody Style)

- **Algorithm**: Modified Knuth-Morris-Pratt substring search
- **Parameters**:
  - Maximum mismatch rate: 9% (configurable)
  - Minimum overlap length: 25 bases
- **Process**:
  1. Scan read1 right-to-left on reverse complement of read2
  2. Find closest offset with lowest mismatches below threshold
  3. Split merged read according to R1 minimum length + bead capture sequence length
- **Benefit**: Prevent downstream mis-alignment and mis-assembly
- **Metrics**: Calculate overlap detection percentage for troubleshooting 


### What is hybseq, 
why would it need kmer trimming?
https://github.com/OpenGene/fastp/issues/590



# good illustration of why graph based pipeline definition actually matters
https://github.com/OpenGene/fastp/issues/575

# in the reports, add some thousand separators to large numbers
