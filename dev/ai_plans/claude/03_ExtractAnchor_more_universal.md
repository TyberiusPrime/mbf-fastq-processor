# Plan: ExtractAnchor More Universal

## High Level Goal
Rewrite the ExtractAnchor transformation to not perform its own sequence searching, but instead use a previously established tag and its leftmost position for anchor extraction. This makes the transformation more modular and reusable.

## Current State Analysis
- ExtractAnchor currently searches for anchor sequences within reads
- It performs its own pattern matching and position finding
- This duplicates search functionality that may already exist in other transformations

## Proposed Changes

### 1. Modify ExtractAnchor Interface
- Change ExtractAnchor to accept a tag name parameter instead of search patterns (input_label)
- Remove internal search logic
- Use the tag's position information to extract anchor sequences
- fail if the tag can't provide a valid position (e.g. ExtractLength)

### 2. Tag-Based Position System
- ExtractAnchor will look for a user defined tag that contains position information
- The tag should store the leftmost position of a previously found pattern
- Extract the anchor sequence starting from this tagged position

### 3. Configuration Updates
- Update TOML configuration schema to reflect new tag-based approach
- Remove search pattern parameters from ExtractAnchor config
- Add tag reference parameter

### 4. Pipeline Integration
- Ensure ExtractAnchor can be chained after search/tagging transformations
- Maintain compatibility with existing output formats
- Preserve anchor extraction functionality while using external position data

## Implementation Steps

### Phase 1: Analysis
1. Examine current ExtractAnchor implementation in `src/transformations.rs`
2. Identify existing tag system and how positions are stored
3. Review test cases that use ExtractAnchor

### Phase 2: Refactor ExtractAnchor
1. Modify ExtractAnchor struct to accept tag references
2. Remove internal search/pattern matching code
3. Implement tag-based position lookup
4. Update anchor extraction logic to use tagged positions

### Phase 3: Configuration Updates
1. Update TOML parsing for new ExtractAnchor parameters
2. Modify configuration validation
3. Update example configurations

### Phase 4: Testing
1. Update existing tests to use new tag-based approach
2. Add tests for tag integration
3. Verify pipeline chaining works correctly
4. Verify that it's using the left most coordinate by writing a test with ExtractRegions that has later regions before earlier ones

## Benefits
- Reduces code duplication by reusing search functionality
- Makes ExtractAnchor more modular and composable
- Improves pipeline flexibility
- Maintains separation of concerns between search and extraction

## Risks & Considerations
- Breaking change for existing configurations
- Need to ensure tag data is available when ExtractAnchor runs
- May need to handle cases where tag is missing or invalid
