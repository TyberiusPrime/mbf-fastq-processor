### FillMissing


```toml
[[step]]
    action = "FillMissing"
    in_label_primary = "mytag"
    in_label_secondary = "mytag2"
    out_label = "result"
```

Return the primary tag if it is not missing, otherwise return the secondary tag.
If both are missing the output is also missing.

Both input tags must be Location or String tags.

When the two tags have different kinds the output
is always a String tag and any Location value is converted to the bases it
covers.

`in_label_primary` accepts aliases `primary` and `first`.
`in_label_secondary` accepts aliases `secondary` and `second`.

#### Example: barcode with fallback

```toml
[[step]]
    action = "ExtractIUPAC"
    search = "AAAA"
    out_label = "barcode_primary"
    segment = "read1"
    anchor = "Left"
    max_mismatches = 0

[[step]]
    action = "ExtractIUPAC"
    search = "TTTT"
    out_label = "barcode_fallback"
    segment = "read1"
    anchor = "Right"
    max_mismatches = 0

[[step]]
    action = "FillMissing"
    in_label_primary = "barcode_primary"
    in_label_secondary = "barcode_fallback"
    out_label = "barcode"
```

For reads where `barcode_primary` was not found, `barcode` will contain
the value of `barcode_fallback` instead.
