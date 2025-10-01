#outcome: failure
(this ai plan failed. 
It has been implemented manually though.)

# Implementation Plan: Arbitrary Segments Support

## Current Analysis

### Current Input Structure
- Fixed 4-segment structure: `read1`, `read2`, `index1`, `index2`
- Hard-coded in `config/mod.rs:Input` struct (lines 21-44)
- Used throughout via `FourReadsCombined<T>` struct (io.rs:788-793)
- `Target` enum hard-coded to 4 values (lines 160-170)
- Output config has separate boolean flags for each segment (lines 137-145)

### Key Dependencies
- **Core processing**: `FastQBlocksCombined` struct mirrors the 4-segment structure
- **Transformations**: All steps use `Target` enum to specify which segment(s) to operate on
- **I/O**: File reading/writing assumes 4 segments via `FourReadsCombined`
- **Validation**: Config validation checks segment count consistency across all 4 segments
- **Demultiplexing**: Uses fixed segment structure for routing

## Implementation Plan

### Phase 1: Extend Input Configuration (Backward Compatible)

#### Step 1.1: Add new `segments` field to Input struct
```rust
pub struct Input {
    #[serde(default)]
    pub interleaved: bool,

    // New field for arbitrary segments
    // have serde read all non-definied fields into this map
    #[serde(default)]
    pub segments: Option<HashMap<String, Vec<String>>>,

    #[serde(skip)]
    pub segment_order: Vec<String>, // Maintain consistent ordering, use to assign indices to segments. Based on the sorted keys of segments
    
}
```

#### Step 1.2: Add validation logic
- Validate that there's at least a read1 segment
- Validate that all segment vectors have equal length
- Check for duplicate filenames across all segments


### Phase 2: Generalize Core Data Structures

#### Step 2.1: Replace `FourReadsCombined<T>` with `SegmentsCombined<T>`
```rust
pub struct SegmentsCombined<T> {
    pub segments: Vec<T>,
}
```

#### Step 2.2: Update `FastQBlocksCombined`
- Replace individual segment fields with `Vec< FastQBlock>`


### Phase 3: Update Target System

#### Step 3.1: Extend Target enum
```rust
pub enum Target {
    // This is what we read from the tomls via serde 
    Named(String),
    Indexed(usize), // 0-based index into segments vector, present after validate_targets
}

pub enum TargetPlusAll {
    // This is what we read from the tomls via serde 
    Named(String),
    //also a valid serde option
    All,
    Indexed(usize), // 0-based index into segments vector, present after validate_targets
}
```

#### Step 3.2: Update Target parsing

### Phase 4: Update I/O Layer

#### Step 4.1: Modify file reading
- Update `InputFiles` to work with arbitrary segments
- Maintain ordering consistency
- Update parallel reading logic

#### Step 4.2: Update file writing
- Extend Output struct to support arbitrary segments
- Replace boolean flags with flexible segment selection

### Phase 5: Update Transformations

#### Step 5.1: Update transformation validation
- Ensure referenced segments exist
- Update error messages to work with arbitrary segment names

#### Step 5.2: Update transformation execution
- Replace hard-coded segment access with dynamic lookup
- Ensure all transformations work with new segment system

### Phase 6: Update Tests and Documentation

#### Step 6.1: Add test cases
- Test new configuration format
- Test backward compatibility
- Test error handling for invalid segment names

#### Step 6.2: Update documentation
- Add examples of new segment configuration
- Document migration path from legacy format

## Implementation Order

1. **Start with Step 1** - Extend Input struct with backward compatibility
2. **Implement adapter layer** - Allow rest of codebase to continue working
3. **Gradually migrate core structures** - One component at a time
4. **Update transformations** - Ensure all steps work with new system
5. **Add comprehensive tests** - Validate both new and legacy functionality
6. **Update documentation** - Guide users through new capabilities

## Risk Mitigation

- **Maintain backward compatibility** throughout implementation
- **Extensive testing** at each phase
- **Gradual migration** to avoid breaking existing functionality
- **Clear error messages** for configuration issues
- **Documentation** for migration path

## Success Criteria

- All existing TOML configurations continue to work unchanged
- New arbitrary segment configurations work correctly
- No performance regression
- All tests pass
- Code coverage maintained or improved
