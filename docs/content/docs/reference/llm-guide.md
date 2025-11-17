---
title: "LLM Configuration Guide"
weight: 1
not-a-transformation: true
---

# LLM Configuration Generation Guide

This guide is optimized for Large Language Models to generate valid `mbf-fastq-processor` configurations. It provides structured information with explicit types, constraints, and patterns.

## Configuration Structure

Every configuration has 3 required sections and 2 optional sections:

```
# example-only - structure overview, not valid TOML
[input]           # REQUIRED: Define input files
[[step]]          # OPTIONAL: Processing steps (0 or more, order matters)
[output]          # REQUIRED: Output configuration
[barcodes.*]      # OPTIONAL: Barcode definitions for demultiplexing
[options]         # OPTIONAL: Global processing options
```

## Quick Start Patterns

### Pattern 1: Basic Quality Report

Generate reports without modifying sequences.

```toml
[input]
    read1 = ['sample.fastq.gz']

[[step]]
    action = 'Report'
    name = 'quality_check'
    count = true
    base_statistics = true
    length_distribution = true
    duplicate_count_per_read = true

[output]
    prefix = 'output'
    format = 'None'  # No sequence output, just reports
    report_html = true
    report_json = true
```

### Pattern 2: UMI Extraction and Preservation

Extract UMI from read1, store in comment, remove from sequence.

```toml
[input]
    read1 = ['sample_R1.fastq.gz']
    read2 = ['sample_R2.fastq.gz']

[[step]]
    action = 'ExtractRegion'
    segment = 'read1'
    start = 0
    len = 8
    out_label = 'umi'

[[step]]
    action = 'StoreTagInComment'
    in_label = 'umi'
    segment = 'read1'

[[step]]
    action = 'CutStart'
    segment = 'read1'
    n = 8

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

### Pattern 3: Adapter Trimming (3' end)

Find and trim 3' adapters using partial matching.

```toml
[input]
    read1 = ['sample.fastq.gz']

[[step]]
    action = 'ExtractIUPACSuffix'
    segment = 'read1'
    query = 'AGATCGGAAGAGC'
    min_length = 3
    max_mismatches = 1
    out_label = 'adapter'

[[step]]
    action = 'TrimAtTag'
    in_label = 'adapter'
    direction = 'End'
    keep_tag = false

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

### Pattern 4: Quality Filtering

Keep reads with at least 100 high-quality bases.

```toml
[input]
    read1 = ['sample.fastq.gz']

[[step]]
    action = 'CalcQualifiedBases'
    segment = 'read1'
    threshold = 'C'       # Phred+33 encoding: 'C' = Phred 34
    op = 'below'          # Count bases with quality >= threshold
    out_label = 'hq_bases'

[[step]]
    action = 'FilterByNumericTag'
    in_label = 'hq_bases'
    min_value = 100.0
    keep_or_remove = 'Keep'

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

### Pattern 5: Demultiplexing by Barcode

Extract barcode from index1, correct errors, split into separate files.

```toml
[input]
    read1 = ['sample_R1.fastq.gz']
    index1 = ['sample_I1.fastq.gz']

[[step]]
    action = 'ExtractRegion'
    segment = 'index1'
    start = 0
    len = 8
    out_label = 'barcode'

[[step]]
    action = 'HammingCorrect'
    in_label = 'barcode'
    out_label = 'barcode_corrected'
    barcodes = 'my_barcodes'
    max_hamming_distance = 1
    on_no_match = 'remove'

[[step]]
    action = 'Demultiplex'
    in_label = 'barcode_corrected'
    barcodes = 'my_barcodes'
    output_unmatched = true

[barcodes.my_barcodes]
    'AAAAAAAA' = 'sample_1'
    'CCCCCCCC' = 'sample_2'
    'GGGGGGGG' = 'sample_3'
    'TTTTTTTT' = 'sample_1'  # Multiple barcodes can map to same sample

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

### Pattern 6: PolyA Tail Removal

Remove polyA tails from RNA-seq reads.

```toml
[input]
    read1 = ['sample.fastq.gz']

[[step]]
    action = 'ExtractPolyTail'
    segment = 'read1'
    base = 'A'
    min_length = 10
    max_mismatch_rate = 0.1
    max_consecutive_mismatches = 3
    out_label = 'polyA'

[[step]]
    action = 'TrimAtTag'
    in_label = 'polyA'
    direction = 'End'
    keep_tag = false

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

### Pattern 7: Complex Filtering with EvalExpression

Filter based on multiple conditions: GC content and length.

```toml
[input]
    read1 = ['sample.fastq.gz']

[[step]]
    action = 'CalcGCContent'
    segment = 'read1'
    out_label = 'gc'

[[step]]
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'length'

[[step]]
    action = 'EvalExpression'
    expression = 'gc >= 0.4 && gc <= 0.6 && length >= 50'
    result_type = 'bool'
    out_label = 'pass_filter'

[[step]]
    action = 'FilterByTag'
    in_label = 'pass_filter'
    keep_or_remove = 'Keep'

[output]
    prefix = 'output'
    format = 'Fastq'
    compression = 'Gzip'
```

## Input Section

### Required Fields

```toml
# fragment - minimum required input
[input]
    read1 = ['file1.fastq', 'file2.fastq.gz', 'file3.fastq.zst']  # REQUIRED (unless using interleaved)
```

**TYPE**: `read1`, `read2`, `index1`, `index2` are arrays of file paths
**CONSTRAINT**: All arrays must have the same length
**SUPPORTED FORMATS**: FASTQ (.fq, .fastq), FASTA (.fa, .fasta), BAM (.bam)
**COMPRESSION**: Auto-detected (.gz for gzip, .zst for zstd, uncompressed otherwise)

### Optional Segments

```toml
# fragment - optional input segments
[input]
    read1 = ['file_R1.fastq']
    read2 = ['file_R2.fastq']    # OPTIONAL: paired-end reads
    index1 = ['file_I1.fastq']   # OPTIONAL: index/barcode reads
    index2 = ['file_I2.fastq']   # OPTIONAL: dual-index reads
