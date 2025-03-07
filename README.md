# mbf_fastq_processor

The swiss army knife of FastQ (pre-)processing.

It filters, samples, slices, dices, analysis, demultiplexes  and generally
does all the things you might want to do with a set of FastQ files.

It's chief concern is correctness ... correctness and flexibility ... flexibility and correctness.

It's two concerns are correctness and flexibility ... and speed.

It's three main objectives are correctness, flexibility, speed and reproducible results.

It's four... no amongst it's objectives are such element as...

## Full list of FastQ manipulations supported

Please refer to the 'step' sections of our our [reference
documentation](https://tyberiusprime.github.io/mbf_fastq_processor/docs/reference/filter-steps/)

## Status

It's in beta until the 1.0 release, but already quite usable.

All the major functionality and testing is in place.

## Installation

This repo is a [nix flake](https://nixos.wiki/wiki/flakes).

There are statically-linked binaries in the github releases section that will run on any linux with a recent enough glibc.

Currently not packaged by any distribution.

Windows binaries are build for each release - be advised that these do not see much testing.

It's written in rust, so `cargo build --release` should work as long as you have zstd and cmake around.

## Usage

(Refer to the [full documentation](https://tyberiusprime.github.io/mbf_fastq_processor/) for details)

CLI: `mbf_fastq_processor input.toml`

We use a [TOML](https://toml.io/en/) file for configuration,
because command lines are too limited and prone to misunderstandings.

And you should be writing down what you are doing anyway.

Here's a brief example:

```toml
[input]
    # supports multiple input files.
    # in at least three autodetected formats.
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd']
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd']


[[step]]
    # we can do a flexible report at any point in the pipeline
    # filename is output.(html|json)
    action = 'Report'
    label = "initial"
    duplication_count = true
    counts = true
    base_distribution = true

[[step]]
    # take the first five thousand reads
    action = "Head"
    n = 5000

[[step]]
    # extract umi and place it in the read name
    action = "ExtractToName"
    # the umi is the first 8 bases of read1
    regions = [{source: 'read1', start: 0, length: 8}]

[[step]]
    # now remove the UMI from the read sequence
    action = "CutStart"
    target = 'Read1'
    n = 8

[[step]]
    action = "Report"
    counts = true
    label = "post_filter"

[output]
    #generates output_1.fq and output_2.fq. For index reads see below.
    prefix = "output"
    # uncompressed. Suffix is determined from format
    format = "Raw"

    report_json = true
    report_html = true 
```


## Citations

A manuscript is being drafted.
