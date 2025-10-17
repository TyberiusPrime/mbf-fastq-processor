# outcome: success
# Tag Extension Plan: Adding Numeric and Generic Tag Types

## Current State Analysis

The current tag system in mbf_fastq_processor uses a single tag type:
`HashMap<String, Vec<Option<Hits>>>` where:
- `Hits` contains location-based sequence data (`Vec<Hit>`)  
- Each `Hit` has an optional `HitRegion` (location data) and a `BString`
(sequence data)- All tags are essentially `<locations,sequence>` pairs

This design is functional but prevents combining tag extraction and filtering

For example, `FilterMinLen` directly examines read lengths
rather than using any extracted (numeric) tag.

## Vision: Flexible Tag Architecture

The goal is to create a more versatile tag system supporting multiple data types:

1. **Sequence Tags** (current): Store location + sequence data
2. **Numeric Tags**: Store numeric values (lengths, quality scores, counts)

This enables decoupling extraction from filtering:
- `ExtractLen` creates a numeric tag with read length
- `FilterLen` uses the numeric tag to filter by length ranges
- Multiple filters can reuse the same extracted data

## Implementation Plan

### Phase 1: Core Tag Type System

#### 1.1 Define New Tag Value Types
**File**: `src/dna.rs` (extend existing types)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TagValue {
    #[default]
    Missing,                 // No value (replaces Option<TagValue>)
    Sequence(Hits),           // Current location+sequence tags
    Numeric(f64),            // Numeric values (lengths, scores, etc.)
}

impl TagValue {
    pub fn is_missing(&self) -> bool {
        matches!(self, TagValue::Missing)
    }
    
    pub fn as_numeric(&self) -> Option<f64> {
        match self { TagValue::Numeric(n) => Some(*n), _ => None }
    }
    
    pub fn as_sequence(&self) -> Option<&Hits> {
        match self { TagValue::Sequence(h) => Some(h), _ => None }
    }
}
```

#### 1.2 Update Block Structure
**File**: `src/io.rs` 

Change from:
```rust
pub tags: Option<HashMap<String, Vec<Option<Hits>>>>,
```

To:
```rust  
pub tags: Option<HashMap<String, Vec<TagValue>>>,
```

### Phase 2: New Tag Extractors

#### 2.1 Numeric Tag Extractors
**File**: `src/transformations/tag.rs`

##### ExtractLength (Enhanced)
Update existing `ExtractLength` to create numeric tags:

```rust
impl Step for ExtractLength {
    fn tag_provides_location(&self) -> bool {
        false  // Numeric tags don't need locations
    }
    
    fn apply(&mut self, mut block: FastQBlocksCombined, ...) -> (...) {
        extract_numeric_tags(
            self.target,
            &self.label, 
            |read| read.seq().len() as f64,
            &mut block,
        );
        (block, true)
    }
}
```

##### New Numeric Extractors
```rust
#[derive(eserde::Deserialize, Debug, Clone)]
pub struct ExtractMeanQuality {
    pub label: String,
    pub target: Target,
}

#[derive(eserde::Deserialize, Debug, Clone)]  
pub struct ExtractGCContent {
    pub label: String,
    pub target: Target,
}

#[derive(eserde::Deserialize, Debug, Clone)]
pub struct ExtractNCount {
    pub label: String, 
    pub target: Target,
}
```

#### 2.3 Helper Functions
```rust
fn extract_numeric_tags<F>(
    target: Target,
    label: &str, 
    extractor: F,
    block: &mut FastQBlocksCombined,
) where F: Fn(&WrappedFastQRead) -> f64 {
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }
    
    let mut values = Vec::new();
    match target {
        Target::Read1 => block.read1.apply(|read| {
            values.push(TagValue::Numeric(extractor(read)));
        }),
        // ... other targets
    }
    
    block.tags.as_mut().unwrap().insert(label.to_string(), values);
}

```

### Phase 3: Generic Filters

#### 3.1 Numeric Filters  
**File**: `src/transformations/filters.rs`

```rust
#[derive(eserde::Deserialize, Debug, Clone)]
pub struct FilterByNumericTag {
    pub label: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>, 
    pub keep_or_remove: KeepOrRemove,
}

