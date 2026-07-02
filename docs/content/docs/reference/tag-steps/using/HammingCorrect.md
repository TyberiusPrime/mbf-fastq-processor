# HammingCorrect

Correct a tag to one of a predefined set of 'barcodes' using closest hamming distance.


```toml
[[step]]
    action = "HammingCorrect"
    in_label = "mytag"
    out_label = "my_corrected_tag"
    barcodes = "mybarcodelist"
    output = "barcode" # or "label"

    max_hamming_distance = 1
    on_no_match = 'remove' # 'remove', 'keep'
    on_tie = 'by_majority' # 'remove', 'keep', 'first', 'first_strict', 'fail', 'by_majority', 'by_edit_probability'
    name_split_character = '|' # optional

    on_tie_threshold = 0.975 # optional, default 0.975. Adjusted frequency: (n+1) / (N+k)
    on_tie_min_molecules_to_start = 1_000_000 # optional, default 1_000_000. See below
    on_tie_use_counts_from_report = { # optional, See below
            file_name = "report.json",
           report_name = "barcodes",
           tag_name = "whitelisted"
    }

[barcodes.mybarcodelist]
    "AAAA" = "label_ignored" # only read when demultiplexing 
```


## Output 

`HammingCorrect` can either output a corrected barcode,
or the corrected barcodes' label. 
Use the `output` setting for this. It defaults to 'barcode'

Either way, the result is a string tag, the locations are 'lost' along the way.

## Non matching barcode handling
 
`on_no_match` controls what happens if the tag cannot be corrected within the max_hamming_distance:

 * remove: Remove the hit (location and sequence), 
   useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
 * keep: Keep the original tag (and location)


`on_tie` controls what happens when if the tag can't be corrected uniquely, i.e. how to handle 
multiple equally good matches within `max_hamming_distance` (lower distance matches are always preferred).

* remove: Remove the hit (location and sequence), 
   useful for [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) later.
* keep: Keep the original tag (and location)
* first: Just take the first (lexicographical ordering)
* first_strict: Take the first (lexicographical ordering),
  but only if they all map to the same barcode label (or label prefix up to the first `name_split_character`, if that is set).
  Otherwise, see fail.
* fail: Throw a runtime error
* by_majority: Break ties toward the majority barcode. See below.
* by_edit_probability: Break ties by majority barcode, 
  moderated by per-base-edit probabilities (max hamming: 1). See below.
 
Note that hamming_correction removes the location information on tags if
they spanned more than one region. (This is an implementation limitation, not a conceptual one).


## By majority tie breaking

`ByMajority` assigns read sequences that are equidistant to multiple barcodes to the 
most common barcode, if the that barcode has an observed frequency >= `by_majority_threshold`
within the equidistant barcodes. 

By default, frequency is estimated from the number-of-previous-reads with 
an exact match to this barcode + 1 (Laplace correction).

If none of the candidates satisfy the frequency, the tag is set to 'Missing' (see 'Remove' above).

To be able to correct 'early reads', fastqrab delays correction (and output) until
the first `by_majority_min_molecules_to_start` have been counted.

This is a trade-off. Ideally, you'd want to consider all reads for the frequency estimation,
but that either requires processing the input files twice, or buffering everything in memory.

You can tune memory usage by adjusting `by_majority_min_molecules_to_start`. 
The default (1 million reads) should require less than a gigabyte of RAM for 300 bp reads.

### Loading whitelisted barcode frequencies from previous report

But if you need to use *all* reads for estimation,
you can first run a report with tag_histogram on the whitelisted barcodes,
then use that report file in `on_tie_use_counts_from_report`.

Use a configuration like this adapted to your needs to generate the necessary report
```toml # ignore_in_test
[output]
    prefix = 'barcode_histogram'
    report_json = true

[barcodes.cells.from_file]
    file_name = "cell_barcodes.txt.gz"

[[step]]
    action = "ExtractRegion"
    start = 0
    length = 16
    segment = "read1"
    out_label = "cell_barcode_input"
    anchor = "left"

[[step]]
    action = "HammingCorrect"
    max_hamming_distance = 0
    barcodes = "cells"
    in_label = "cell_barcode_input"
    out_label = "cell_barcode_corrected"
    on_no_match = "remove"
    on_tie = "remove"
#
[[step]]
    action = "Report"
    name = 'cb_corrected'
    tag_histogram = ['cell_barcode_corrected']
```

Then in your actual run 
```toml # ignore_in_test
[[step]]
    action = "HammingCorrect"
#   ...
on_tie_use_counts_from_report = {
    file_name = "barcode_histogram.json,
    report_name = 'cb_corrected'
    tag_name = 'cell_barcode_corrected
```

No frequency data will be updated during this run.

## Caveats

`by_majority_min_molecules_to_start` must be a multiple of 
[`options.block_size`]({{< relref "docs/reference/Options.md" >}}), and it must be 
smaller or equal to the product `options.block_size` * `options.max_blocks_in_flight`,
since you're going to have that many blocks 'in flight'.

If there are multiple candidates with exactly equal read counts (and hence frequencies),
that satisfy the threshold, the one with the smaller lexicographical sequence is being chosen.

ByMajority does not consider barcode labels at all during it's evaluation.

If there are fewer reads in your data set than `by_majority_min_molecules_to_start`, 
all of them are considered for the frequency estimation.


## By edit probability tie breaking
`ByEditProbability` assigns read sequences that are equidistant to multiple barcodes to 
most likely barcode, where the 'likelihood score' is defined as `p(edit) * (count + 1)`.
p(edit) is clamped to a lower bound of phred 33 (0.0005)).
Technically, the score that can exceed 1.0. 

The barcode with the highest score is picked. 

Clean barcode occurrence is estimated as above in the ByMajority section.

The likelihood score must exceed the `on_tie_threshold` parameter (which defaults to 0.975)
for any barcode to be reported.
