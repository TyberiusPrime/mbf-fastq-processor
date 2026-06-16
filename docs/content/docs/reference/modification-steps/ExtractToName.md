---
not-a-transformation: true
---

# ExtractToName no longer exists

```toml
[[step]]
    action = "ExtractRegions"
    source= "read1"
    regions = [
        { start = 0, length = 8, anchor='left'}, # first 8 bytes
        { start = -4, length = 4, anchor='right'}, # last 4 bytes
    ]
    out_label = "umi"

[[step]]
    action = "StoreTagInComment" 
    region_separator = "_" # (optional) str, what to put between the regions, defaults to '_'
    in_label = "umi"
```

Extract a sequence from the read and place it in the read name's comment section,
so a (space separated) 'key=value' pair is added to the read name.

Supports multiple region-extraction.

See [the tag section]({{< relref "docs/reference/tag-steps/_index.md" >}}) for
more tag generation options.

(there used to be an [ExtractToName]({{< relref
"docs/reference/modification-steps/ExtractToName.md" >}}) step before we
introduced tag based analysis, hence this piece of how-to documentation in the
reference section)
