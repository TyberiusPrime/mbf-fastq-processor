Here's a very basic workflow that collects a report, but does not export a new FastQ.


```toml
[input]
    read1 = "myfastq.fq.gz"

[step]
    action = "Report"
    infix = "report"
    html = true
    json = false

[output]
    prefix = "myfastq_output"
    format = "None"
```

You will receive a file called "myfastq_output.report.html"