```

### Interleaved Mode

Alternative to separate read1/read2 files:

```toml
# fragment - interleaved input mode
[input]
    interleaved = ['read1', 'read2']
    read12 = ['interleaved.fastq']
```

### Input Options

```toml
# fragment - input format options
[input.options]
    fasta_fake_quality = 30           # TYPE: u8 (0-93), REQUIRED for FASTA input
    bam_include_mapped = true         # TYPE: bool, REQUIRED for BAM input
    bam_include_unmapped = true       # TYPE: bool, REQUIRED for BAM input
    read_comment_char = ' '           # TYPE: char, DEFAULT: ' '
```

## Processing Steps

Steps execute in order. Tags created by one step can be used by subsequent steps.

### Step Categories

1. **Extraction** - Create tags from sequences
2. **Numeric Tags** - Calculate metrics
3. **Boolean Tags** - Mark reads
4. **Filtering** - Remove reads
5. **Modification** - Edit sequences
6. **Tag Storage** - Save tags
7. **Barcode/Demux** - Barcode correction and splitting
8. **Validation** - Check data quality
9. **Reporting** - Generate reports

## Extraction Steps

Create "tags" that identify and label parts of sequences.

### ExtractRegion

Extract a fixed position region.

**USE WHEN**: You know exact base positions (e.g., UMI at bases 0-7)

```toml
[[step]]
    action = 'ExtractRegion'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    start = 0                      # TYPE: usize, REQUIRED, 0-indexed
    len = 8                        # TYPE: usize, REQUIRED
    out_label = 'umi'              # TYPE: string, REQUIRED, must be unique
```

### ExtractRegions

Extract multiple fixed position regions (concatenated).

**USE WHEN**: Barcode split across multiple positions

```toml
[[step]]
    action = 'ExtractRegions'
    regions = [                    # TYPE: array of objects, REQUIRED
        {segment = 'read1', start = 0, length = 8},
        {segment = 'read1', start = 12, length = 8}
    ]
    out_label = 'barcode'          # TYPE: string, REQUIRED
```

### ExtractIUPAC

Find IUPAC pattern with mismatches (substitutions only, no indels).

**USE WHEN**: Searching for adapter/motif with mismatches
**IUPAC CODES**: N=any, R=A/G, Y=C/T, S=G/C, W=A/T, K=G/T, M=A/C, B=C/G/T, D=A/G/T, H=A/C/T, V=A/C/G

```toml
[[step]]
    action = 'ExtractIUPAC'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    search = 'CTNNGG'              # TYPE: IUPAC string, REQUIRED
    max_mismatches = 1             # TYPE: usize, REQUIRED
    anchor = 'Anywhere'            # TYPE: 'Left'|'Right'|'Anywhere', REQUIRED
    out_label = 'motif'            # TYPE: string, REQUIRED
```

**anchor VALUES**:
- `'Left'`: Only match at start of read
- `'Right'`: Only match at end of read
- `'Anywhere'`: Match anywhere in read

### ExtractIUPACWithIndel

Find IUPAC pattern allowing insertions/deletions.

**USE WHEN**: Searching with indels (slower than ExtractIUPAC)

```toml
[[step]]
    action = 'ExtractIUPACWithIndel'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    search = 'CTNNGG'              # TYPE: IUPAC string, REQUIRED
    max_mismatches = 1             # TYPE: usize, REQUIRED
    max_indel_bases = 1            # TYPE: usize, REQUIRED
    anchor = 'Anywhere'            # TYPE: 'Left'|'Right'|'Anywhere', REQUIRED
    out_label = 'motif'            # TYPE: string, REQUIRED
    max_total_edits = 2            # TYPE: usize, OPTIONAL (default: mismatches + indels)
```

### ExtractIUPACSuffix

Trim adapter at end with partial matching.

**USE WHEN**: Trimming 3' adapters that may be partially present

```toml
[[step]]
    action = 'ExtractIUPACSuffix'
    segment = 'read1'              # TYPE: segment name, DEFAULT: 'read1'
    query = 'AGATCGGAAGAGC'        # TYPE: DNA string (AGTCN only), REQUIRED
    min_length = 3                 # TYPE: usize, REQUIRED, min bases to match
    max_mismatches = 1             # TYPE: usize, REQUIRED
    out_label = 'adapter'          # TYPE: string, REQUIRED
```

### ExtractRegex

Extract using regular expression.

**USE WHEN**: Complex pattern matching needed

```toml
[[step]]
    action = 'ExtractRegex'
    search = '^CT(..)CT'           # TYPE: regex string, REQUIRED
    replacement = '$1'             # TYPE: string, REQUIRED, use $1, $2 for groups
    source = 'read1'               # TYPE: segment name or 'name:<segment>', REQUIRED
    out_label = 'extracted'        # TYPE: string, REQUIRED
```

**source VALUES**:
- `'read1'`, `'read2'`, etc.: Extract from sequence
- `'name:read1'`: Extract from read name

### ExtractAnchor

Extract regions relative to a previously found tag.

**USE WHEN**: Extracting regions relative to a found motif
**REQUIRES**: Another tag created first as anchor

```toml
[[step]]
    action = 'ExtractIUPAC'
    search = 'CAYA'
    out_label = 'anchor_tag'
    segment = 'read1'
    anchor = 'Anywhere'
    max_mismatches = 0

[[step]]
    action = 'ExtractAnchor'
    in_label = 'anchor_tag'        # TYPE: existing tag name, REQUIRED
    regions = [[-2, 4], [4, 1]]    # TYPE: [[start, length], ...], REQUIRED
    region_separator = '_'         # TYPE: string, DEFAULT: '_'
    out_label = 'extracted'        # TYPE: string, REQUIRED
