---
weight: 52
title: Extract IUPAC with Indels
---

# ExtractIUPACWithIndel

```toml
[[step]]
    action = "ExtractIUPACWithIndel"
    out_label = "adapter"
    search = "AGTC"        # IUPAC pattern to align against
    max_mismatches = 1      # allowed substitutions (IUPAC-aware)
    max_indel_bases = 1     # total insertions + deletions allowed
    max_total_edits = 2     # optional overall edit ceiling
    anchor = 'Anywhere'     # Left | Right | Anywhere
    segment = 'read1'       # defaults to read1
```

Locate an [IUPAC](https://doi.org/10.1093%2Fnar%2F13.9.3021) pattern even when the read contains small insertions or deletions relative to the pattern. The extractor performs a semiglobal alignment (pattern vs. read segment) using IUPAC-aware scoring and returns the aligned span as a location tag.

## Parameters

- `out_label` – tag name inserted into downstream reads or tables.
- `search` – IUPAC string that defines the expected adapter/feature.
- `anchor` – constrain the alignment start/end (`Left` forces the match to start at the read beginning, `Right` forces the match to end at the read end, `Anywhere` allows internal matches).
- `max_mismatches` – maximum number of aligned substitutions after IUPAC resolution.
- `max_indel_bases` – maximum total bases involved in insertions and deletions (sum of gaps in either sequence).
- `max_total_edits` – optional limit on `mismatches + indels`; defaults to `max_mismatches + max_indel_bases` when not set, letting you tighten the budget without changing the individual caps.
- `segment` – which input read to inspect; defaults to `read1`.

When multiple positions satisfy the constraints, the left-most alignment is chosen. If the thresholds are exceeded or the pattern is longer than the permitted alignment window, the tag value becomes missing.
