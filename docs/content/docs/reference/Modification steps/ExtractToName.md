# ExtractToName


```toml
[[step]]
    action = "ExtractToName"
    regions = [
        {source= "Read1", start = 0, length = 8},
        {source= "Read1", start = 12, length = 4},
    ]

    separator: "_" # (optional) string, what to put between the read name and the umi, 
                   # defaults to '_'
    readname_end_chars: # (optional) Place (with sep) at the first of these characters.
                        # Defaults to " /" (which are where STAR strips the read name).
                        # If none are found, append it to the end.
    region_separator: "_" #(optional) str, what to put between the regions, defaults to '_'
```

Extract a sequence from the read and place it in the read name, for example for an UMI.

Supports multiple region-extraction.

