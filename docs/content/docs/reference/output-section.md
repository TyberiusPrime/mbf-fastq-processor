---
weight: 4
not-a-transformation: true
---

# Output section


```toml
[output]
    prefix = "output" # files get named {prefix}_1{suffix}, _2, _i1, _i2. Default is 'output'
    format = "Gzip" # (optional), defaults to 'Raw'
                    # Valid values are Raw, Gzip, Zstd and None
                    # None means no fastq output (but we need the prefix for Reports etc.)
    suffix = ".fq.gz" # optional, determined by the format if left off.
    compression_level = 6 # optional compression level for gzip (0-9) or zstd (1-22)
                          # defaults: gzip=6, zstd=5

    report_json = false # (optional) write a json report file ($prefix.json)? 
    report_html = false # (optional) write an interactive html report report file ($prefix.html)? 


    stdout = false # write Read1 to stdout, do not produce other fastq files.
                   # set's interleave to true (if Read2 is in input),
                   # format to Raw
                   # You still need to set a prefix for
                   # Reports/keep_index/Inspect/QuantifyRegion(s)
                   # Incompatible with a Progress Transform that's logging to stdout

    interleave = false # (optional) interleave fastq output, producing
                       # only a single output file for read1/read2
                       # (with infix _interleaved instead of '_1', e.g. 'output_interleaved.fq.gz')
    keep_index = false # (optional) write index (i1/i2) files as well? (optional)
                       # (independent the interleave setting. )

    output_hash_uncompressed = false # (optional) write a {prefix}_{1|2|i1|i2}.uncompressed.sha256
                                     # with a hexdigest of the uncompressed data's sha256,
                                     # similar to what sha256sum would do on the raw FastQ
    output_hash_compressed = false   # (optional) write a {prefix}_{1|2|i1|i2}.compressed.sha256
                                     # with a hexdigest of the compressed output file's sha256,
                                     # allowing verification with sha256sum on the actual output files

```

Generates files named output_1.fq.gz, output_2.fq.gz, (optional output_index1.fq.gz, output_index2.fq.gz if keep_index is true)

Compression is independent of file ending if suffix is set.

Supported compression formats: Raw, Gzip, Zstd (and None, see next section)

### No FastQ output

If you want to run mbf-fastq-processor just for a report / region quantification,
you can disable the generation of fastq output with `format = 'None'`.

You will still need to supply a prefix, it's needed for the report filenames.

See [Report steps]({{< relref "docs/reference/report-steps/_index.md" >}}) for more information.
