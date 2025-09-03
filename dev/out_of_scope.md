# Out of scope

Just a quick list of things we are not going to support.

## Fast5

https://medium.com/@shiansu/a-look-at-the-nanopore-fast5-format-f711999e2ff6
Oxford Nanopore squiggle data.
Apparently no formal spec.


## kallisto BUS format 
    - a brief barcode/umi format for single cell RNA-seq
    - needs an 'equivalance class' - i.e. at least pseudo alignment
    - weird length restrictions on barcodes and umis (1(!)-32), 
      but stores the length in an uint32...


