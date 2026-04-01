---
weight: 54
---

# StoreTagInFastQ

```toml
# Store the content of a tag in a fastq file.
# Needs a 'location 'tag'.
# Can store other tags in the read name.
# quality scores are set to '~'.
# With demultiplexing: creates separate files per barcode
 [[step]]
    action = "StoreTagInFastQ"
    in_label = "mytag" # tag to store. File name is derived with this as infix
    format = "Fastq" # FASTQ / FASTA / BAm
    compression = "gz" # or "zstd" | "none" # (optional) compression format, not if format == BAM
    compression_level = 6 # (optional) compression level for gzip (0-9) or zstd (1-22)
   					  # defaults: gzip=6, zstd=5
    comment_tags = []# e.g. ["other_tag"] # see StoreTagInComment
    comment_insert_char = ' ' # (optional) char at which to insert comments
    comment_separator = '|' # (optional) char to separate comments
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple

```

Store the sequence of a tag in a fastq file,
with other tags (and virtual tags) optionally stored in the read name as comments.

The output filename is constructed as `{prefix}_tag.{label}.fq` (or with custom separator if configured).


Comments are key=value pairs, separated by `comment_separator` which defaults
to '|'. They get inserted at the first `comment_insert_char`, which defaults to
space.


### Interaction with demultiplex

When demultiplexing is used, separate FASTQ files are created for each barcode: `{prefix}_{barcode}.tag.{label}.fq`
