# ExtractToName


```toml
[[step]]
    action = "ExtractRegion"
    regions = [
        {source= "Read1", start = 0, length = 8},
        {source= "Read1", start = 12, length = 4},
    ]
    label = "umi"
    separator: "_" #(optional) str, what to put between the regions, defaults to '_'

[[step]]
    action = "TagToName" 
    label = "umi"
```

Extract a sequence from the read and place it in the read name's comment section,
so a (space separated) 'key=value' pair is added to the read name.

Supports multiple region-extraction.

See [the tag section](../../tag-steps) for more tag generation options.


(there used to be an ExtractToName step before we introduced tag based analysis)

