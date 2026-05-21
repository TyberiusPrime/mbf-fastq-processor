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
    max_anchor_distance = 0 # Optional, Default = 0. Allow hit in the first/last n bp.
                            # only valid if anchor != anywhere.
    search = "CTN" # May also be a list ["CTN", "GAN", ...]
    segment = 'read1' # Any of your input segments
    max_mismatches = 0 # required. How many mismatches are allowed
    on_tie = 'Earliest' # Earliest|LeftMost|RightMost 
                        # - decide what happens when multiple searches hit


```

Search and extract a sequence from the read, defined by one ore more [IUPAC
string(s)](https://doi.org/10.1093%2Fnar%2F13.9.3021).

The anchor decides where we search.

* Anywhere: Search for the IUPAC string everywhere in the segment, return left most hit.
* Left: Search at the beginning. Left most position is at most `max_anchor_distance`
* Right: Search at the end. Right most position is at most segment.len() - `max_anchor_distance`

`max_anchor_distance` mustnot be set if anchor is 'Anywhere'.

Ambiguous matches (e.g. query 'Y' matching 'C') do not count as mismatches, but
as full matches.

If you set `max_anchor_distance` > 0, the leftmost (`anchor`='Left') respectively
the rightmost (`anchor`='Right'`) hit for any given query is preferred.

This is independent of the on_tie setting which handles multiple queries.


## Multiple search queries

With multiple search queries, multiple may be present in your read.

If `anchor` is 'Left' or 'Right', and max_anchor_distance is 0,
the first from the list of `search` that hits within `max_mismatches` will be reported.

Otherwise, this is also the default.

This may be changed by setting `on_tie` to 'LeftMost' or 'RightMost' which will
instead take the left or right most occurrence.

No preference to lower hamming distances is given.




