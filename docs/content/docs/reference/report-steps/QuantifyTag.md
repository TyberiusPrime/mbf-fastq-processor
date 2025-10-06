---
weight: 50
---

# QuantifyTag

Count the occurrences of each tag-sequence.

```toml
[[step]]
    action = "QuantifyTag"
    label = "mytag"
    infix = "tagcount" # output file is output_prefix_infix.tag.qr.json
```

This transformation counts how many times each unique tag value appears and outputs the results to a JSON file.