```

**regions FORMAT**: `[[start, length], ...]` where start is relative to anchor's leftmost position (can be negative)

### Quality-Based Extraction

#### ExtractLowQualityStart

Find low-quality bases at 5' end.

```toml
[[step]]
    action = 'ExtractLowQualityStart'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    min_qual = 'C'                 # TYPE: char (Phred+33), REQUIRED
    out_label = 'low_qual'         # TYPE: string, REQUIRED
```

#### ExtractLowQualityEnd

Find low-quality bases at 3' end.

```toml
[[step]]
    action = 'ExtractLowQualityEnd'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    min_qual = 'C'                 # TYPE: char (Phred+33), REQUIRED
    out_label = 'low_qual'         # TYPE: string, REQUIRED
```

#### ExtractRegionsOfLowQuality

Find all low-quality regions.

```toml
[[step]]
    action = 'ExtractRegionsOfLowQuality'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    min_quality = 60               # TYPE: u8, REQUIRED, ASCII value
    out_label = 'low_regions'      # TYPE: string, REQUIRED
```

**min_quality**: ASCII value (Phred+33). Example: 60 = '<' = Phred 27

### PolyX Extraction

#### ExtractPolyTail

Find homopolymer tail at read end.

**USE WHEN**: Removing polyA/T/G/C/N tails

```toml
[[step]]
    action = 'ExtractPolyTail'
    segment = 'read1'              # TYPE: segment name, DEFAULT: 'read1'
    base = 'A'                     # TYPE: 'A'|'T'|'G'|'C'|'N'|'.', REQUIRED
    min_length = 5                 # TYPE: usize, REQUIRED
    max_mismatch_rate = 0.1        # TYPE: float (0.0-1.0), REQUIRED
    max_consecutive_mismatches = 3 # TYPE: usize, REQUIRED
    out_label = 'polyA'            # TYPE: string, REQUIRED
```

**base VALUES**:
- `'A'`, `'T'`, `'G'`, `'C'`, `'N'`: Specific base
- `'.'`: Any repeated base

#### ExtractLongestPolyX

Find longest homopolymer anywhere in read.

```toml
[[step]]
    action = 'ExtractLongestPolyX'
    segment = 'read1'              # TYPE: segment name, DEFAULT: 'read1'
    base = '.'                     # TYPE: 'A'|'T'|'G'|'C'|'N'|'.', REQUIRED
    min_length = 5                 # TYPE: usize, REQUIRED
    max_mismatch_rate = 0.1        # TYPE: float (0.0-1.0), REQUIRED
    max_consecutive_mismatches = 3 # TYPE: usize, REQUIRED
    out_label = 'longest_poly'     # TYPE: string, REQUIRED
```

## Numeric Tag Steps

Create numeric tags for filtering or analysis.

### CalcLength

Calculate sequence length.

```toml
[[step]]
    action = 'CalcLength'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    out_label = 'length'           # TYPE: string, REQUIRED
```

### CalcGCContent

Calculate GC percentage (0.0-1.0).

```toml
[[step]]
    action = 'CalcGCContent'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    out_label = 'gc'               # TYPE: string, REQUIRED
```

### CalcBaseContent

Calculate custom base percentage.

```toml
[[step]]
    action = 'CalcBaseContent'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    bases_to_count = 'AT'          # TYPE: string, REQUIRED
    relative = true                # TYPE: bool, REQUIRED
    out_label = 'at_content'       # TYPE: string, REQUIRED
    bases_to_ignore = 'N'          # TYPE: string, OPTIONAL (only with relative=true)
```

**relative**:
- `true`: Returns percentage (0.0-1.0)
- `false`: Returns absolute count (bases_to_ignore not allowed)

### CalcNCount

Count N bases (wrapper around CalcBaseContent).

```toml
[[step]]
    action = 'CalcNCount'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    out_label = 'n_count'          # TYPE: string, REQUIRED
```

### CalcQualifiedBases

Count bases above/below quality threshold.

```toml
[[step]]
    action = 'CalcQualifiedBases'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    threshold = 'C'                # TYPE: char (Phred+33), REQUIRED
    op = 'below'                   # TYPE: string, REQUIRED
    out_label = 'hq_bases'         # TYPE: string, REQUIRED
```

**op VALUES**:
- `'below'`, `'<'`, `'lt'`, `'better'`: Count bases with quality < threshold (better quality)
- `'below_or_equal'`, `'<='`, `'lte'`, `'better_or_equal'`: Count bases with quality <= threshold
- `'above'`, `'>'`, `'gt'`, `'worse'`: Count bases with quality > threshold (worse quality)
- `'above_or_equal'`, `'>='`, `'gte'`, `'worse_or_equal'`: Count bases with quality >= threshold

### CalcExpectedError

Aggregate per-base error probabilities.

```toml
[[step]]
    action = 'CalcExpectedError'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    aggregate = 'sum'              # TYPE: 'sum'|'max', REQUIRED
    out_label = 'expected_error'   # TYPE: string, REQUIRED
```

### CalcComplexity

Calculate sequence complexity (transition ratio).

**USE WHEN**: Filtering low-complexity sequences

```toml
[[step]]
    action = 'CalcComplexity'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    out_label = 'complexity'       # TYPE: string, REQUIRED
```

### CalcKmers

Count matching k-mers from reference.

**USE WHEN**: Filtering contamination or specific sequences

```toml
[[step]]
    action = 'CalcKmers'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    files = ['reference.fa']       # TYPE: array of file paths, REQUIRED
    k = 21                         # TYPE: usize, REQUIRED
    count_reverse_complement = true # TYPE: bool, REQUIRED
    out_label = 'kmer_count'       # TYPE: string, REQUIRED
    min_count = 2                  # TYPE: usize, DEFAULT: 1
```

### ConvertRegionsToLength

Convert region tag to numeric length.

**USE WHEN**: You need numeric length from a region tag

```toml
[[step]]
    action = 'ConvertRegionsToLength'
    in_label = 'region_tag'        # TYPE: existing region tag, REQUIRED
    out_label = 'region_length'    # TYPE: string, REQUIRED
