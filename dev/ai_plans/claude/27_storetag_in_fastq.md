# this one failed utterly. Wrong implementation, overwrote *my* implementation 
# when I reminded it to write test casess...

# StoreTagInFastQ Implementation Plan

## Overview
Implement a `StoreTagInFastQStep` that writes tagged reads to separate FASTQ files based on a specified location tag. This step will create one FASTQ file per unique tag value.

## File Structure
- **Main implementation**: `src/transformations/tag/store_tag_in_fastq.rs`
- **Integration**: Add to `src/transformations.rs` enum

## Step-by-Step Implementation

### 1. Create the Core Structure
Based on `StoreTagInComment`, create a new struct with these fields:
```rust
#[derive(eserde::Deserialize, Debug, Clone)]
pub struct StoreTagInFastQ {
    label: String,                    // Tag name to use for filtering
    segment: SegmentOrAll,           // Which segments to apply to
    segment_index: Option<SegmentIndexOrAll>,  // Computed field
    
    // Optional read name comment fields (like StoreTagInComment)
    #[serde(default)]
    comment_tags: Vec<String>,       // Tags to include in read names
    #[serde(default = "default_comment_separator")]
    comment_separator: u8,           // '|' by default
    #[serde(default = "default_comment_insert_char")]
    comment_insert_char: u8,         // ' ' by default
    #[serde(default = "default_region_separator")]
    region_separator: BString,       // For joining sequences
}
```

### 2. File Naming Convention
Output files follow the pattern: `{output_prefix}.tag.{tag_name}.fastq.{suffix}`
- `output_prefix`: From config output section
- `tag_name`: The actual tag value (sanitized for filesystem)
- `suffix`: Includes compression extension if applicable

### 3. Core Implementation Components

#### A. Tag Value Collection & File Management
- Collect all unique tag values during processing
- Create output files eagerly and store in struct.
- Handle file naming conflicts and invalid filesystem characters

#### B. File Creation Logic
Based on analysis of `open_output_files` and `OutputFile::new_file`:
- Use `OutputFile::new_file()` for file creation
- Support compression levels and formats from config
- Include hash computation options if configured
- Handle output directory path construction

#### C. Write Logic
For each read with the target tag:
1. Extract tag value from read's tag collection  
2. Optionally add comment tags to read name (reuse StoreTagInComment logic)
3. Get or create output file for this tag value
4. Write the read to the appropriate file

### 4. Integration Points

#### A. Add to Transformation Enum
In `src/transformations.rs`:
```rust
//store
RemoveTag(tag::RemoveTag),
StoreTagInComment(tag::StoreTagInComment),
StoreTagInFastQ(tag::StoreTagInFastQ),  // <- Add this
StoreTagLocationInComment(tag::StoreTaglocationInComment),
```

#### B. Module Declaration
Add to the appropriate module file:
```rust
pub mod store_tag_in_fastq;
pub use store_tag_in_fastq::StoreTagInFastQ;
```

### 5. Step Trait Implementation

#### A. `validate_segments()`
- Validate segment configuration against input definition
- Set computed `segment_index` field

#### B. `validate_others()`
- Verify target tag exists in pipeline and is of TagValueType::Sequence
- Validate comment_tags if specified
- Check output configuration compatibility
- make sure there is only one StoreTagInFastQ with this tag.

#### C. `uses_tags()`
- Return vector containing the label tag and any comment_tags

#### D. `apply()`
- Process each read in the block
- Extract tag values and write to appropriate files
- Handle missing tags gracefully
- Apply comment tag logic if configured

### 6. Resource Management
- Implement proper file handle cleanup
- Buffer writes appropriately
- Handle compression efficiently
- Manage memory for large numbers of unique tag values

### 7. Error Handling
- Graceful handling of filesystem issues
- Clear error messages for invalid tag names
- Proper cleanup on failures
- Validation of output directory permissions

### 8. Testing Strategy
- Create test cases with multiple tag values
- Test file naming with various tag characters
- Verify compression works correctly
- Test comment tag integration

- Validate proper resource cleanup

## Key Dependencies
- Reuse existing utilities from `store_tag_in_comment.rs`
- Leverage `OutputFile` infrastructure from `src/lib.rs`
- Use tag processing utilities from the tag module
- Follow existing patterns for step validation and application

## Configuration Example
```toml
[[step]]
type = "StoreTagInFastQ"
label = "cell_barcode"
segment = "All"
comment_tags = ["umi", "gene"]
comment_separator = "|"
```

This will create files like:
- `output.tag.AAACCTGAGAAGGCCT.fastq.gz`  
- `output.tag.AAACCTGAGACAGAGA.fastq.gz`
- etc.

Each containing reads with that specific cell barcode, with UMI and gene tags added to read names.
