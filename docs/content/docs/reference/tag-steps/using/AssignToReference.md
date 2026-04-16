# AssignToReference

Assign each query sequence to the closest entry in a named reference database
using Hamming distance.

The reference is a FASTA FASTQ or BAM file (optionally compressed) whose
records each have a fixed-length sequence and a name.  At start-up the step
builds an efficient Hamming-distance index over the database.  For every read,
the tag supplied in `in_label` is looked up in the index and the name of the
closest matching reference entry is written to `out_label` as a string tag.

When no reference entry falls within `max_hamming_distance` the output tag is
set to `Missing`, which can be acted on by a downstream
[FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) step.

Missing tags lead to empty strings when exported.

For BAM input, all 'reads' from the BAM file independent of alignment status
are included.


```toml
# 1. Extract the region you want to compare against the reference.
# (Even do so if you're querying with the complete sequence)
[[step]]
    action = "ExtractRegion"
    source = "read1"
    start = 0
    length = 50
    anchor = "Start"
    out_label = "query_seq"

# 2. Look up the extracted sequence in the reference database.
[[step]]
    action = "AssignToReference"
    in_label = "query_seq"
    out_label = "ref_name"
    reference = "reference.fa"
    max_hamming_distance = 2

# 3. (Optional) Discard reads that did not match any reference entry.
[[step]]
    action = "FilterByTag"
    in_label = "ref_name"
    keep_or_remove = "keep"

# 4. (Optional) Store the reference name in the read comment.
[[step]]
    action = "StoreTagInComment"
    in_label = "ref_name"
```

## Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `in_label` | tag name | yes | Tag holding the query sequence (String or Location tag). |
| `out_label` | tag name | yes | Output tag for the matched reference name (String). |
| `reference` | file path | yes | FASTA or FASTQ reference file (plain or gzip-compressed). |
| `reference_read_comment_character` | character | no | Cut reference read names at first occurances. Default: no cut |
| `max_hamming_distance` | integer | yes | Maximum allowed Hamming distance.  Use `0` for exact matches only. |

## Notes

* All sequences in the reference file **must have the same length** as the
  query sequences.
* If the reference contains duplicate sequences, an error will occur during the initial reading.
* The `in_label` tag can be a **String** tag (e.g. from
  [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}}))
  or a **Location** tag (e.g. from
  [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}})).
* Use [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}})
  instead when you want to correct a tag to one of a small set of known
  barcodes that are already embedded in the config file (on which you want to demultiplex).
