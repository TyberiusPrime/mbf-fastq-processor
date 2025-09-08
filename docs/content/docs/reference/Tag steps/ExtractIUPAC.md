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
    target = 'Read1' # Read1|Read2|Index1|Index2


```

Search and extract a sequence from the read, defined by a IUPAC string.

See [the upper section](..) for uses of the tag.

