Here's a very basic workflow that collects a report, but does not export a new FastQ.


```toml
[input]
    read1 = "myfastq.fq.gz"

[[step]]
    action = "Report"
    label = "report"

[output]
    prefix = "myfastq_output"
    format = "None"
    report_html = true
    report_json = false
```

You will receive a file called "myfastq_output.html"