```

### EvalExpression

Calculate arithmetic expression combining tags.

**USE WHEN**: Combining multiple tags with math or logic

```toml
# First create the tags we'll use in the expression
[[step]]
    action = 'CalcGCContent'
    segment = 'read1'
    out_label = 'gc'

[[step]]
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'length'

# Now use them in the expression
[[step]]
    action = 'EvalExpression'
    expression = 'gc >= 0.4 && length > 50'  # TYPE: string, REQUIRED
    result_type = 'bool'           # TYPE: 'bool'|'numeric', REQUIRED
    out_label = 'pass'             # TYPE: string, REQUIRED
```

#### Expression Language

**Arithmetic Operators**: `+`, `-`, `*`, `/`, `%` (modulo), `^` (exponentiation)

**Comparison Operators**: `<`, `<=`, `>`, `>=`, `==`, `!=`

**Logical Operators**: `&&` (and), `||` (or), `!` (not)

**Functions**:
- `log(base, value)`: Logarithm
- `e()`: Euler's number
- `pi()`: Pi constant
- `int(x)`, `ceil(x)`, `floor(x)`, `round(x)`: Rounding
- `abs(x)`, `sign(x)`: Absolute value, sign
- `min(a, b, ...)`, `max(a, b, ...)`: Min/max
- `sin(x)`, `cos(x)`, `tan(x)`: Trigonometric (radians)
- `sinh(x)`, `cosh(x)`, `tanh(x)`: Hyperbolic

**Variables**:
- Any previously defined tag name (e.g., `gc`, `length`, `umi`)
- Location/string tags are converted to booleans (1 if present, 0 if absent)
- Numeric tags use their numeric value

**Virtual Tags** (automatically available):
- `len_<segment>`: Length of segment (e.g., `len_read1`, `len_read2`)
- `len_<tagname>`: Length of tag (e.g., `len_umi`, `len_barcode`)
  - For location tags: span of matched regions
  - For string tags (ExtractRegex with source=name:...): length of replaced string

**Examples**:

```toml
# First create some tags to use in expressions
[[step]]
    action = 'CalcGCContent'
    segment = 'read1'
    out_label = 'gc'

[[step]]
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'length'

[[step]]
    action = 'CalcQualifiedBases'
    segment = 'read1'
    threshold = 'I'                # Phred+33 char for Q40
    op = 'above'
    out_label = 'hq_bases'

[[step]]
    action = 'CalcBaseContent'
    segment = 'read1'
    bases_to_count = 'AT'
    relative = true
    out_label = 'at_content'

[[step]]
    action = 'ExtractRegion'
    segment = 'read1'
    start = 0
    length = 8
    out_label = 'umi'

# Boolean: Keep reads where GC is 40-60% AND length > 50bp
[[step]]
    action = 'EvalExpression'
    expression = 'gc >= 0.4 && gc <= 0.6 && length > 50'
    result_type = 'bool'
    out_label = 'good_read'

# Numeric: Calculate ratio
[[step]]
    action = 'EvalExpression'
    expression = 'hq_bases / length'
    result_type = 'numeric'
    out_label = 'hq_ratio'

# Using virtual tags (len_read1, len_umi are automatically available)
[[step]]
    action = 'EvalExpression'
    expression = 'len_read1 > 100 && len_umi == 8'
    result_type = 'bool'
    out_label = 'valid_structure'

# Complex math
[[step]]
    action = 'EvalExpression'
    expression = 'log(2, length) * gc + abs(at_content - 0.5)'
    result_type = 'numeric'
    out_label = 'score'
```

## Boolean Tag Steps

Mark reads with boolean tags.

### TagDuplicates

Mark duplicate reads (2nd and further duplicates).

**USE WHEN**: Identifying PCR duplicates

```toml
[[step]]
    action = 'TagDuplicates'
    source = 'read1'               # TYPE: string, REQUIRED
    false_positive_rate = 0.01     # TYPE: float (0.0-1.0), REQUIRED
    seed = 42                      # TYPE: u64, REQUIRED (if FPR > 0)
    out_label = 'is_duplicate'     # TYPE: string, REQUIRED
```

**source VALUES**:
- Segment name: `'read1'`, `'read2'`, etc.
- `'All'`: Concatenation of all segments
- `'tag:<name>'`: Use tag content
- `'name:<segment>'`: Use read name (requires split_character)

**false_positive_rate**:
- `0.0`: Exact hash (high memory)
- `> 0.0`: Cuckoo filter (approximate, lower memory)

### TagOtherFileByName

Mark reads whose names appear in another file.

```toml
[[step]]
    action = 'TagOtherFileByName'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    filename = 'names.fastq'       # TYPE: file path, REQUIRED
    false_positive_rate = 0.01     # TYPE: float (0.0-1.0), REQUIRED
    seed = 42                      # TYPE: u64, REQUIRED
    out_label = 'in_reference'     # TYPE: string, REQUIRED
    ignore_unaligned = false       # TYPE: bool, DEFAULT: false (for BAM/SAM)
    fastq_readname_end_char = ' '  # TYPE: char, OPTIONAL
    reference_readname_end_char = '/' # TYPE: char, OPTIONAL
```

### TagOtherFileBySequence

Mark reads whose sequences appear in another file.

```toml
[[step]]
    action = 'TagOtherFileBySequence'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    filename = 'sequences.fastq'   # TYPE: file path, REQUIRED
    false_positive_rate = 0.01     # TYPE: float (0.0-1.0), REQUIRED
    seed = 42                      # TYPE: u64, REQUIRED
    out_label = 'contaminant'      # TYPE: string, REQUIRED
    ignore_unaligned = false       # TYPE: bool, DEFAULT: false
```

## Filtering Steps

Remove or keep reads based on criteria. All filters have `keep_or_remove` parameter:
- `'Keep'`: Keep matching reads, remove non-matching
- `'Remove'`: Remove matching reads, keep non-matching

### FilterByTag

Filter by presence of location/string tag.

```toml
[[step]]
    action = 'FilterByTag'
    in_label = 'adapter'           # TYPE: existing tag, REQUIRED
    keep_or_remove = 'Remove'      # TYPE: 'Keep'|'Remove', REQUIRED
