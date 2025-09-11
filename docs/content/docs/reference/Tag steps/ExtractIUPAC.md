---
weight: 50
---

# ExtractIUPAC


```toml
[[step]]
    action = "ExtractIUPAC"
    label = "mytag"
    anchor = 'Left' # Left | Right | Anywhere
    search = "CTN" # what we are searching
    segment = 'read1' # Any of your input segments


```

Search and extract a sequence from the read, defined by a IUPAC string.

See [the upper section](..) for uses of the tag.
