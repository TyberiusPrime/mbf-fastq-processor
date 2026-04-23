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
    on_tie = 'remove' # 'remove', 'empty', 'keep', 'first', 'first_strict', 'fail', 'by_majority'
    name_split_character = '|' # optional

[barcodes.mybarcodelist]
    "AAAA" = "label_ignored" # only read when demultiplexing 
```
 
`on_no_match` controls what happens if the tag cannot be corrected within the max_hamming_distance:

 * remove: Remove the hit (location and sequence), 
   useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
 * keep: Keep the original tag (and location)
 * empty: Keep the original location, but set the tag to empty.


`on_tie` controls what happens when if the tag can't be corrected uniquely, i.e. there are
multiple equally good matches within `max_hamming_distance` (lower distance matches are always prefered).

* remove: Remove the hit (location and sequence), 
   useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
* keep: Keep the original tag (and location)
* empty: Keep the original location, but set the tag to empty.
* first: Just take the first (lexicographical ordering)
* first_strict: Take the first (lexicographical ordering),
  but only if they all map to the same barcode label (or label prefix up to the first `name_split_character`, if that is set)
  Otherwise, see fail.
* fail: Throw a runtime error
* by_majority: Break ties toward the majority barcode. See below.
 

Note that hamming_correction removes the location information on tags if
they spanned more than one region. (This is an implementation limitation, not a conceptual one).