```

### FilterByNumericTag

Filter by numeric tag value range.

```toml
# First create a numeric tag
[[step]]
    action = 'CalcLength'
    segment = 'read1'
    out_label = 'length'

# Then filter by it
[[step]]
    action = 'FilterByNumericTag'
    in_label = 'length'            # TYPE: existing numeric tag, REQUIRED
    keep_or_remove = 'Keep'        # TYPE: 'Keep'|'Remove', REQUIRED
    min_value = 50.0               # TYPE: float, OPTIONAL
    max_value = 500.0              # TYPE: float, OPTIONAL (exclusive)
```

**CONSTRAINT**: At least one of `min_value` or `max_value` must be set

### FilterEmpty

Remove empty (zero-length) reads.

```toml
[[step]]
    action = 'FilterEmpty'
    segment = 'All'                # TYPE: segment name or 'All', REQUIRED
```

**segment='All'**: Only removes reads empty in ALL segments. Use multiple FilterEmpty steps for "any segment empty".

### Head

Keep only first N reads.

```toml
[[step]]
    action = 'Head'
    n = 1000                       # TYPE: usize, REQUIRED
```

### Skip

Skip first N reads.

```toml
[[step]]
    action = 'Skip'
    n = 100                        # TYPE: usize, REQUIRED
```

### FilterSample

Random sampling by probability.

```toml
[[step]]
    action = 'FilterSample'
    p = 0.1                        # TYPE: float (0.0-1.0), REQUIRED
    seed = 42                      # TYPE: u64, OPTIONAL
```

### FilterReservoirSample

Random sampling with exact count (reservoir sampling).

**USE WHEN**: Need exactly N random reads

```toml
[[step]]
    action = 'FilterReservoirSample'
    n = 10000                      # TYPE: usize, REQUIRED
    seed = 42                      # TYPE: u64, OPTIONAL
```

## Sequence Modification Steps

Edit sequences or quality scores.

### TrimAtTag

Trim read at tag position.

**USE AFTER**: ExtractIUPACSuffix, ExtractLowQualityEnd, etc.

```toml
[[step]]
    action = 'TrimAtTag'
    in_label = 'adapter'           # TYPE: existing tag, REQUIRED
    direction = 'End'              # TYPE: 'Start'|'End', REQUIRED
    keep_tag = false               # TYPE: bool, REQUIRED
```

**direction='End', keep_tag=false**: Trim everything from tag start to read end
**direction='End', keep_tag=true**: Trim everything after tag end
**direction='Start', keep_tag=false**: Trim everything from read start to tag end
**direction='Start', keep_tag=true**: Trim everything before tag start

### CutStart / CutEnd

Remove fixed number of bases.

```toml
[[step]]
    action = 'CutStart'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    n = 5                          # TYPE: usize, REQUIRED

[[step]]
    action = 'CutEnd'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    n = 10                         # TYPE: usize, REQUIRED
```

### Truncate

Limit maximum read length.

```toml
[[step]]
    action = 'Truncate'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    n = 150                        # TYPE: usize, REQUIRED
```

### Prefix / Postfix

Add bases to start/end of reads.

```toml
[[step]]
    action = 'Prefix'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    seq = 'AGTC'                   # TYPE: DNA string (agtcn), REQUIRED
    qual = 'IIII'                  # TYPE: quality string, REQUIRED (same length)

[[step]]
    action = 'Postfix'
    segment = 'read1'              # TYPE: segment name, REQUIRED
    seq = 'AGTC'                   # TYPE: DNA string (agtcn), REQUIRED
    qual = 'IIII'                  # TYPE: quality string, REQUIRED (same length)
```

### ReplaceTagWithLetter

Replace tagged regions with a specific letter.

**USE WHEN**: Masking low-quality regions as 'N'

```toml
[[step]]
    action = 'ReplaceTagWithLetter'
    in_label = 'low_qual'          # TYPE: existing tag, REQUIRED
    letter = 'N'                   # TYPE: char, DEFAULT: 'N'
```

### StoreTagInSequence

Replace sequence at tag position with tag content.

```toml
[[step]]
    action = 'StoreTagInSequence'
    in_label = 'corrected_barcode' # TYPE: existing tag, REQUIRED
    ignore_missing = true          # TYPE: bool, REQUIRED
```

### ReverseComplement

Reverse complement a segment.

```toml
[[step]]
    action = 'ReverseComplement'
    segment = 'read1'              # TYPE: segment name, REQUIRED
```

### MergeReads

Merge overlapping paired-end reads.

**USE WHEN**: Merging R1/R2 with overlap (e.g., short amplicons)

```toml
[[step]]
    action = 'MergeReads'
    segment1 = 'read1'             # TYPE: segment name, DEFAULT: 'read1'
    segment2 = 'read2'             # TYPE: segment name, DEFAULT: 'read2'
    reverse_complement_segment2 = true  # TYPE: bool, REQUIRED
    algorithm = 'FastpSeemsWeird'  # TYPE: string, REQUIRED
    min_overlap = 30               # TYPE: usize, REQUIRED
    max_mismatch_rate = 0.2        # TYPE: float (0.0-1.0), OPTIONAL
    max_mismatch_count = 5         # TYPE: usize, OPTIONAL
    no_overlap_strategy = 'as_is' # TYPE: 'as_is'|'concatenate', REQUIRED
    out_label = 'merged'           # TYPE: string, OPTIONAL (creates boolean tag)
    concatenate_spacer = 'NNNN'    # TYPE: string, REQUIRED if strategy='concatenate'
    spacer_quality_char = 33       # TYPE: u8, DEFAULT: 33
```

**CONSTRAINT**: At least one of `max_mismatch_rate` or `max_mismatch_count` required

### Swap 

Swap two segments.

```toml
[[step]]
    action = 'Swap'
    segment_a = 'read1'            # TYPE: segment name, REQUIRED
    segment_b = 'read2'            # TYPE: segment name, REQUIRED

