# mbf-fastq-processor

The multi-tool of FASTQ (pre-)processing.

It filters, samples, slices, dices, analysis, demultiplexes  and generally
does all the things you might want to do with a set of FASTQ files.

It's chief concern is correctness ... correctness and flexibility ... flexibility and correctness.

It's two concerns are correctness and flexibility ... and speed.

It's three main objectives are correctness, flexibility, speed and reproducible results.

It's four... no amongst it's objectives are such element as...

## Getting started right away

### 1. Define temporary run command
`ABOVE="nix run github:TyberiusPrime/mbf-fastq-processor"` 

or

`ABOVE="docker run docker run --rm ghcr.io/tyberiusprime/mbf-fastq-processor:latest"`

### 2. Run Your First Pipeline 

Generate a basic quality report configuration from our example cookbook entry 01:

`$ABOVE cookbook 01 > my-first-pipeline.toml`

Edit the input section to point to your FASTQ files:

`nano my-first-pipeline.toml`:

Run it:

`$ABOVE my-first-pipeline.toml`

### 3. View your report
`xdg-open output_report.html`


## Documentation

We have [extensive documentation](https://tyberiusprime.github.io/mbf-fastq-processor/main) following the Diátaxis framework.

Further examples can be found in the [cookbook section](https://tyberiusprime.github.io/mbf-fastq-processor/main/docs/how-to/cookbooks/).

### Language Server (IDE Support)

We provide a Language Server Protocol (LSP) implementation for enhanced IDE support when editing configuration files. Features include:

- **Auto-completion** for step actions, configuration keys, and section headers
- **Inline validation** with real-time error checking
- **Hover documentation** showing detailed information about steps and parameters

The language server works with VS Code, Neovim, Helix, Emacs, and any other LSP-compatible editor.

See [docs/language-server.md](docs/language-server.md) for installation and setup instructions.

## Full list of FastQ manipulations supported

Please refer to the 'step' sections of our our [reference
documentation](https://tyberiusprime.github.io/mbf-fastq-processor/docs/main/reference/filter-steps/)

## Status

It's in beta until the 1.0 release, but already quite usable.

All the major functionality and testing is in place, and I don't anticipate breaking changes.


## Installation

This repo is a [nix flake](https://nixos.wiki/wiki/flakes).

There are statically-linked binaries in the github releases section that will run on any linux with a recent enough glibc.

Currently not packaged by any distribution.

Windows and MacOS binaries are build for each release - be advised that these do not see much testing.

It's written in [rust](https://rust-lang.org/), so `cargo build --release` should work as long as you have zstd and cmake around. The nix flake does offer a fully reproducible build and development environment. Same goes for `cargo install mbf-fastq-processor`.


### Container image

A ready-to-run OCI image is published with each tag at `ghcr.io/tyberiusprime/mbf-fastq-processor`.

```bash
# Docker
docker pull ghcr.io/tyberiusprime/mbf-fastq-processor:latest
docker run --rm ghcr.io/tyberiusprime/mbf-fastq-processor:latest --help

# Podman
podman pull ghcr.io/tyberiusprime/mbf-fastq-processor:latest
podman run --rm ghcr.io/tyberiusprime/mbf-fastq-processor:latest --help
```

Mount your working directory to feed a pipeline configuration:

```bash
docker run --rm -v "$(pwd)":/work ghcr.io/tyberiusprime/mbf-fastq-processor:latest process input.toml
```

## Usage

Refer to the [full documentation](https://tyberiusprime.github.io/mbf-fastq-processor/) or the
binaries help page (shown when run without arguments) for details.

CLI: `mbf-fastq-processor process input.toml`

We use a [TOML](https://toml.io/en/) file for configuration,
because command lines are too limited and prone to misunderstandings.

And you should be writing down what you are doing anyway.

Here's a brief example:

```toml
[input]
    # supports multiple input files.
    # in at least three autodetected formats.
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    read2 = ['fileA_2.fastq', 'fileB_2.fastq.gz', 'fileC_2.fastq.zstd']
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd']
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd']


[[step]]
    # we can do a flexible report at any point in the pipeline
    # filename is output.(html|json)
    action = 'Report'
    name = "initial"
    duplicate_count_per_read = true
    count = true
    base_statistics = true

[[step]]
    # take the first five thousand reads
    action = "Head"
    n = 5000

[[step]]
    # extract UMI 
    action = "ExtractRegions"
    out_label = "region"
    # the umi is the first 8 bases of read1
    regions = [{segment = 'read1', start = 0, length = 8}]

[[step]]
    #and place it in the read name
    action = "StoreTagInComment"
    in_label = "region"

[[step]]
    # now remove the UMI from the read sequence
    action = "CutStart"
    segment = 'read1'
    n = 8

[[step]]
    action = "Report"
    count = true # include read counts
    name = "post_filter"

[output]
    #generates output_1.fq and output_2.fq. For index reads see below.
    prefix = "output"
    # uncompressed. Suffix is determined from format
    format = "FASTQ"
    compression = "Raw"

    report_json = true
    report_html = true
```

### Canonical template

The repository ships an authoritative configuration scaffold at [`src/template.toml`](src/template.toml).
When prompting an LLM or drafting a new pipeline, point it to that file so it can reference
the full set of supported sections, comments, and examples.

### Cookbooks

Looking for practical examples? Check out the [`cookbooks/`](cookbooks/) directory for complete,
runnable examples demonstrating common use cases, or [visit them in the documentation](https://tyberiusprime.github.io/mbf-fastq-processor/main/docs/how-to/cookbooks/):

- **Basic Quality Report** - Generate comprehensive quality metrics from FastQ files
- **UMI Extraction** - Extract and handle Unique Molecular Identifiers
- And many more...

Each cookbook includes:
- Sample input data
- Fully documented configuration files
- Expected output for verification
- Detailed README explaining the use case

Run any cookbook with:
```bash
git clone https://github.com/tyberiusprime/mbf-fastq-processor
cd cookbooks/[cookbook-name]
mbf-fastq-processor process input.toml
```

## Citations

A manuscript is being drafted.


## Contributions

PR's welcome.

If at any point you find the tool not doing what you expected it to,
please open an issue so we can discuss how to improve it!
