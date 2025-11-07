---
title: Introduction
type: docs
---

# mbf-fastq-processor

{{< columns >}}
Reproducible, memory safe FASTQ transformations.

<--->

Graph based description of the required transformations.

Great UX without surprises.



{{< /columns >}}

The swiss army knife of FASTQ (pre-)processing: filter, sample, demultiplex, and report on sequencing reads with explicit, auditable configuration.

## Install

- Pre-compiled releases available for Linux and windows from the [releases page](https://github.com/TyberiusPrime/mbf-fastq-processor/releases)
- To build Rust toolchain 1.86+ and zlib / libzstd development files are required
- Alternatively, a nix flake is provided for a fully reproducible environment
- Container image: `ghcr.io/tyberiusprime/mbf-fastq-processor:latest` (works with Docker or Podman)

## Quickstart

1. Prepare a configuration file `input.toml` (see example below).
2. Run `mbf-fastq-processor template >input.toml` to create a configuration file. 
   Edit as necessary.
3. Run `mbf-fastq-processor process input.toml` (or `cargo run --release -- process input.toml` during development).
3. Inspect generated FASTQ files or HTML/JSON reports (the [Inspect]({{< relref "docs/reference/report-steps/Inspect.md" >}}) step helps surface summaries).

```toml
[input]
    read1 = ['myreads.fq']

[[step]]
    action = "Head"
    n = 5000

[[step]]
    action = "Report"
    label = "qc"
    html = true

[output]
    prefix = "output"
    format = "Raw"
```

```bash
mbf-fastq-processor input.toml
```

Or run the published container (bind-mount your working directory to `/work`):

```bash
docker run --rm -v "$(pwd)":/work ghcr.io/tyberiusprime/mbf-fastq-processor:latest process input.toml
```

You will find `output_read1.fq` alongside a [sample HTML report](html/example_report.html) at `output_qc.html`.

## Documentation guide

- Read the [Concepts]({{< relref "docs/concepts/_index.md" >}}) chapter first for a mental model of molecules, segments, and processing steps.
- Dive into the [Reference]({{< relref "docs/reference/_index.md" >}}) for exhaustive step-by-step configuration details.
- When you are ready to compose real pipelines, consult the (work-in-progress) [How-Tos]({{< relref "docs/how-to/cookbooks/_index.md" >}}) for applied recipes and integrations.


## Development

Looking for validations or edge cases? 

We have extensive end-to-end tests in `test_cases`/ that cover a wide range of scenarios.
