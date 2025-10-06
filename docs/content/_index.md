---
title: Introduction
type: docs
---

# mbf-fastq-processor

{{< columns >}}
Reproducible, memory safe FastQ transformations.

<--->

Graph based description of the required transformations.

Great UX without suprises.



{{< /columns >}}

The swiss army knife of FastQ (pre-)processing.

It filters, samples, slices, dices, analyses, demultiplexes and generally
does all the things you might want to do with a set of [FastQ](https://en.wikipedia.org/wiki/FASTQ_format) files.

## Microexample

To process a FastQ file 'myreads.fq', create a report on the first 5000 reads,
and write them to 'output_1.fq', write a toml file (`input.toml`) like this:

```toml
[input]
    read1 = ['myreads.fq']

[[step]]
    action = "Head"
    n = 5000

[[step]]
    action = "Report"
    infix = "report"
    json = false
    html = true 

[output]
    prefix = "output"
    format = "Raw"
```

Run ```mbf-fastq-processor input.toml```, receive the bundled [example report](html/example_report.html) in `output_report.html`.

