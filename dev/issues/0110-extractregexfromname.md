status: closed
# ExtractRegexFromName

Or have ExtractRegex also take 'name'?

## Implementation

Implemented in commit 50d426e. ExtractRegex now supports reading from read names using the syntax:

```toml
[[step]]
    action = 'ExtractRegex'
    label = 'barcode'
    segment = 'name:read1'  # Extract from read name instead of sequence
    search = ':([A-Z]+):'
    replacement = '$1'
```

Key features:
- New segment types: `SegmentSequenceOrName` and `SegmentOrNameIndex`
- Syntax: `segment = "name:<segment>"` extracts from read name
- Syntax: `segment = "<segment>"` extracts from sequence (existing behavior)
- Tag labels cannot start with `"name:"` (reserved prefix)
- Full segment validation and error messages

Test coverage:
- Integration tests for single and multi-segment extraction
- Validation tests for invalid segments, reserved labels, and missing segment specification

Note: Test expected outputs need to be generated once network access to crates.io is restored. Run:
```bash
dev/update_tests.py
cargo test
```