```

### Case Conversion

**Sequence Case Conversion**:

```toml
[[step]]
    action = 'LowercaseSequence'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED

[[step]]
    action = 'UppercaseSequence'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
```

**Tag Case Conversion**:

```toml
[[step]]
    action = 'LowercaseTag'
    in_label = 'barcode'           # TYPE: existing tag, REQUIRED

[[step]]
    action = 'UppercaseTag'
    in_label = 'other_tag'         # TYPE: existing tag, REQUIRED
```

### Rename

Rename reads using regex.

```toml
[[step]]
    action = 'Rename'
    search = 'read_(.+)'           # TYPE: regex string, REQUIRED
    replacement = 'READ_$1'        # TYPE: string, REQUIRED
```

**replacement**: Use `$1`, `$2` for groups, `{{READ_INDEX}}` for unique counter

### ConvertQuality

Convert quality encoding.

```toml
[[step]]
    action = 'ConvertQuality'
    from = 'Illumina1.3'           # TYPE: string, REQUIRED
    to = 'Sanger'                  # TYPE: string, REQUIRED (must differ from 'from')
```

**VALID ENCODINGS**: `'Sanger'`, `'Illumina1.3'`, `'Solexa'`
**NOTE**: `'Illumina1.8'` is an alias for `'Sanger'`
**NOTE**: Automatically adds ValidateQuality step before conversion

## Tag Storage Steps

Save tags to output files or read names.

### StoreTagInComment

Store tag in read name comment.

**USE WHEN**: Preserving tags through alignment (e.g., UMIs for STAR)

```toml
[[step]]
    action = 'StoreTagInComment'
    segment = 'read1'              # TYPE: segment name or 'All', DEFAULT: 'read1'
    in_label = 'umi'               # TYPE: string, OPTIONAL (omit for all tags)
    comment_insert_char = ' '      # TYPE: char, DEFAULT: ' '
    comment_separator = '|'        # TYPE: char, DEFAULT: '|'
    region_separator = '_'         # TYPE: char, DEFAULT: '_'
```

**EXAMPLE**: `@read1 A00627:18:HGV7T` becomes `@read1|umi=ACGTACGT A00627:18:HGV7T`

### StoreTagLocationInComment

Store tag coordinates in comment (start-end, 0-based, half-open).

```toml
[[step]]
    action = 'StoreTagLocationInComment'
    in_label = 'adapter'           # TYPE: existing tag, REQUIRED
    segment = 'read1'              # TYPE: segment name or 'All', DEFAULT: 'read1'
    comment_insert_char = ' '      # TYPE: char, DEFAULT: ' '
    comment_separator = '|'        # TYPE: char, DEFAULT: '|'
```

### StoreTagInFastQ

Save tag content to separate FASTQ file.

```toml
[[step]]
    action = 'StoreTagInFastQ'
    in_label = 'umi'               # TYPE: existing tag, REQUIRED
    compression = 'Gzip'           # TYPE: 'Raw'|'Gzip'|'Zstd', REQUIRED
    compression_level = 6          # TYPE: usize, OPTIONAL (gzip: 0-9, zstd: 1-22)
    comment_tags = []              # TYPE: array of tag names, DEFAULT: []
    comment_location_tags = ['umi'] # TYPE: array, DEFAULT: [in_label]
    comment_insert_char = ' '      # TYPE: char, DEFAULT: ' '
    comment_separator = '|'        # TYPE: char, DEFAULT: '|'
    region_separator = '_'         # TYPE: char, DEFAULT: '_'
```

### StoreTagsInTable

Save all tags to TSV file.

```toml
[[step]]
    action = 'StoreTagsInTable'
    infix = 'tags'                 # TYPE: string, REQUIRED
    compression = 'Raw'            # TYPE: 'Raw'|'Gzip'|'Zstd', REQUIRED
    region_separator = '_'         # TYPE: char, DEFAULT: '_'
    in_labels = ['umi', 'barcode'] # TYPE: array, OPTIONAL (omit for all tags)
```

**OUTPUT**: `{prefix}_{infix}.tsv`

### QuantifyTag

Count tag occurrence frequencies.

```toml
[[step]]
    action = 'QuantifyTag'
    in_label = 'barcode'           # TYPE: existing tag, REQUIRED
    infix = 'barcode_counts'       # TYPE: string, REQUIRED
```

**OUTPUT**: `{prefix}_{infix}.qr.json`

## Barcode Correction & Demultiplexing

### HammingCorrect

Correct barcodes using Hamming distance.

**USE BEFORE**: Demultiplex or FilterByTag

```toml
[[step]]
    action = 'HammingCorrect'
    in_label = 'barcode'           # TYPE: existing tag, REQUIRED
    out_label = 'barcode_corrected' # TYPE: string, REQUIRED
    barcodes = 'my_barcodes'       # TYPE: string, REQUIRED
    max_hamming_distance = 1       # TYPE: usize, REQUIRED
    on_no_match = 'remove'         # TYPE: string, REQUIRED
```

**on_no_match VALUES**:
- `'remove'`: Remove tag (location and sequence)
- `'keep'`: Keep original tag
- `'empty'`: Keep location, set sequence to empty

### Demultiplex

Split output by barcode or boolean tag.

**CREATES**: Separate output files per barcode value

```toml
[[step]]
    action = 'Demultiplex'
    in_label = 'barcode_corrected' # TYPE: existing tag, REQUIRED
    barcodes = 'my_barcodes'       # TYPE: string, OPTIONAL (required for string tags)
    output_unmatched = true        # TYPE: bool, REQUIRED
```

**OUTPUT FILES**: `{prefix}_{sample_name}_{segment}.{suffix}`

### Barcode Definitions

Referenced by HammingCorrect and Demultiplex.

```toml
[barcodes.my_barcodes]
    'AAAAAAAA' = 'sample_1'        # barcode -> output_name
    'CCCCCCCC' = 'sample_2'
    'GGGGGGGG' = 'sample_1'        # Multiple barcodes can map to same sample
    'TTTTTTTT' = 'sample_3'
