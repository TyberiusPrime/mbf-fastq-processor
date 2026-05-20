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
    search = "CTN" # May also be a list ["CTN", "GAN", ...]
    segment = 'read1' # Any of your input segments
    max_mismatches = 0 # required. How many mismatches are allowed
    on_tie = 'Earliest' # Earliest|LeftMost|RightMost 
                        # - decide what happens when multiple searches hit


```

Search and extract a sequence from the read, defined by one ore more [IUPAC
string(s)](https://doi.org/10.1093%2Fnar%2F13.9.3021).

Anchor is the regex equivalent of ^ (Left), $ (Right) or no anchor (Anywhere).

If anchor = 'Anywhere', ExtractIUPAC will find the left most occurrence.

Ambiguous matches (e.g. query 'Y' matching 'C') do not count as mismatches, but
as full matches.


## Multiple search queries

With multiple search queries, multiple may be present in your read.

If `anchor` is 'Left' or 'Right', the first from the list of `search'
that hits within `max_mismatches` will be reported.

For `anchor` == 'Anywhere', this is also the default. This may be changed
by setting `on_tie` to 'LeftMost' or 'RightMost' which will instead
take the left or right most occurence. No preference to lower hamming distances
is given.