impl Step for FilterByNumericTag {
    fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
        Some(vec![self.label.clone()])
    }
    
    fn apply(&mut self, mut block: FastQBlocksCombined, ...) -> (...) {
        let tag_values = block.tags.as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Numeric tag not found");
            
        let keep: Vec<bool> = tag_values.iter()
            .map(|tag_val| {
                if let Some(value) = tag_val.as_numeric() {
                    let passes_min = self.min_value.map_or(true, |min| value >= min);
                    let passes_max = self.max_value.map_or(true, |max| value <= max);
                    passes_min && passes_max
                } else {
                    false // Non-numeric values are filtered out
                }
            })
            .map(|passes| if self.keep_or_remove == KeepOrRemove::Remove { 
                !passes 
            } else { 
                passes 
            })
            .collect();
            
        apply_bool_filter(&mut block, &keep);
        (block, true)
    }
}
```


### Phase 4: Update Transformation Registry

#### 4.1 Add New Transformations
**File**: `src/transformations.rs`

```rust
pub enum Transformation {
    // Existing...
    
    // New numeric extractors
    ExtractMeanQuality(tag::ExtractMeanQuality),
    ExtractGCContent(tag::ExtractGCContent), 
    ExtractNCount(tag::ExtractNCount),
    
    // New boolean extractors
    ExtractHasAdapter(tag::ExtractHasAdapter),
    ExtractIsEmpty(tag::ExtractIsEmpty),
    
    // New generic filters  
    FilterByNumericTag(filters::FilterByNumericTag),
    FilterByBooleanTag(filters::FilterByBooleanTag),
}
```

#### 4.2 Update Step Implementations
All new steps need proper `Step` trait implementations with validation, tag usage tracking, etc.

### Phase 5: Migration and Compatibility

#### 5.1 Gradual Migration Strategy
2. **Phase 5b**: Update internal systems to use new TagValue enum  
3. **Phase 5c**: Migrate existing steps one by one
4. **Phase 5d**: Eventually deprecate legacy tag accessors

#### 5.2 Update Existing Tag Consumers
Files needing updates:
- `src/transformations/tag.rs` - All existing tag operations
- `src/transformations/filters.rs` - Tag-based filters  
- `src/io.rs` - Tag storage and access methods

Example migration for `FilterByTag`:
```rust
// Old version used Option<Hits>
let keep: Vec<bool> = block.tags.as_ref()
    .and_then(|tags| tags.get(&self.label))
    .expect("Tag not set")
    .iter()
    .map(Option::is_some)  // Old: Option<Hits>
    .collect();

// New version uses TagValue
let keep: Vec<bool> = block.tags.as_ref()
    .and_then(|tags| tags.get(&self.label))
    .expect("Tag not set") 
    .iter()
    .map(|tag_val| !tag_val.is_missing())  // New: TagValue
    .collect();
```

#### 5.3
 Extractors must declare what tag type they provide (enum).
 Then validation for filters can check that it matches the expected type.

### Phase 6: Testing and Documentation

#### 6.1 Test Coverage
Create comprehensive tests in `test_cases/` for:
- Each new tag extractor type
- Each new filter type  
- Mixed tag type scenarios
- Legacy compatibility
- Error handling for type mismatches

#### 6.2 Integration Tests
**File**: `tests/integration_tests.rs`

Add test cases demonstrating the new workflow:
```toml
[[step]]
transformation = "ExtractLength" 
target = "Read1"
label = "read1_len"

[[step]] 
transformation = "FilterByNumericTag"
label = "read1_len"
min_value = 50
max_value = 300
keep_or_remove = "Keep"
```

#### 6.3 Configuration Examples
Update documentation with examples showing:
- Length-based filtering using numeric tags
- Quality-based filtering using numeric tags  
- Complex multi-tag filtering scenarios
- Performance benefits of tag reuse

### Phase 7: Performance Considerations

#### 7.1 Memory Optimization
- `TagValue` enum should be memory-efficient 


### Phase 8: Configuration Schema Updates


## Benefits of This Design

1. **Decoupling**: Extraction logic separated from filtering logic
2. **Reusability**: One extraction can feed multiple filters
3. **Performance**: Computed values cached in tags, not recalculated  
4. **Flexibility**: Easy to add new tag types and operations
5. **Composability**: Complex filtering pipelines using multiple tag types
6. **Backward Compatibility**: Existing configurations continue to work