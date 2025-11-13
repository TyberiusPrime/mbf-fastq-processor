# ReverseComplementConditional

```toml
[[step]]
    action = "ReverseComplementConditional"
    in_label = "should_rc"  # Boolean tag that determines whether to reverse complement
    segment = "read1"       # Any of your input segments (default: read1)
```

Conditionally reverse-complements the read sequence (and reverses the quality) based on a boolean tag value.

For each read, if the boolean tag is `true`, the read is reverse-complemented.
If the tag is `false`, the read is left unchanged.

This is useful for conditionally orienting reads based on strand information,
adapter detection, or other criteria determined by prior steps.

This supports IUPAC codes (U is complemented to A, so it's not strictly
reversible). Unknown letters are output verbatim.

## Parameters

- `in_label`: (required) The name of a boolean tag that determines whether to reverse complement each read
- `segment`: (optional) Which segment to reverse complement (default: read1)

## Example

```toml
# Detect adapter sequence
[[step]]
    action = 'ExtractIUPAC'
    out_label = 'adapter'
    search = 'AGATCGGAAGAGC'
    max_mismatches = 1
    anchor = 'Left'

# Create boolean tag: reverse complement if adapter found at start
[[step]]
    action = 'EvalExpression'
    out_label = 'needs_rc'
    expression = 'adapter > 0'  # True if adapter locations exist
    result_type = 'bool'

# Conditionally reverse complement
[[step]]
    action = 'ReverseComplementConditional'
    in_label = 'needs_rc'
```
