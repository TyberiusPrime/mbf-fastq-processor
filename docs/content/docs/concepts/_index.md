---
weight: 10
bookFlatSection: true
title: "Concepts"
---

# High level

Mbf-fastq-processor takes one (called read1), two (read1 & read2) or four (read1, read2, index1 & index2)
FastQ files, subjects them to a series of user defined steps,
and finally (optionally) outputs one, two, or four FastQ files.

Each step can either modify the reads, filter them, validate them, or collect information about them for
reports.

## Parameterisation

The complete configuration is defined in a TOML file, and steps happen in the order they're defined 
within the TOML. It is valid to have steps multiple times, for example to produce before and after filtering reports .

Values in the TOML file are generally mandatory, exceptions are noted in the reference.

## Output files

The output filenames are derived from a prefix, suffixed with _1, _2, _i1, _i2 plus the file suffix (e.g. .fq.gz) 
for read1, read2, index1, index2 respectively.

Report filenames have the same prefix, plus a report step specific infix.

## Target

Many steps take a 'target', which is the segment of a read they are used on - that is 'read1' | 'read2' | 'index1' | 'index2'. 

Note that the fragments from one molecules are always processed together - e.g. if you have a filter based on read1,
it will remove the corresponding read2 as well.


## Further reading

Please visit the [how-tos](../how-to/report) for workflow examples, or the [reference section](/reference) for a detailed description of the available steps.


