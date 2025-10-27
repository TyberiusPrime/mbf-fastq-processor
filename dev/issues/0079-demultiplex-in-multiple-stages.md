status: open
# Demultiplex in multiple stages

We're currently hard limiting to one demultiplex stage per workflow.
I can envision problems where we'd want to demultiplex multiple times,
for example once on barcode and once on reads where we could / could not 
merge the segments. or perhaps an kmer score.

Mostly the work would be to assign bit ranges, and or the 
demultiplex scores together.
