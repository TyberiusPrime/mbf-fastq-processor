---
weight: 5
not-a-transformation: true

---
# Report

Capture data for the final report (see [the output section]({{< relref "docs/reference/output-section/_index.md" >}})).

You can add multiple reports, at any stage of your transformation chain
to get e.g. before/after filtering reports.

[Example report](../../../../html/example_report.html)

```toml
[[step]]
    action = 'Report'
    name = "report" # Key that the report will be listed under. Must be distinct
    count = true # count reads at this position
    base_statistics = false # include base distribution at each read position, q20, q30, total, gc bases
    length_distribution = false # capture read length distribution
    duplicate_count_per_read = false # count duplicates using Cukoo filter on each read1/read2/index1/index2
    duplicate_count_per_fragment = false # count duplicates using Cukoo filter, on concatenated read1/read2/index1/index2
    count_oligos = [] # if set, count these oligos in the segment. Full match only, no iupac
    count_oligos_segment = "all" # segment to count oligos in, can be 'all', 'read1', ...
```

Statistics available (for each 'segment'. If demultiplexd, per barcode combination):

- read counts
- total base count
- bases count q20 or better
- bases count q30 or better
- read length distribution
- AGTCN counts at each position
- expected error rate at each position
- duplicate count (if each read occurs twice, duplicate count = read count / 2)
