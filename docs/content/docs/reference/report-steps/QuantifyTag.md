---
weight: 50
---

# QuantifyTag

Count the occurrences of each tag-sequence.

```toml
[[step]]
    action = "QuantifyTag"
    label = "mytag"
    infix = "tagcount" # output file is output{ix_separator}tagcount.qr.json (default '_' → output_tagcount.qr.json)
```

This transformation counts how many times each unique tag value appears and outputs the results to a JSON file.
