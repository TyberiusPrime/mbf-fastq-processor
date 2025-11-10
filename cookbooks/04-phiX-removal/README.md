# Cookbook 04: phiX removal

## Use Case

You have Illumina default spike-in PhiX in your sequences, and would like
to remove those reads.


## What This Pipeline Does

1. Quantifies how many kmers of your reads map to the PhiX genome
2. Prepares a table so you can plot a histogram.
2. Filters those that have too many phiX kmers.



[TODO]
