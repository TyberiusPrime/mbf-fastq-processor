---
weight: 50
title: Extract IUPAC
---

# ExtractIUPAC


```toml
[[step]]
    action = "ExtractIUPAC"
    out_label = "mytag"
    anchor = 'Left' # Left | Right | Anywhere
    search = "CTN" # what we are searching. May also be a list ["CTN", "GAN", ...]
    segment = 'read1' # Any of your input segments
    max_mismatches = 0 # required. How many mismatches are allowed


```

Search and extract a sequence from the read, defined by a [IUPAC string](https://doi.org/10.1093%2Fnar%2F13.9.3021).

Anchor is the regex equivalent of ^ (Left), $ (Right) or no anchor (Anywhere).

If anchor = 'Anywhere', ExtractIUPAC will find the left most occurrence.

When multiple search queries are present they'll be searched in order. 
The first hit wins.

Ambiguous matches (e.g. query 'Y' matching 'C') do not count as even partial mismatches.


