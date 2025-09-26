---
weight: 10
bookFlatSection: true
title: "Concepts"
---

# High level

Mbf-fastq-processor takes one FastQ files, subjects them to a series of user defined steps,
and finally (optionally) outputs one, two, or four FastQ files.

Each step can either modify the reads, filter them, validate them, or collect information about them for
reports.

There are no 'default' steps applied to your data. What's defined in the input TOML is the
complete set.

## Parameterisation

The complete configuration is defined in a TOML file, and steps happen in the order they're defined 
within the TOML. It is valid to have steps multiple times, for example to produce before and after filtering reports .

Values in the TOML file are generally mandatory, exceptions are noted in the reference.

## Input files

Mbf-fastq-processor processes FASTQ files in uncompressed, gziped or zstd formats.

It supports an arbitrary number of 'segments per fragment', i.e. reads  per molecule,
such as the typically read1/read2 Illumina pairs, or read1/read2/index1/index2 quadruples.

FASTQ files are expected to follow the format defined in (Cock et al)[https://academic.oup.com/nar/article/38/6/1767/3112533].

Data on the + line of reads are ignored, and will be omitted in output.

## Output files

The output filenames are derived from a prefix, suffixed with '_' plus the name they were given 
in the `[input]` section of the configuration. Typically suffixes are for example _read1, _read2,
_index1, _index2 or _r1, _r2.

Reports are named `prefix.json` and `prefix.html`

## Steps

FastQ files are processed in any number of 'steps' (see reference section).

Steps always 'see' complete molecules.

## Target

Many steps take a 'segment', which is the segment of a read they are used on - the names of which are taken from the input section.

Note that the fragments from one molecules are always processed together - e.g. if you have a filter based on read1,
it will remove the corresponding read2 as well.


## Further reading

Please visit the [how-tos](../how-to/report) for workflow examples, or the [reference section](/reference) for a detailed description of the available steps.