```

**DEMUX WITH BOOLEAN TAGS**: Omit `barcodes` parameter in Demultiplex step. Creates two outputs: `{prefix}_true_*` and `{prefix}_false_*`.

## Validation Steps

Check data quality and consistency.

### ValidateQuality

Check quality scores are in valid range for encoding.

```toml
[[step]]
    action = 'ValidateQuality'
    segment = 'All'                # TYPE: segment name or 'All', REQUIRED
    encoding = 'Illumina1.8'       # TYPE: string, REQUIRED
```

**VALID ENCODINGS**: `'Illumina1.8'`, `'Illumina1.3'`, `'Sanger'`, `'Solexa'`

### ValidateSeq

Check sequences contain only allowed bases.

```toml
[[step]]
    action = 'ValidateSeq'
    segment = 'read1'              # TYPE: segment name or 'All', REQUIRED
    allowed = 'agtcn'              # TYPE: string, REQUIRED (case-sensitive)
```

### ValidateName

Check read names match across segments.

```toml
[[step]]
    action = 'ValidateName'
    readname_end_char = '/'        # TYPE: char, OPTIONAL (omit for exact match)
```

**readname_end_char**: If set, only compares prefix before this character

### SpotCheckReadPairing

Sample-based read name validation (less strict than ValidateName).

```toml
[[step]]
    action = 'SpotCheckReadPairing'
    sample_stride = 1000           # TYPE: usize, DEFAULT: 1000
    readname_end_char = '/'        # TYPE: char, DEFAULT: '/'
```

**AUTO-INJECTED**: Automatically added when multiple segments present and `options.spot_check_read_pairing = true` (default)

### ValidateAllReadsSameLength

Check all reads have identical length.

```toml
[[step]]
    action = 'ValidateAllReadsSameLength'
    source = 'read1'               # TYPE: string, REQUIRED
```

**source VALUES**: segment name, `'All'`, `'tag:<name>'`, `'name:<segment>'`

## Reporting & Debugging Steps

### Report

Generate quality report at this point in pipeline.

**USE AT**: Beginning and/or end for before/after comparison

```toml
[[step]]
    action = 'Report'
    name = 'after_filtering'       # TYPE: string, REQUIRED
    count = true                   # TYPE: bool, DEFAULT: false
    base_statistics = true         # TYPE: bool, DEFAULT: false
    length_distribution = true     # TYPE: bool, DEFAULT: false
    duplicate_count_per_read = true # TYPE: bool, DEFAULT: false
    duplicate_count_per_fragment = true # TYPE: bool, DEFAULT: false
    count_oligos = ['AGTC', 'GGGG'] # TYPE: array, OPTIONAL
    count_oligos_segment = 'read1' # TYPE: string, REQUIRED if count_oligos set
```

### Progress

Report processing progress to stdout or file.

```toml
[[step]]
    action = 'Progress'
    n = 1000000                    # TYPE: usize, REQUIRED
    output_infix = 'progress'      # TYPE: string, OPTIONAL
```

**output_infix**: If set, writes to `{prefix}_{infix}.progress` instead of stdout
**CONSTRAINT**: Progress to stdout incompatible with `output.stdout = true`

### Inspect

Output sample reads for debugging.

```toml
[[step]]
    action = 'Inspect'
    n = 10                         # TYPE: usize, REQUIRED
    infix = 'inspection'           # TYPE: string, REQUIRED
    segment = 'read1'              # TYPE: segment name or 'all', REQUIRED
    suffix = 'txt'                 # TYPE: string, OPTIONAL
    compression = 'Gzip'           # TYPE: 'Raw'|'Gzip'|'Zstd', DEFAULT: 'Raw'
    compression_level = 6          # TYPE: usize, OPTIONAL
```

**OUTPUT**: `{prefix}_{infix}_{segment}.{suffix}` or `{prefix}_{infix}_interleaved.{suffix}` if segment='all'

## Tag Management Steps

### ForgetTag

Remove a tag from memory.

**USE WHEN**: Preventing tag from appearing in StoreTagsInTable or consuming memory

```toml
[[step]]
    action = 'ForgetTag'
    in_label = 'temporary_tag'     # TYPE: existing tag, REQUIRED
```

### ForgetAllTags

Remove all tags from memory.

```toml
[[step]]
    action = 'ForgetAllTags'
```

## Output Section

### Required Fields

```toml
# fragment - minimum required output configuration
[output]
    prefix = 'output'              # TYPE: string, REQUIRED
```

### Common Options

```toml
# fragment - common output options
[output]
    prefix = 'output'              # TYPE: string, REQUIRED
    format = 'Fastq'               # TYPE: string, DEFAULT: 'Fastq'
    compression = 'Gzip'           # TYPE: string, DEFAULT: 'Raw'
    suffix = '.fq.gz'              # TYPE: string, OPTIONAL (auto-determined)
    compression_level = 6          # TYPE: usize, OPTIONAL
```

**format VALUES**: `'Fastq'`, `'Fasta'`, `'BAM'`, `'None'`
**compression VALUES**: `'Raw'`, `'Gzip'`, `'Zstd'`
**compression_level**: gzip: 0-9 (default 6), zstd: 1-22 (default 5)

### Report Generation

```toml
# fragment - report generation options
[output]
    report_json = true             # TYPE: bool, DEFAULT: false
    report_html = true             # TYPE: bool, DEFAULT: false
```

**OUTPUT FILES**: `{prefix}.json`, `{prefix}.html`

### Advanced Options

```toml
# fragment - advanced output options
[output]
    stdout = false                 # TYPE: bool, DEFAULT: false
    interleave = false             # TYPE: bool, DEFAULT: false
    keep_index = false             # TYPE: bool, DEFAULT: false
    output = ['read1', 'read2']    # TYPE: array, OPTIONAL
    ix_separator = '_'             # TYPE: string, DEFAULT: '_'
    Chunksize = 1000000            # TYPE: usize, OPTIONAL
    output_hash_uncompressed = false # TYPE: bool, DEFAULT: false
    output_hash_compressed = false # TYPE: bool, DEFAULT: false
