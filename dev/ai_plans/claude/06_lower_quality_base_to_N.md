# Quality-Based Base Replacement Implementation Plan

## Overview
Implement functionality to replace bases below a quality threshold with 'N' characters. This will be done using a two-step approach as suggested in the original plan:

1. **ExtractRegionsOfLowQuality**: Extract regions where bases have quality scores below threshold
2. **ReplaceTagWithLetter**: Replace the sequence in tagged regions with a specified letter

## Architecture Analysis

Based on the existing transformation system:
- Transformations use the `Step` trait and are dispatched via `enum_dispatch`
- Quality operations like `TrimQualityStart`/`TrimQualityEnd` already exist as templates
- Tag-based operations are in `src/transformations/tag.rs`
- Edit operations are in `src/transformations/edits.rs`

## Implementation Plan

### 1. ExtractRegionsOfLowQuality Transformation

**Location**: `src/transformations/tag.rs` (tag-extracting transformation)

**Configuration**:
```toml
[[step]]
action = "ExtractRegionsOfLowQuality"
target = "Read1"  # Target::Read1, Read2, Index1, Index2
min_quality = '<'  # u8 quality threshold (Phred+33)
label = "low_quality_regions"  # Tag name to store results
```

**Functionality**:
- Scan through quality scores of specified target
- Identify contiguous regions where quality < threshold  
- Store regions as tags with location information (start, length)
- Each region becomes a `HitRegion` with target, start position, and length

**Implementation Details**:
- Use existing `u8_from_char_or_number` deserializer for quality threshold
- Follow pattern of existing tag extractors like `ExtractRegex`
- Use `extract_tags` helper function to store results
- Return `Hits` containing `HitRegion` for each low-quality region

### 2. ReplaceTagWithLetter Transformation

**Location**: `src/transformations/tag.rs` (tag-consuming transformation) 

**Configuration**:
```toml
[[step]]
action = "ReplaceTagWithLetter"
tag = "low_quality_regions"  # Tag to consume
letter = "N"  # Replacement character (default 'N')
```

**Functionality**:
- Read tagged regions from previous step
- Replace sequence bases in those regions with specified letter
- Preserve quality scores (don't modify them)
- Clear/remove the consumed tag after processing

**Implementation Details**:
- Use existing tag consumption patterns
- Modify sequence in-place using `WrappedFastQReadMut`
- Default replacement letter to 'N' 
- Use `u8_from_char_or_number` for letter parameter
- Implement `uses_tags()` and `removes_tag()` methods

### 3. TOML Configuration Support

**Location**: `src/transformations.rs` 

Add new variants to `Transformation` enum:
```rust
ExtractRegionsOfLowQuality(tag::ExtractRegionsOfLowQuality),
ReplaceTagWithLetter(tag::ReplaceTagWithLetter),
```

### 4. Testing Strategy

**Integration Tests**:
- Create test case in `test_cases/integration_tests/`
- Input FASTQ with known quality patterns
- Expected output with 'N' replacements in low-quality positions
- Test different quality thresholds
- Test with paired-end reads
- Test edge cases (all high quality, all low quality)

**Unit Tests**:
- Test quality threshold boundary conditions
- Test region extraction accuracy
- Test replacement letter customization
- Test tag interaction (creation and consumption)

### 5. Documentation

**Reference Documentation**:
- Create `docs/content/docs/reference/Tag steps/ExtractRegionsOfLowQuality.md`
- Create `docs/content/docs/reference/Tag steps/ReplaceTagWithLetter.md`
- Include example TOML configurations
- Document quality score interpretation (Phred+33) ( link https://en.wikipedia.org/wiki/Phred_quality_score#Symbols )
- Explain both in templates.toml

### 6. Example Usage

```toml
[input]
read1 = "input.fastq.gz"

[[step]]
action = "ExtractRegionsOfLowQuality" 
target = "Read1"
min_quality = 27
label = "low_qual"

[[step]]
action = "ReplaceTagWithLetter"
tag = "low_qual"
letter = "N"

[output]
prefix = "processed"
```

## Implementation Order

1. ✅ **Analysis**: Understand existing transformation architecture
2. ✅ **Design**: Plan ExtractRegionsOfLowQuality transformation details
3. 🔄 **Design**: Plan ReplaceTagWithLetter transformation details  
4. **Implement**: ExtractRegionsOfLowQuality in `tag.rs`
5. **Implement**: ReplaceTagWithLetter in `tag.rs`
6. **Config**: Add enum variants to `transformations.rs`
7. **Test**: Create integration test cases
8. **Document**: Add reference documentation

## Technical Considerations

- **Quality Score Format**: Assume Phred+33 encoding (standard for modern FASTQ)
- **Performance**: Process regions efficiently without excessive memory allocation
- **Tag Lifecycle**: Ensure proper tag creation and cleanup
- **Validation**: Validate quality thresholds and target availability
- **Edge Cases**: Handle empty regions, boundary conditions
- **Reproducibility**: Ensure deterministic behavior across runs

This approach follows the existing codebase patterns while providing flexible, composable quality-based base replacement functionality.
