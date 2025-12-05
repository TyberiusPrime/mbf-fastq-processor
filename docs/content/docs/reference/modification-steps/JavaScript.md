---
weight: 60
---

# JavaScript

```toml
[[step]]
    action = "JavaScript"
    segment = "read1"
    code = '''
function process_reads(reads, tags, state) {
    // Convert sequences to uppercase
    for (let read of reads) {
        read.seq = read.seq.toUpperCase();
    }
    return {};
}
'''
```

Execute arbitrary JavaScript code on FASTQ reads using the [Boa](https://boajs.dev/) engine (a pure Rust JavaScript engine).

This step is useful for complex transformations that can't be expressed with existing steps,
or for prototyping new logic before implementing it as a native step.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `code` | string | Either `code` or `file` | Inline JavaScript code |
| `file` | string | Either `code` or `file` | Path to a JavaScript file |
| `segment` | string | No (default: `read1`) | Segment to operate on |
| `in_tags` | array | No | Tag names to pass to JavaScript |
| `out_tags` | object | No | Output tag declarations: `{ tag_name = "Type" }` |

### Output Tag Types

- `String` - String value (allows null for Missing)
- `Numeric` - Floating point number (null not allowed)
- `Bool` - Boolean value (null not allowed)
- `Location` - Array of hit objects with location info

## Function Signature

Your JavaScript code must define a `process_reads` function:

```javascript
function process_reads(reads, tags, state) {
    // Process reads
    return { tag1: [...], tag2: [...], _state: {...} };
}
```

### Parameters

- **reads**: Array of read objects, each with:
  - `seq` (string): Sequence bases
  - `qual` (string): Quality scores
  - `name` (string): Read name
  - `index` (number): Read index in block

- **tags**: Object mapping tag names to arrays of values (one per read).
  Location tags are passed as arrays of hit objects:
  ```javascript
  tags.umi[i] = [
    { start: 0, len: 8, segment: "read1", sequence: "ACGTACGT" }
  ]
  ```

- **state**: Object containing state from previous block (null on first block)

### Return Value

Return an object with:
- Tag names as keys, each mapping to an array of values (one per read)
- Optional `_state` key for state persistence across blocks

## Sequence Modification

You can modify reads directly by changing their properties:

```toml
[[step]]
    action = "JavaScript"
    segment = "read1"
    code = '''
function process_reads(reads, tags, state) {
    for (let read of reads) {
        // Convert to uppercase
        read.seq = read.seq.toUpperCase();
    }
    return {};
}
'''
```

When modifying sequences:
- If `seq` length changes, quality scores are truncated or extended (with 'I')
- You can also modify `qual` and `name`

## State Persistence

Use the `_state` key to persist data across blocks:

```toml
[[step]]
    action = "JavaScript"
    segment = "read1"
    code = '''
function process_reads(reads, tags, state) {
    let count = state ? state.count : 0;
    count += reads.length;
    return { _state: { count: count } };
}
'''
```

## Location Tags

To output Location tags, return arrays of hit objects:

```javascript
function process_reads(reads, tags, state) {
    let locations = [];
    for (let read of reads) {
        locations.push([
            { start: 0, len: 5, segment: "read1", sequence: read.seq.substring(0, 5) }
        ]);
    }
    return { my_region: locations };
}
```

## Examples

### Calculate Sequence Length

```toml # ignore_in_test
[[step]]
    action = "JavaScript"
    segment = "read1"
    code = '''
function process_reads(reads, tags, state) {
    let lengths = [];
    for (let read of reads) {
        lengths.push(read.seq.length);
    }
    return { seq_length: lengths };
}
'''
    out_tags = { seq_length = "Numeric" }
```

### ROT-Encode Bases

```toml
[[step]]
    action = "JavaScript"
    segment = "read1"
    code = '''
function process_reads(reads, tags, state) {
    let rot = {'A': 'C', 'C': 'G', 'G': 'T', 'T': 'A'};
    for (let read of reads) {
        read.seq = read.seq.split('').map(function(b) {
            return rot[b] || b;
        }).join('');
    }
    return {};
}
'''
```

### Use Input Tags

```toml # ignore_in_test
[[step]]
    action = "ExtractRegion"
    source = "read1"
    start = 0
    len = 8
    anchor = "Start"
    out_label = "umi"

[[step]]
    action = "JavaScript"
    segment = "read1"
    in_tags = ["umi"]
    code = '''
function process_reads(reads, tags, state) {
    let flags = [];
    for (let i = 0; i < reads.length; i++) {
        // Check if UMI contains 'N'
        let umi = tags.umi[i];
        let has_n = umi && umi[0] && umi[0].sequence.indexOf('N') >= 0;
        flags.push(has_n);
    }
    return { has_n_in_umi: flags };
}
'''
    out_tags = { has_n_in_umi = "Bool" }
```

### Load from File

Instead of inline `code`, you can use `file = "path/to/script.js"` to load JavaScript from an external file.
The file should define the `process_reads` function just like inline code.

## Notes

- The JavaScript step runs serially (not parallelized) due to JS engine constraints
- Each block is processed independently; use `_state` for cross-block data
- Performance: Native steps are faster; use JavaScript for flexibility, not speed
- The Boa engine implements ECMAScript, so modern JS features are available
