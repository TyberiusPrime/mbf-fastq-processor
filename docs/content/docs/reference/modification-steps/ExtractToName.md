---
not-a-transformation: true
---


```toml
[[step]]
    action = "ExtractRegions"
    regions = [
        {segment= "Read1", start = 0, length = 8},
        {segment= "Read1", start = 12, length = 4},
    ]
    out_label = "umi"
    region_separator = "_" # (optional) str, what to put between the regions, defaults to '_'

[[step]]
    action = "StoreTagInComment" 
    in_label = "umi"
```

Extract a sequence from the read and place it in the read name's comment section,
so a (space separated) 'key=value' pair is added to the read name.

Supports multiple region-extraction.

See [the tag section]({{< relref "docs/reference/tag-steps/_index.md" >}}) for more tag generation options.

(there used to be an [ExtractToName]({{< relref "docs/reference/modification-steps/ExtractToName.md" >}}) step before we introduced tag based analysis,
hence this piece of how-to documentation in the reference section)
