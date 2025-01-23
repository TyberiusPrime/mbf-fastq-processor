### Rename


```toml
[[step]]
    action = "Rename"
    search = '(.)/([1/2])$'
    replacement = '$1 $2'
```

Apply a regular expression based renaming to the reads.

It is always applied to all available targets (read1, read2, index1, index2).

The example above fixes old school MGI reads for downstream processing, like
fastp's '--fix_mgi' option

You can use the full power of the [rust regex crate](https://docs.rs/regex/latest/regex/) here.


