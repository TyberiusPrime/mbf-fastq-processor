# the plan itself is bonkers and not how we're going to anything about this.

# Overlapping Region Tags Implementation Plan

## Problem Statement

Currently, when sequence edits (like `ReplaceTagWithLetter`, `StoreTagInSequence`) modify regions that overlap with existing tag locations, the system's location tracking can become inconsistent. The existing `filter_tag_locations` mechanism handles simple location adjustments, but complex scenarios involving overlapping or intersecting regions need careful handling.

## Current System Analysis

### Tag Location System
- Tags are stored as `TagValue::Sequence(Hits)` containing `Hit` structs
- Each `Hit` has an optional `HitRegion` with `{start, len, target}`
- `filter_tag_locations()` updates locations after sequence edits
- Current logic removes entire tags if any region becomes invalid ("any_none" behavior)

### Critical Edge Cases Identified
1. **Overlapping Replacements**: When replacing sequence in overlapping regions
2. **Length Changes**: When replacement text differs in length from original
3. **Partial Overlaps**: When only part of a tag region overlaps with edited region
4. **Multiple Overlaps**: When one edit affects multiple tags
5. **Cascading Updates**: When one tag replacement affects locations of other tags

## Implementation Plan

### Phase 1: Core Logic Implementation (2-3 hours)

#### 1.1 Enhanced NewLocation Enum
- **File**: `src/transformations.rs:765`
- **Action**: Extend `NewLocation` enum with new variants:
  ```rust
  pub enum NewLocation {
      Remove,
      Keep,
      New(HitRegion),
      NewWithSeq(HitRegion, BString),
      // NEW VARIANTS:
      Split(Vec<(HitRegion, BString)>),  // Split into multiple regions
      PartialReplace(HitRegion, BString, usize), // Partial replacement with offset
  }
  ```

#### 1.2 Overlap Detection Utility
- **File**: `src/transformations.rs` (new function)
- **Function**: `detect_region_overlap()`
  ```rust
  fn detect_region_overlap(
      tag_region: &HitRegion, 
      edit_start: usize, 
      edit_len: usize
  ) -> OverlapType {
      // Returns: NoOverlap, PartialStart, PartialEnd, Complete, Contains, Contained
  }
  ```

#### 1.3 Enhanced filter_tag_locations
- **File**: `src/transformations.rs:772`
- **Action**: Modify function to handle complex overlaps:
  ```rust
  fn filter_tag_locations_with_overlaps(
      block: &mut io::FastQBlocksCombined,
      target: Target,
      f: impl Fn(&HitRegion, usize, &BString, usize) -> Vec<NewLocation>,
  )
  ```

### Phase 2: Tag Replacement Logic (3-4 hours)

#### 2.1 Update ReplaceTagWithLetter
- **File**: `src/transformations/tag/replace_tag_with_letter.rs`
- **Action**: Implement overlap-aware replacement
- **Logic**: 
  - Before replacement, scan all tags for location conflicts
  - Calculate new positions after replacement
  - Update all affected tag locations consistently

#### 2.2 Update StoreTagInSequence  
- **File**: `src/transformations/tag/store_tag_in_sequence.rs`
- **Action**: Handle overlapping storage scenarios
- **Logic**:
  - Detect when storing would overlap existing tags
  - Decide resolution strategy (error, merge, split, or replace)
  - Update positions of subsequent tags

#### 2.3 New Overlap Resolution Strategies
- **File**: `src/transformations.rs` (new functions)
- **Strategies**:
  - **Conservative**: Error on any overlap
  - **Merge**: Combine overlapping regions
  - **Split**: Break tags at overlap boundaries  
  - **Replace**: Last operation wins

### Phase 3: Comprehensive Test Cases (2-3 hours)

#### 3.1 Basic Overlap Tests
- **Directory**: `test_cases/overlapping_tags/basic/`
- **Cases**:
  - Simple overlap: two regions with 1-2bp overlap
  - Complete overlap: one region entirely within another
  - Partial overlap: regions overlap at start/end
  - Adjacent regions: touching but not overlapping

#### 3.2 Length Change Tests  
- **Directory**: `test_cases/overlapping_tags/length_changes/`
- **Cases**:
  - Replacement shorter than original (gaps)
  - Replacement longer than original (shifts)
  - Zero-length replacement (deletion)
  - Multiple replacements affecting same region

#### 3.3 Multiple Tag Tests
- **Directory**: `test_cases/overlapping_tags/multi_tag/`
- **Cases**:
  - Three overlapping tags with different strategies
  - Chain of dependent tag locations
  - Cascading updates from single edit
  - Complex multi-step pipeline with overlaps

#### 3.4 Edge Case Tests
- **Directory**: `test_cases/overlapping_tags/edge_cases/`
- **Cases**:
  - Tags at sequence boundaries (start=0, end of read)
  - Empty tags (len=0) 
  - Tags extending beyond read length
  - Invalid regions after edits

### Phase 4: Documentation & Integration (1 hour)

#### 4.1 Configuration Options
- **File**: Add new config options for overlap handling
- **Options**:
  - `overlap_strategy: "error" | "merge" | "split" | "replace"`
  - `allow_tag_overlaps: bool`
  - `preserve_tag_order: bool`

#### 4.2 Error Messages
- **Files**: Update error handling throughout
- **Messages**: Clear descriptions of overlap conflicts and resolutions

#### 4.3 Documentation
- **Files**: Update existing step documentation  
- **Content**: Document overlap behavior for each transformation

## Test Scenarios Detail

### Scenario 1: Basic Overlap
```toml
[[step]]
action = 'ExtractRegions'  
regions = [{source = 'Read1', start = 5, length = 10}]
label = 'tag1'

[[step]]
action = 'ExtractRegions'
regions = [{source = 'Read1', start = 10, length = 8}]  # Overlaps positions 10-14
label = 'tag2'

[[step]]
action = 'ReplaceTagWithLetter'
label = 'tag1'  
letter = 'N'  # What happens to tag2's location?
```

### Scenario 2: Length Change
```toml
# Original: ATCGATCGATCG (12bp)
# tag1 at positions 2-6 (CGAT, 4bp)
# tag2 at positions 8-11 (ATCG, 4bp)

[[step]]
action = 'ReplaceTagWithLetter'
label = 'tag1'
letter = 'NN'  # Replace 4bp with 2bp -> sequence becomes ATNNGATCGATCG (10bp)
# tag2 should now be at positions 6-9, not 8-11
```

### Scenario 3: Complete Containment
```toml
# tag1: positions 5-15 (10bp)
# tag2: positions 7-12 (5bp, entirely within tag1)
# Replace tag1 with single letter -> what happens to tag2?
```

## Implementation Timeline

- **Day 1**: Phase 1 - Core overlap detection logic
- **Day 2**: Phase 2 - Tag replacement updates  
- **Day 3**: Phase 3 - Test case development
- **Day 4**: Phase 4 - Integration and documentation

## Success Criteria

1. All existing tests continue to pass
2. New overlap scenarios are handled predictably  
3. Clear error messages for unsupported overlap cases
4. Performance impact < 10% for typical pipelines
5. Comprehensive test coverage for edge cases

## Risk Mitigation

- **Breaking Changes**: Maintain backward compatibility with flag-controlled behavior
- **Performance**: Profile complex overlap scenarios, optimize if needed
- **Complexity**: Start with conservative "error on overlap" strategy, add flexibility later
- **Testing**: Extensive automated test coverage before merging
