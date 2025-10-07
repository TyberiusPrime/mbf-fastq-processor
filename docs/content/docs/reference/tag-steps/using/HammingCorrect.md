# HammingCorrect

Correct a tag to one of a predefined set of 'barcodes' using closest hamming distance.


```toml
[[step]]
    action = "HammingCorrect"
    label_in = "extracted_tag"
    label_out = "corrected_tag"
    barcodes = "mybarcodelist"
    max_hamming_distance = 1
    on_no_match = 'remove' # 'remove', 'empty', 'keep'

[barcodes.mybarcodelist]
    "AAAA" = "ignored" # only read when demultiplexing 
```
 
on_no_match controls what happens if the tag cannot be corrected within the max_hamming_distance:

 * remove: Remove the hit (location and sequence), useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
 * keep: Keep the original tag (and location)
 * empty: Keep the original location, but set the tag to empty.



