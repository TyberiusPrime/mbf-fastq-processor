### Rename


```toml
[[step]]
    action = "Rename"
    search = '(.)/([1/2])$'
    replacement = '$1 $2'
```

Apply a regular expression based renaming to the reads.

It is always applied to all available segments (read1, read2, index1, index2).

The example above fixes old school MGI reads for downstream processing, like
fastp's '--fix_mgi' option

You can use the full power of the [rust regex crate](https://docs.rs/regex/latest/regex/) here.

#### Read index placeholder

After the regex replacement runs, the special literal `{{READ_INDEX}}` is expanded to
the running 1-based index of each logical read. When multiple segments are present
(for example `read1`/`read2` pairs), every segment for the same read receives the
same index so pairs stay aligned. This makes it easy to re-sequence identifiers:

```toml
[[step]]
    action = "Rename"
    search = '^(.*)/(\d)$'
    replacement = 'LIB-{{READ_INDEX}}/$2'
```

The placeholder can appear multiple times within the replacement and is safe to
mix with regex capture groups.

As an optimization, if no '{{READ_INDEX}}' is present in `replacement`,
the step can run multi core, and omits the replacement. This means that the 
placeholder must be present before (and after) the regex replacement.

