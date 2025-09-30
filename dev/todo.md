# MBF FastQ Processor TODO

## Paper Ideas

### Demonstrate why the graph approach is necessary
https://github.com/OpenGene/fastp/issues/132 for an example


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

Also consider setting the incongruent bases to N,
see https://github.com/OpenGene/fastp/issues/346


### Insert Size Histogram Analysis

- **Objective**: Implement fastp-style overlapping reads processing statistics
- **Current Status**: We have the merging capability, just need the statistics collection
- **Value**: Provides users with library preparation quality metrics

#  once we have a paper, add a 'citation' command

## Code Changes

# Quality

    Figure out the quality story. Is it 'whatever's in the file?'
    is in decoded phred, if so which format is the default and how does the user specify the right one,
    (autodetect?)

[FASTQ quality encodings](https://en.wikipedia.org/wiki/FASTQ_format#Encoding), 
[SeqKit](https://github.com/shenwei356/seqkit), after some [discussion](https://github.com/shenwei356/seqkit/issues/18).
 

### Testing & Quality

- **Fix Non-Deterministic Tests**: `test_case_head_early_termination_multi_stage_head_report_middle` needs to be made deterministic

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

### Inspec### Architecture Improvements

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


# Simulate out-of-disk-error

# Split fastq into number-of-lines sized files. (interaction with demultiplex?)

# investigate https://github.com/vals/umi

# investigate [FastUinq](http://journals.plos.org/plosone/article?id=10.1371/journal.pone.0052249) ( duplicate reads for denovo analysis'?)

# investigate  http://ngsutils.org/modules/fastqutils/tile/

# investigate https://github.com/sequencing/NxTrim


# consider the ability to output 'unpaired' reads when only read1/read2 has been filtered?

# filetr by expected error https://academic.oup.com/bioinformatics/article/31/21/3476/194979

# test case: read without a name (empty name)

# add ExtractUnqualifiedBases that counts bases below threshold
GitHub Issue: [https://github.com/OpenGene/fastp/issues/128](https://github.com/OpenGene/fastp/issues/128)

# investigate  https://github.com/csf-ngs/fastqc/blob/master/Contaminants/contaminant_list.txt i

# document 'adapters you might want to cut'. Fastp appearantly has an automatic mode?
BGI:
See [here](http://seqanswers.com/forums/showthread.php?t=87647) for the thread (2nd post).

On page 7:
```
The following sequences are used to filter the adapter contamination in raw data.
Forward filter:  AAGTCGGAGGCCAAGCGGTCTTAGGAAGACAA
Reverse filter:  AAGTCGGATCGTAGCCATGTCGTTCTGTGAGCCAAGGAGTTG
```

# ExtractIUPACWithIndel https://github.com/OpenGene/fastp/issues/130)
(and 504. and 531. and 517)

# investigate https://github.com/rrwick/Porechop i
and filtlong

# ExtractPoly that finds largish homopolymers (like ExtractPolyTail, but anywhere)

# should we output --help to stdout? version does.
it's gnu standard  http://www.gnu.org/prep/standards/html_node/_002d_002dhelp.html 

# investigate (https://github.com/biocore/sortmerna)

# investigate Atropos 

# read from unaligned bam: https://gatkforums.broadinstitute.org/gatk/discussion/5990/what-is-ubam-and-why-is-it-better-than-fastq-for-storing-unmapped-sequence-data
consider (unmapped) BAM input?
How are the segments represented though.

# turn (https://github.com/OpenGene/fastp/issues/165) into test cases

# ExtractNCount
guess it could be a more generic 'extract-match-count', but what about overlapping matches?

# Method to convert a numeric tag (counts) to a rate per bp?

# we have an excessively large (500 GB) allocation when TagOtherFileName with a Zea mays sample.
It's much worse that just going 'exact'.
Investigate & fix

# Verify that report order == toml order. (before not after after)

# when progress is in the step list, also output things like 'reading all names from <other-file>'

# head still doesn't always work.
Possibly because of report?
```
[input]
read1 = ["read1.fq.gz", "read2.fq.gz""]

[output]
prefix = "transformed"
format = "Raw"
output_hash_compressed = true
output = ["read1", "read2"]
report_json = true
report_html = true


[[step]]
	action = 'Head'
	n = 100_000

[[step]]
action = "Report"
label = "report.before"
count = true
base_statistics = true
length_distribution = true
duplicate_count_per_read = true

[[step]]
action = "TagOtherFileByName"
segment = "read1"
label = "in_zea"
filename = "some_bam_file"
false_positive_rate = 0
seed = 42
ignore_unaligned = true
readname_end_chars = " "


[[step]]
	action = "FilterByBoolTag"
	label = "in_zea"
	keep_or_remove = 'remove'

[[step]]
action = "Report"
label = "report.after"
count = true
base_statistics = true
length_distribution = true
duplicate_count_per_read = true
```


# Issue a warning / error when a tag is being set but not used downstream by anything.

# Go through and find all fs:: usages and replace them with ex, because no file-name-in-error is *annoying*

# investigate SDUST
 [SDUST algorithm](https://pubmed.ncbi.nlm.nih.gov/16796549/), perhaps by linking in @lh3's [standalone C implementation](https://github.com/lh3/sdust)?
(https://github.com/OpenGene/fastp/issues/216)

(also) A low complexity filter using methods such as Entropy or Dust. The current filter does not work well on tandem repeats and similar type of low complexity sequences.
(https://github.com/OpenGene/fastp/issues/332)


# investigate preprocess.seq
[preprocess.seq](https://github.com/atulkakrana/preprocess.seq)
https://github.com/OpenGene/fastp/issues/217

# make parser robust for windows newlines

# should we have an adapter detection mode?
I'm unwilling to hook it up for auto-trim, but
it might be useful as a separate mode, like overrepresentation detection? 

# verify input files != output_files (hard to do, but hey...)

# we should introduce a marker that signals even after the fact that processing was finished (even if no reports are requested). Rename the output files or such..

# do we have a test case when segment files are of unequal length...?

# what is fastp doing with 'is_two_color_system'?

# investigate illumiana tile information
- can we extract it, report it, plot on it?

# what is illumina read chastity?
https://github.com/OpenGene/fastp/issues/310

# implement quality filtering by maximum expected error as outlined here: https://www.drive5.com/usearch/manual/exp_errs.html.

This quality filtering technique has shown to be superior to filtering techniques like mean Q-score.
See here for reference: https://doi.org/10.1093/bioinformatics/btv401.



# tripple check with sanger fastq file format 'spec'
https://academic.oup.com/nar/article/38/6/1767/3112533
(especially with regards to the comments)

# should we have a sample function that picks exactly N reads?
- what happens if there are not enough reads.
- how does it differ from head, or a head/tail combo, 
- how does it actually decide which reads to keep? Given each read 
an equal probability is difficult when you don't know how many reads will be there
And making an approximate but almost exact estimator for the number of reads
we are going to see is too much work for this.


# investigate https://github.com/nebiolabs/nebnext-single-cell-rna-seq

# devise a test case from https://github.com/OpenGene/fastp/issues/416

# add native mac arm binaries?
I think a github runner could help us here

# add test case that verifies we 'ignore' third line data after +

# do we need a 'rename reads' function, or is the regexs enough?
can we extend to stamp the 'segment number' / read number into the regex result?
Add cookbook example how to 'remove all fastq comments'

# mean q score should at least be in delogged space?

# can we support named pipes
https://github.com/OpenGene/fastp/issues/504

# make sure our parser doesn't choke on these files https://github.com/OpenGene/fastp/pull/491

# there is a tool called 'flash' for read  merging
find and investigate.
Breadcrumb: https://github.com/OpenGene/fastp/issues/513

# test case that shows we don't have a memory leak 'per read/segment/block'.

# add test case: when output is empty, files are still compressed format

# should we provide a docker container?
I have no clue what the docker story is these days

# investigate https://fulcrumgenomics.github.io/fgbio/tools/latest/CopyUmiFromReadName.html)

# deduplicate by read name
for when people have really screwed up their files?

# deduplicate by tag? is this useful

# investigate https://github.com/chanzuckerberg/czid-dedup

# hyseq / kmer filtering?
(https://github.com/OpenGene/fastp/issues/590)

# test case for https://github.com/OpenGene/fastp/issues/606 ?

# todo: for pe end data, we don't need to verify every read has the right name
a subsampling should suffice to detect most errors
