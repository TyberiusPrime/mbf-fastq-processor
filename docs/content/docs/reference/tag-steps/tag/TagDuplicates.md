### FilterDuplicates

```toml
[[step]]
    action = "TagDuplicates"
    false_positive_rate = 0.00001 #
            # the false positive rate of the filter.
            # 0..1
    seed = 59 # required!
    source = "All" # Any input segment, 'All', 'tag:<tag-name>' or 'name:<segment>'
    # split_character = "/" # required (and accepted only iff using name:<segment>
    label = "dups"

[[step]]
    action = "FilterByBoolTag"
    label = "dups"
    keep_or_remove = "Remove" # Keep|Remove
```

Tag duplicates (2nd onwards) from the stream using a [Cuckoo filter](https://en.wikipedia.org/wiki/Cuckoo_filter).

That's a probabilistic data structure, accordingly there's a false positive rate,
and a tunable memory requirement.

Needs a seed for the random number generator, and a source
to know which values to consider for deduplication (filters the complete molecule, like
all other filters of course). Sources can be a segment name, `All` (to combine every
segment in the molecule), another tag via `tag:<tag-name>`, or a read name prefix using
`name:<segment>`. The `name:` form requires `split_character` to define where to split the
read name, matching the semantics of `readname_end_char` elsewhere in the tool. When
referencing an existing tag, every tag value is converted into a binary representation
before entering the filter, allowing numeric, boolean, and sequence tags to participate.
Because these prefixes are reserved, the output `label` must not begin with `name:` or
`tag:`.

The lower you set the false positive rate, the higher your memory requirements will be.
0.00001 might be a good place to start. 

If you set the false positive rate to 0.0, a HashSet will be used instead,
which will produce exact results, albeit at the expense of keeping a copy of *all* reads in memory! 

Please note our [remarks about cuckoo filters]({{< relref "docs/faq/_index.md" >}}#cuckoo-filtering).

If the source is a tag, missing values (e.g. not-matching regex results) will always be treated
as unique. Only Location/String tags are supported for TagDuplicates.