```

**stdout**: Write read1 to stdout (sets format='Raw', interleave=true if read2 exists)
**interleave**: Write R1/R2 interleaved in single file (`{prefix}_interleaved.{suffix}`)
**keep_index**: Also write index1/index2 files
**output**: Which segments to write (defaults to all)
**Chunksize**: Split output into chunks with index suffix

## Options Section

Global processing options (all optional).

```toml
# fragment - global processing options
[options]
    spot_check_read_pairing = true # TYPE: bool, DEFAULT: true
    thread_count = 2               # TYPE: usize, DEFAULT: 2 (or auto-detect)
    block_size = 10000             # TYPE: usize, DEFAULT: 10000
    buffer_size = 102400           # TYPE: usize, DEFAULT: 102400
    accept_duplicate_files = false # TYPE: bool, DEFAULT: false
```

## Decision Trees

### Task: Remove adapter sequences

**3' adapters (may be partial)**:
```
ExtractIUPACSuffix → TrimAtTag
```

**5' or internal adapters**:
```
ExtractIUPAC → TrimAtTag
```

### Task: Extract and preserve UMIs

```
ExtractRegion/ExtractRegions → StoreTagInComment → CutStart/CutEnd
```

### Task: Quality filtering

**By qualified bases**:
```
CalcQualifiedBases → FilterByNumericTag
```

**By quality trimming**:
```
ExtractLowQualityEnd → TrimAtTag
```

### Task: Length filtering

```
CalcLength → FilterByNumericTag
```

Or for zero-length:
```
FilterEmpty
```

### Task: Demultiplex samples

```
ExtractRegion/ExtractIUPAC → HammingCorrect → Demultiplex
```

### Task: Remove contamination

**By sequence matching**:
```
TagOtherFileBySequence → FilterByTag (keep_or_remove='Remove')
```

**By k-mer matching**:
```
CalcKmers → FilterByNumericTag
```

### Task: Deduplicate reads

```
TagDuplicates → FilterByTag (keep_or_remove='Remove')
```

### Task: Merge paired-end reads

```
MergeReads
```

### Task: Quality report only

```
Report
(Set output.format = 'None')
```

### Task: Complex filtering logic

```
Create multiple tags → EvalExpression → FilterByTag/FilterByNumericTag
```

## Validation Rules

1. **[input] section REQUIRED** with at least `read1` or `interleaved`
2. **[output] section REQUIRED** with at least `prefix`
3. **All file lists in [input] must have same length**
4. **Each [[step]] must have valid 'action' field**
5. **Tag names (out_label) must be unique** within pipeline
6. **in_label must reference previously created tag**
7. **segment names must match** those defined in [input]
8. **Barcode references must match** [barcodes.*] section names
9. **Steps execute in order** - tags must be created before use
10. **Type constraints must be satisfied** (see TYPE annotations)

## Common Errors to Avoid

❌ Using `in_label` before creating the tag with `out_label`
❌ Referencing non-existent segment names
❌ Using `FilterByNumericTag` on location tags (use `FilterByTag`)
❌ Using `FilterByTag` on numeric tags (use `FilterByNumericTag`)
❌ Forgetting to `TrimAtTag` after `ExtractIUPACSuffix`
❌ Using `output.stdout = true` with `Progress` (incompatible)
❌ Setting `format='None'` with compression (no sequence output)
❌ Mismatched file list lengths in [input]
❌ Using reserved names or special characters in tag names
❌ Applying segment-specific operations to non-existent segments
❌ Missing required parameters (check TYPE annotations)
❌ Wrong parameter types (string vs number vs bool vs array)
❌ Invalid enum values (check allowed values in TYPE comments)
❌ Not providing at least one of `max_mismatch_rate`/`max_mismatch_count` in `MergeReads`
❌ Not providing at least one of `min_value`/`max_value` in `FilterByNumericTag`

## File Naming Patterns

**Standard output**: `{prefix}_{segment}.{suffix}`
**Interleaved**: `{prefix}_interleaved.{suffix}`
**Demultiplexed**: `{prefix}_{sample_name}_{segment}.{suffix}`
**Chunked**: `{prefix}_{chunk_index}_{segment}.{suffix}`
**Reports**: `{prefix}.json`, `{prefix}.html`
**Inspect**: `{prefix}_{infix}_{segment}.{suffix}`
**StoreTagInFastQ**: `{prefix}_{in_label}_{segment}.{suffix}`
**StoreTagsInTable**: `{prefix}_{infix}.tsv`
**QuantifyTag**: `{prefix}_{infix}.qr.json`

## Quality Score Encodings

| Encoding | ASCII Range | Phred Range | Notes |
|----------|-------------|-------------|-------|
| Sanger / Illumina 1.8+ | 33-126 | 0-93 | Most common, Phred+33 |
| Illumina 1.3-1.7 | 64-126 | 0-62 | Phred+64 |
| Solexa | 59-126 | -5 to 62 | Rare, different formula |

**Common quality characters** (Phred+33):
- `!` = Phred 0 (p=1.0 error)
- `#` = Phred 2 (p=0.63)
- `+` = Phred 10 (p=0.1)
- `5` = Phred 20 (p=0.01)
- `?` = Phred 30 (p=0.001)
- `I` = Phred 40 (p=0.0001)

## Best Practices

1. **Start with a Report step** to understand input data
2. **Add Report steps** before and after major transformations
3. **Use Head during development** to process subset quickly
4. **Use Inspect** to verify transformations work correctly
5. **Validate quality encoding** if converting or reading BAM/FASTA
6. **Store UMIs in comments** to preserve through alignment
7. **Use EvalExpression** for complex multi-condition filtering
8. **Use virtual tags** (`len_*`) to avoid redundant CalcLength steps
9. **Forget unused tags** if processing many tags to save memory
10. **Test with cookbooks** before creating custom pipelines
