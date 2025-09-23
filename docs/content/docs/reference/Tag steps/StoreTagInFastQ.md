---
weight: 54
---


# StoreTagInFastQ 
```toml
## Store the content of a tag in a fastq file.
## Needs a 'location 'tag'.
## Can store other tags in the read name.
## quality scores are set to '~'.
# [[step]]
#    action = "StoreTagInFastQ"
#    label = "mytag" # ${output.prefix}.tag.mytag.fq${.suffix_from_format}
#    format = "Raw" # Raw, Gzip, Zstd
##   compression_level = 6 # (optional) compression level for gzip (0-9) or zstd (1-22)
						  # defaults: gzip=6, zstd=5
#    comment_tags = []# e.g. ["other_tag"] # see StoreTagInComment
#    comment_location_tags = ["mytag"] # (optional) tags to add location info for, defaults to [label]
#                                      # set to [] to disable location tracking
#    comment_insert_char = ' ' # (optional) char at which to insert comments
#    comment_separator = '|' # (optional) char to separate comments
#    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple

```

Store the sequence of a tag in a fastq file, 
with other tags optionally stored in the read name as comments.

Comments are key=value pairs, separated by `comment_separator` which defaults
to '|'. They get inserted at the first `comment_insert_char`, which defaults to
space.

By default, location information is added to read names in the format 
`{tag}_location=segment:start-end` (e.g., `barcode_location=read1:0-4`).
This can be controlled with `comment_location_tags`:
- Defaults to `[label]` - adds location for the main tag
- Set to `[]` to disable location tracking
- Set to specific tag names to track locations for those tags only

