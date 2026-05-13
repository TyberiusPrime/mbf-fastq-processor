status: open
# Slowdown during runtime

This slows down from 2.7 mio reads to like 1.6 
during it's 327 million read progression.
What's doing that?

```toml

[input]
read1 = "..."
read2 = "..."

[input.options]
use_rapidgzip = true

[options]
#threads = 2
max_blocks_in_flight = 300

[output]
report_html = false
prefix = "output"
format = 'none'

[barcodes.probes.from_file]
filename = "probeset.fasta"

[barcodes.cells.from_file]
filename = "737K-fixed-rna-profiling.txt.gz"

[[step]]
  action = 'Progress'
#
# [[step]]
#   action ='Head'
#   n = 30_000_000

[[step]]
action = "ExtractRegion"
start = 0
length = 16
segment = "read1"
out_label = "cb"
anchor = "left"

[[step]]
action = "ExtractRegion"
start = 16
length = 12
segment = "read1"
out_label = "umi"
anchor = "left"

[[step]]
action = "HammingCorrect"
max_hamming_distance = 1
barcodes = "cells"
in_label = "cb"
out_label = "cb_corrected"
on_no_match = "remove"
on_tie = "ByEditProbability"
on_tie_threshold = 0.975
on_tie_min_molecules_to_start = 1000000

[[step]]
action = "ExtractRegion"
start = 0
length = 50
source = "read2"
anchor = "left"
out_label = "probe_barcode"

[[step]]
action = "AssignByHalves"
in_label = "probe_barcode"
out_label = "assigned_probe_barcode"
barcodes = "probes"
name_split_char = "|"
#
[[step]]
action = "StoreSingleCellMatrix"
cell_tag = "cb_corrected"
gene_tag = "assigned_probe_barcode"
umi_tag = "umi"
cell_barcodes = "cells"
gene_barcodes = "probes"
umi_aggregation = 'Cluster'
#
# [[step]]
```

even happens with just
```toml

[input.options]
use_rapidgzip = true

[options]
#threads = 2
max_blocks_in_flight = 300

[output]
report_json = true
prefix = "tag_histogram"
format = 'none'

[barcodes.cells.from_file]
filename = "737K-fixed-rna-profiling.txt.gz"

[[step]]
  action = 'Progress'

[[step]]
action = "ExtractRegion"
start = 0
length = 16
segment = "read1"
out_label = "cb"
anchor = "left"

[[step]]
action = "HammingCorrect"
max_hamming_distance = 0
barcodes = "cells"
in_label = "cb"
out_label = "cb_corrected"
on_no_match = "remove"
on_tie = "remove"
#
[[step]]
action = "Report"
name = 'cb_corrected'
tag_histogram = ['cb_corrected']
```
