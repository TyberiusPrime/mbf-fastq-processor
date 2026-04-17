# AssignToReference

Assign each query sequence to the closest entry in barcodes section,
using Hamming distance.

( As opposed to [HammingCorrect]({{< relref "docs/redirects/HammingCorrect.md" >}})
which will correct to the closest barcode sequence).

At start-up the step builds an efficient Hamming-distance index over the database.  For every read,
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
    max_hamming_distance = 2
    barcodes = 'reference_barcodes'

[barcodes.reference_barcodes]
    from_file = {
            filename= "reference.fa"
    }

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
| `max_hamming_distance` | integer | yes | Maximum allowed Hamming distance.  Use `0` for exact matches only. |
| `barcodes` | yes | String | Which barcode section to reference. |

## Notes

* All sequences in the reference file **must have the same length** as the
  query sequences.
* If the reference contains duplicate sequences, an error will occur during the initial reading.
  Multiple sequences leading to the same label are ok.
* The `in_label` tag can be a **String** tag (e.g. from
  [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}}))
  or a **Location** tag (e.g. from
  [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}})).
* Use [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}})
  instead when you want to correct a tag to one of the barcodes
  barcodes.
