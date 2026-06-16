---
weight: 4
not-a-transformation: true
---

# Output section

The `[output]` table controls how transformed reads and reporting artefacts are written.

```toml
[output]
    prefix = "output"          # required.
    ix_separator = "_"          # optional separator between prefix, infixes, and segments. Defaults to '_'
    compression_threads = 5        # (optional) number of threads to use for compressing gzip data

```

| Key                                                   | Default             | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ----------------------------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `prefix`                                              | `"output"`          | Base name for all files produced by the run.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `ix_separator`                                        | `"_"`               | Separator inserted between `prefix`, any infix (demultiplex labels, inspect names, etc.), and segment names.                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `compression_threads`                                 | auto    | if using gzip compression, or bam output, how many thread should be used for compression. See [threading]({{< relref "docs/reference/threading.md" >}}) |

Generated filenames join these components with `ix_separator` (default `_`), e.g. `{prefix}_{segment}{suffix}`. Interleaving replaces `segment` with `interleaved`; demultiplexing adds per-barcode infixes before the segment. Checksums use `.uncompressed.sha256` or `.compressed.sha256` suffixes.


For actual output writing, see the following steps:

- [OutputFASTQ]({{< relref "docs/reference/output-steps/OutputFASTQ.md" >}})- 
- [OutputBAM]({{< relref "docs/reference/output-steps/OutputBAM.md" >}})- 
- [OutputFASTA]({{< relref "docs/reference/output-steps/OutputFASTA.md" >}})- 

Compression format and suffix are independent: overriding the suffix will not change the actual compression algorithm.

Note that a pipeline requires at least one output file generating job, i.e one of the above
or for example:

- [OutputReport]({{< relref "docs/reference/output-steps/OutputReport.md" >}})
- [StoreTagInFastQ]({{< relref "docs/reference/tag-steps/using/StoreTagInFastQ.md" >}}) 
- [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}})
- [Inspect]({{< relref "docs/reference/report-steps/Inspect.md" >}})

## Named pipe outputs

Output files may be (preexisting) named pipes (FIFOs).

## Overwrite protection

If any output file already exists, fastqrab will refuse to overwrite them.

Except when the incompletion marker (see below) is present.

### (In-)Completion marker

Every run writes `{prefix}.incompleted` in the output directory before any other file handles are opened.
The file is deleted once processing finishes, so its presence later indicates an interrupted run.

Because the marker predates other outputs, reruns detect its presence and permit overwriting prior artefacts without manual cleanup.

If the process aborts for any reason, the marker stays behind.
