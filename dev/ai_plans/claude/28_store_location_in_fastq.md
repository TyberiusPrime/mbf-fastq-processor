# Implementation Plan: Store Location in FastQ

## Overview
Extend `StoreTagInFastQ` to store tag locations in read names using the existing `store_tag_in_comment` logic. This will allow preserving location information when sequences are written to FastQ files.

## Current State Analysis

### Existing StoreTagInFastQ Structure
- Located: `src/transformations/tag/store_tag_in_fastq.rs`
- Already has `comment_tags` functionality for adding tag values to read names
- Uses existing `store_tag_in_comment` function from `src/transformations/tag.rs:84`
- Current comment parameters:
  - `comment_tags: Vec<String>` - tags to add to read names
  - `comment_separator: u8` - separator character (default from `default_comment_separator`)
  - `comment_insert_char: u8` - where to insert comments (default from `default_comment_insert_char`)
  - `region_separator: BString` - separator for multiple regions

### Existing Location Logic
- `StoreTaglocationInComment` already exists with similar functionality
- Location format: `{tag}_location=target:start-end,target:start-end`
- Uses segment names from `input_info.segment_order`

## Implementation Plan

### 1. Add New Parameter to StoreTagInFastQ
**File**: `src/transformations/tag/store_tag_in_fastq.rs`

Add new field to the struct:
```rust
#[serde(default)]
comment_location_tags: Option<Vec<String>>,
```

### 2. Update validate_others Method
**Location**: `src/transformations/tag/store_tag_in_fastq.rs:85`

Add validation logic after existing comment_tags validation (around line 131):
```rust
// Handle default for comment_location_tags
if self.comment_location_tags.is_none() {
    // Default to using the main label if not specified
    self.comment_location_tags = Some(vec![self.label.clone()]);
}

// Validate comment_location_tags exist
if let Some(location_tags) = &self.comment_location_tags {
    for location_tag in location_tags {
        crate::transformations::filters::validate_tag_set(
            all_transforms,
            this_transforms_index,
            location_tag,
        )?;
    }
}
```

### 3. Update uses_tags Method
**Location**: `src/transformations/tag/store_tag_in_fastq.rs:153`

Extend to include location tags:
```rust
fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
    let mut tags = vec![self.label.clone()];
    tags.extend(self.comment_tags.clone());
    
    // Add location tags (deduplicated)
    if let Some(location_tags) = &self.comment_location_tags {
        for tag in location_tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }
    
    Some(tags)
}
```

### 4. Update Debug Implementation
**Location**: `src/transformations/tag/store_tag_in_fastq.rs:67`

Add new field to debug output:
```rust
.field("comment_location_tags", &self.comment_location_tags)
```

### 5. Modify apply Method Location Processing
**Location**: `src/transformations/tag/store_tag_in_fastq.rs:225-259`

After existing comment_tags loop, add location processing:
```rust
// Process location tags
if let Some(location_tags) = &self.comment_location_tags {
    for location_tag in location_tags {
        if let Some(tag_value) = tags.get(location_tag).unwrap().get(ii) {
            if let Some(hits) = tag_value.as_sequence() {
                let mut location_seq: Vec<u8> = Vec::new();
                let mut first = true;
                for hit in &hits.0 {
                    if let Some(location) = hit.location.as_ref() {
                        if !first {
                            location_seq.push(b',');
                        }
                        first = false;
                        location_seq.extend_from_slice(
                            format!(
                                "{}:{}-{}",
                                input_info.segment_order[location.segment_index.get_index()],
                                location.start,
                                location.start + location.len
                            )
                            .as_bytes(),
                        );
                    }
                }
                
                if !location_seq.is_empty() {
                    let location_label = format!("{}_location", location_tag);
                    let new_name = store_tag_in_comment(
                        &name,
                        location_label.as_bytes(),
                        &location_seq,
                        self.comment_separator,
                        self.comment_insert_char,
                    );
                    match new_name {
                        Err(err) => {
                            error_encountered = Some(format!("{err}"));
                            break 'outer;
                        }
                        Ok(new_name) => {
                            name = new_name;
                        }
                    }
                }
            }
        }
    }
}
```

### 6. Update Function Signature
**Location**: `src/transformations/tag/store_tag_in_fastq.rs:188`

The `apply` method needs access to `input_info` for segment names:
- Already has `_input_info` parameter - remove the underscore prefix
- Use `input_info.segment_order` for segment names

## Implementation Steps

1. **Add comment_location_tags field** to StoreTagInFastQ struct
2. **Update validate_others** to handle default and validation
3. **Update uses_tags** to include location tags with deduplication
4. **Update Debug implementation** to include new field
5. **Modify apply method** to process location tags and add them to read names
6. **Update input_info parameter** usage in apply method

## Testing Strategy

1. **Create test case** with StoreTagInFastQ using comment_location_tags
2. **Verify location format** matches existing StoreTaglocationInComment output
3. **Test default behavior** (should use main label when comment_location_tags not specified)
4. **Test deduplication** in uses_tags when same tag appears in multiple lists
5. **Test validation** of non-existent location tags

## Backward Compatibility

- New field is optional with sensible default
- Existing configurations will work unchanged
- Default behavior preserves existing functionality while adding location information

## Files to Modify

1. `src/transformations/tag/store_tag_in_fastq.rs` - Main implementation
2. Potential test files in `test_cases/` directory (to be determined based on existing test structure)

## Expected Behavior

When `comment_location_tags` is specified, read names will include location information in the format:
```
@original_read_name comment_separator {tag}_location=segment:start-end,segment:start-end
```

When not specified, defaults to using the main label for location information, providing automatic location tracking for the primary tag being stored.