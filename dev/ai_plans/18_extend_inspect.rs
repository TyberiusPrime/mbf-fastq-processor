## Extend Inspect to Write TSV Table of All Tags

### Current Analysis
The `Inspect` step currently:
- Collects first `n` reads from a specified target (read1, read2, index1, index2)
- Stores raw FastQ data (name, sequence, quality) in a collector vector
- Outputs collected reads as FastQ format during finalize()
- Located in: `src/transformations/reports/inspect.rs:11-128`

### Tags System Overview
- Tags are stored in `FastQBlocksCombined.tags: Option<HashMap<String, Vec<TagValue>>>`
- `TagValue` enum supports: `Missing`, `Sequence(Hits)`, `Numeric(f64)`, `Bool(bool)`
- Tags flow through pipeline and can be accessed by transformation steps
- Tags are indexed per-read within each block

### Implementation Plan

#### 1. **Modify Inspect struct** (`src/transformations/reports/inspect.rs:11-23`)
- Add new field: `write_tags_table: Option<bool>` with `#[serde(default)]`
- Default to `Some(true)` to match requirement "optional, but defaulting to true"
- Extend collector to store tags alongside read data:
  - Change `collector: Vec<NameSeqQualTuple>` to `collector: Vec<(NameSeqQualTuple, Option<HashMap<String, TagValue>>)>`

#### 2. **Update apply() method** (`src/transformations/reports/inspect.rs:58-85`)
- Access tags from `block.tags` when collecting reads
- For each collected read, capture its corresponding tag values:
  - Extract tag data for the specific read index from the HashMap<String, Vec<TagValue>>
  - Store both read data and tags in collector

#### 3. **Extend finalize() method** (`src/transformations/reports/inspect.rs:86-128`)
- Keep existing FastQ output generation
- Add conditional TSV table generation when `write_tags_table == Some(true)`:
  - Create additional file: `{output_prefix}_{infix}_{target}_tags.tsv`
  - TSV format: `read_name\ttag_name\ttag_type\ttag_value`
  - Handle different TagValue types:
    - `Missing`: output "missing" as value
    - `Numeric(f64)`: output the numeric value
    - `Bool(bool)`: output "true"/"false"
    - `Sequence(Hits)`: output joined sequence using `joined_sequence(None)`

#### 4. **Add test cases**
- Create test case with numeric tags: `test_cases/integration_tests/inspect_tags_numeric/`
- Create test case with sequence tags: `test_cases/integration_tests/inspect_tags_sequence/`
- Create test case with bool tags: `test_cases/integration_tests/inspect_tags_bool/`
- Each test should:
  - Set up pipeline with tag-generating step followed by Inspect
  - Verify both FastQ output and TSV table are generated
  - Validate TSV content matches expected tag values

#### 5. **Update documentation**
- Update `docs/content/docs/reference/Report steps/Inspect.md` 
- Document new `write_tags_table` parameter
- Provide example TSV output format
- Explain tag type handling

### File Changes Required
1. `src/transformations/reports/inspect.rs` - Main implementation
2. `test_cases/integration_tests/inspect_tags_*/` - New test cases
3. `docs/content/docs/reference/Report steps/Inspect.md` - Documentation
4. Run `dev/update_tests.py` after adding test cases

### Implementation Notes
- Maintain backward compatibility - existing Inspect configs continue working
- TSV output enabled by default but can be disabled with `write_tags_table = false`
- Handle edge cases: no tags present, empty tag values, special characters in tag names
- Use consistent file naming pattern matching existing outputs
- Apply same compression settings to TSV file as FastQ output