status: done
# deduplicate by tag? is this useful

What happens with reads that don't have a value for the tag?
some tools offer dedup-by-prefix, such az czid

Implementation wise, we'd extend tag with a magic value,
that means 'SegmentOrAll|tag|name'
