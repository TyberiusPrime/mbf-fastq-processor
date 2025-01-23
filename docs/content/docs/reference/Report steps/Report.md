---
weight: 5
---
# Report

Write a statistics report, machine-(json)
or human-readable (HTML with fancy graphs).

You can add multiple reports, at any stage of your transformation chain
to get e.g. before/after filtering reports.

```toml
[[step]]
    action = 'Report'
    infix = "report" # a string to insert into the filename, between output.prefix and .html/json
    html = true # bool, wether to output html report (not yet implemented)
    json = true # bool, wether to output json report
```

Statistics available (for each 'segment'):

- read counts
- total base count
- bases count q20 or better
- bases count q30 or better
- read length distribution
- AGTCN counts at each position
- expected error rate at each position
- duplication rate 

