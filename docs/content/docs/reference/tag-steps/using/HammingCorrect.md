# HammingCorrect

Correct a tag to one of a predefined set of 'barcodes' using closest hamming distance.


```toml
[[step]]
    action = "HammingCorrect"
    in_label = "mytag"
    out_label = "my_corrected_tag"
    barcodes = "mybarcodelist"
    max_hamming_distance = 1
    on_no_match = 'remove' # 'remove', 'empty', 'keep'
    name_split_character = '|' # optional

[barcodes.mybarcodelist]
    "AAAA" = "label_ignored" # only read when demultiplexing 
```
 
on_no_match controls what happens if the tag cannot be corrected within the max_hamming_distance:

 * remove: Remove the hit (location and sequence), useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
 * keep: Keep the original tag (and location)
 * empty: Keep the original location, but set the tag to empty.



Note that hamming_correction removes the location information on tags if 
they spanned more than one region. (This is an implementation limitation, not a conceptual one).

The barcodes defined must be disjoint under the given hamming distance and label.

That means 'AGT' -> 'label1' and 'CGT' -> 'label2', max_hamming_distance=1 will fail when encountering
e.g. 'GTT' (since it's within 1 hamming unit of either).

If the query matches multiple barcodes that all map to the same label, we correct to the 
closest one, breaking ties by lexicographic ordering.

You can influence whether we consider labels identical (default = complete name) 
by setting `name_split_character`, then they're considered identical if they match up 
to the first occurrence of `name_split_character`




