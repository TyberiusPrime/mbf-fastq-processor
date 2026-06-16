---
title: Output Report
weight: 73
---

# OutputReport

Write the combined run report (JSON and/or HTML) as a pipeline step. This is the
step-based equivalent of the legacy `[output]` `report_json` / `report_html`
options. The report is assembled from every report-producing step (e.g.
[Report](../../report-steps/report/)) after all steps have finalized.

At most one `OutputReport` step is allowed per pipeline; multiple would share the
same collected report data.

```toml
[input]
    read1 = "input.fq"

[[step]]
    action = "Report"
    name = "my_report"
    count = true

[[step]]
    action = "OutputReport"
    json = true    # write {prefix}.json
    html = false   # write {prefix}.html

[output]
    prefix = "output"
```

`{prefix}.json` and `{prefix}.html` are written next to the other outputs.
