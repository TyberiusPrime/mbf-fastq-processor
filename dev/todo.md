# MBF FastQ Processor TODO

## Paper Ideas

### FastP Reproducibility Issues
- **Objective**: Demonstrate that fastp produces non-reproducible results
- **Impact**: Shows a key advantage of our tool's deterministic approach
- **Implementation**: Create test cases that expose fastp's non-deterministic behavior

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

### Insert Size Histogram Analysis
- **Objective**: Implement fastp-style overlapping reads processing statistics
- **Current Status**: We have the merging capability, just need the statistics collection
- **Value**: Provides users with library preparation quality metrics

## Code Changes

### Testing & Quality
- **Fix Non-Deterministic Tests**: `test_case_head_early_termination_multi_stage_head_report_middle` needs to be made deterministic
- **Test Coverage**: Add test cases for `FilterTooManyN(All)`

### New Transformations/Features

#### AnnotateBamWithTags
- **Purpose**: Add tag annotations to BAM files during processing
- **Dependencies**: Need to define tag format specification
- **Contra**: Might be better of as a separate tool reading tsv tables.

#### ConcatTags  
- **Purpose**: Combine multiple tags into a single tag
- **Requirement**: Needs support for 'location-less' tags
- **Use Case**: Simplify tag management in complex pipelines
- **Advantage**: We could get rid of extractregions instead have multiple ExtractRegion
                 I'm not convinced this is a good idea.

#### RewriteTag
- **Purpose**: Modify existing tags using regex patterns
- **Implementation**: Add regex-based tag transformation capability
- **Use Case**: Standardize tag formats or extract information from existing tags

#### StoreTagInSequence Optimization
- **Problem**: Currently discards all tag locations when growing/shrinking sequences
- **Solution**: Preserve relevant tag locations during sequence modifications
- **Benefit**: Better tag location tracking throughout pipeline

#### ExtractPolyTail vs TrimPolyTail
- **Decision Needed**: Should we have `ExtractPolyTail` in replacement to `TrimPolyTail`?
- **Consideration**: Different use cases - extraction for analysis vs removal for cleanup.
                     But extract + TrimAtTag would be the same as TrimPolyTail.


### Inspection Compression
- Add compression support to inspect mode

### Architecture Improvements

#### Filter Inversion Consistency
- **Problem**: Inconsistent inversion support across filters
  - Some filters can invert (e.g., `FilterOtherFile`)
  - Others are inverses of each other (e.g., `FilterMinLen`, `FilterMaxLen`)
- **Solution**: Add consistent `invert` flag to all filters
- **Benefit**: Cleaner, more intuitive filter configuration

#### Multi-File Input Support
- **Current Limitation**: Limited to read1/read2/index1/index2 structure
- **Goal**: Support arbitrary number of input files
- **Scope**: Large refactoring task
- **Alternative**: At minimum support read1, (read2), index1, no index2 with `keep_index`

#### ExtractLength Target Specification
- **Question**: Should `ExtractLength` support `TargetPlusAll` parameter?
- **Impact**: Would allow more flexible length extraction patterns

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
- **Alternative**: Focus on parallel writing optimizations

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
- **Problems**:  Difficult to validate
- **Other ideas**: How does FASTQC do it?

#### Duplication Analysis
- **Feature**: Duplication distribution reporting (frequency of duplicates)
- **Reference**: Compare with fastp's approach (samples ~1 in 20 reads up to 10k)

## Miscellaneous

### Research & Benchmarking
- **Benchmark Suite**: Comprehensive comparison against fastp, fasterq, seqstats
- **Maximum Read Length Testing**: Validate with PacBio long-read data
- **Quality Encoding Support**: Add support for solexa, illumina-1.3+, illumina-1.5+, illumina-1.8+ encodings

### Documentation & Standards
- **Adapter Sequence Research**: 
  - Study cutadapt algorithms and adapter types
  - Reference Illumina adapter sequences document
  - Understand adapter-based trimming mechanisms
- **Parsing Test Cases**: Create HTML/markdown documentation parsing tests (analog to template tests)

### Advanced Features (Lower Priority)
- **Order Shuffling**: Implement read order randomization (long range is difficult to implement)
- **True Duplicate Collapse**: Remove identical sequences (=same name. evaluate utility)
- **High K-mer Read Removal**: Multi-pass normalization (reference: fasten_normalize)
- **Progress Display**: Modify Progress to avoid new line each iteration

### External Tool Integration
- **SeqKit Comparison**: Review seqkit usage patterns for feature ideas
- **SeqStats Analysis**: Study seqstats for additional statistics to implement
- **Niffler Integration**: Explore niffler for compression writer improvements

### Separator Configuration
- **Question**: Do we need separators on `ExtractRegions`, or is store-in-comment sufficient?
- **Related**: Should `ExtractAnchor` be renamed to `ExtractRelativeToTag`?

### Code Quality
- **DNA Storage**: Store DNA as bytestring for prettier printing in transformations
- **Tag vs Filter Decision**: Review all filters to determine which should be tags instead

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
- **Metrics**: Calculate overlap detection percentage for troubleshooting should ExtractAnchor be 'ExtractRelativeToTag' ?
