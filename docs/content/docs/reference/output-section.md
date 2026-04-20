---
weight: 4
not-a-transformation: true
---

# Output section

The `[output]` table controls how transformed reads and reporting artefacts are written.

```toml
[output]
    prefix = "output"          # required.
    format = "Fastq", # (optional) output format, defaults to 'Fastq'
					  # Valid values are: Fastq, Fasta, BAM and None (for no sequence output)
    compression = "Gzip"        # Raw | Uncompressed | Gzip | Zstd | None (default: Raw)
    compression_threads = 5        # (optional) number of threads to use for compressing gzip data
    suffix = ".fq.gz"           # optional override; inferred from format when omitted
    compression_level = 6       # gzip: 0-9, zstd: 1-22, bam: 0-9 (BGZF); defaults are gzip=6, zstd=5
    ix_separator = "_"          # optional separator between prefix, infixes, and segments. Defaults to '_'

    report_json = false         # write prefix.json (default: false)
    report_html = true          # write prefix.html (default: false)
    report_timing = false          # write prefix.timing.json (default: false)

    output = ["read1", "read2"] # limit which segments become FASTQ files
    interleave = false          # emit a single interleaved FASTQ
    stdout = false              # stream to stdout instead of files
    chunk_size = 100000         # Write multiple, numbered output files, each a maximum of chunk_size reads/molecules.

    output_hash_uncompressed = false
    output_hash_compressed = false


[output.bam] # only valid when format = bam

    comment_separation_char = ' ' # read parts after this get put into a 'CO' tag (free-text comment, null terminated in BAM)
    tag_to_bam_tag = {'a-tag-label': 'XY'} # write a-tag-label to bam tag 'xz'
    tag_to_reference = {
        tag = 'assigned-barcode-name',
        # needs one of 
        # references_from_bam = "template.bam",  # take reference information from here
        # or
        references_from_barcodes = 'barcode-section-name' # take references from this barcode section.
    }
```

| Key                                                   | Default             | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ----------------------------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `prefix`                                              | `"output"`          | Base name for all files produced by the run.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `format`                                              | `"Fastq"`           | Output format. Valid values are: `Fastq`, `Fasta`, `Bam`, and `None` (for no sequence output).                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `compression`                                         | `"Uncompressed"`    | Compression format for read outputs. Valid values are: `Gzip`, `Zstd`, `Uncompressed` (alias: `"Raw"`). Must not be set for BAM                                                                                                                                                                                                                                                                                                                                                                                                          |
| `compression_threads`                                 | auto    | if using gzip compression, how many thread should be used for compression. See [threading]({{< relref "docs/reference/threading.md" >}}) |
| `suffix`                                              | derived from format | Override file extension when interop with other tooling demands a specific suffix.                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `compression_level`                                   | gzip: 6, zstd: 5    | Fine-tune compression effort. Ignored for `Raw`/`None`. `Bam` maps directly to the BGZF level (0–9).                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `report_json` / `report_html`                         | `false`             | Toggle structured or interactive reports.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `report_timing`                                       | `false`             | Emit a JSON file with detailed timing information for all steps.                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `output`                                              | all input segments  | Restrict the subset of segments written to disk. Use an empty list to suppress FASTQs while still running steps that depend on fragment data.                                                                                                                                                                                                                                                                                                                                                                                            |
| `interleave`                                          | `false`             | Generate a single interleaved FASTQ (`{prefix}_interleaved.fq*`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `stdout`                                              | `false`             | Write to stdout. Forces `format = "Raw"`. `Sets interleave=true` if more than one fragment is listed in `output`                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `output_hash_uncompressed` / `output_hash_compressed` | `false`             | Emit SHA-256 checksums.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `ix_separator`                                        | `"_"`               | Separator inserted between `prefix`, any infix (demultiplex labels, inspect names, etc.), and segment names.                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `chunk_size`                                          | (unlimited)         | Split outputs into multiple files, each containing at most `chunk_size` reads/molecules. For non-interleaved output files, it's `chunk_size` reads, for interleaved files it's molecules. This means when mixing interleaved and non-interleaved output, you get the same number of files. Files are numbered sequentially, e.g. `output_read1_0.fq.gz`, ..., Numbers start at 0 and use the minimum number of (base 10) digits necessary for alphabetical sorting (by renaming already produced files whenever an extension is needed). |

Generated filenames join these components with `ix_separator` (default `_`), e.g. `{prefix}_{segment}{suffix}`. Interleaving replaces `segment` with `interleaved`; demultiplexing adds per-barcode infixes before the segment. Checksums use `.uncompressed.sha256` or `.compressed.sha256` suffixes.

Compression format and suffix are independent: overriding the suffix will not change the actual compression algorithm.


## BAM output

### References

By default, BAM output is _unaligned_, and no references are defined in the BAM header.

You can define references, either from a template BAM, or from a set of barcodes
(length = barcode length) using `output.bam.tag_to_reference.references_from_bam`
or `output.references_from_barcodes`, which then get embedded in the BAM header.

This then enables using a string tag as the reference of a read. The tag 
must match one of the defined reference (or be empty/missing = unaligned).
Position is always 1.

This is useful for example to later quantify 'looked up' reads with [mbf-bam-quantifier](https://tyberiusprime.github.io/mbf-bam-quantifier/)
in 'preassigned [bam tag' mode](https://tyberiusprime.github.io/mbf-bam-quantifier/docs/reference/input-section/#bam_tag).

### Other tags

You can store arbitrary tags into BAM tags using `output.bam.tag_to_bam_tag'.
It is a dict of 'our-tag-name' -> 'two-letter-bam-tag-name'. See the [SAM/BAM
tag spec](https://samtools.github.io/hts-specs/SAMtags.pdf) for predefined
codes. This works for string & location tags (-> type Z), numeric tags (-> type
float), and boolean tags (-> u8 0/1). Virtual tags are currently unsupported.


### Read name considerations

BAM read names are required to be ascii-readable-letters-minus-@ [!-?A-~] and shorter than 
254 bytes. We enforce this on output. You can enforce it earlier using the 
[ValidateReadNamesPrintable]({{< relref "docs/reference/validation-steps/ValidateReadNamesPrintable.md" >}})
step.

If you read name get's too long because of tags, consider shuffling them into a comment tag
by setting `output.bam.comment_separation_char` which places everything beyond this 
character in a BAM comment tag called 'CO', or not storing them in the read name 
but in BAM tags instead (see previous section).


### Interleaved BAM output
Interleaved produces one paired BAM with appropriate SAM flags for first/last segment.

### Other

BAM output cannot be streamed to stdout and requires `output_hash_uncompressed = false` (compressed hashes continue to work).


### Example output files.

#### As above

The above configuration produces:

- `output_read1.fq.gz` # .fq is the default suffix for raw, .fq.gz for gzip
- `output_read2.fq.gz`
- `output.html` # HTML report

#### If Interleaved was set

- `output_interleaved.fq.gz`
- `output.html` # HTML report

### No sequence output

Set `format = "None"` or `output = []` when you only need reports or tag quantification.
A `prefix` is still required so report files can derive a name.

See also the [Report steps reference]({{< relref
"docs/reference/report-steps/_index.md" >}}) for producing summaries, and the
[Demultiplex documentation]({{< relref "docs/reference/Demultiplex.md" >}}) for
how barcode outputs influence file naming.

There is a slight conceptual difference: Setting `output = []` 
will complain if there is no other file being produces (report, tables, etc).
Setting `format = "None`" will not.

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
