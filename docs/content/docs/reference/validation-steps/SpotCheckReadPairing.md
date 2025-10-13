# SpotCheckReadPairing

Confirms for every `sample_stride`th read 'pair' that the names share the same prefix
(part before 'readname_end_char').

```toml
[[step]]
    action = "SpotCheckReadPairing"
    sample_stride = 1000 # Every nth fragment, default 1000. Must be > 0. 
    readname_end_char = '/' # u8/byte-char, Defaults to no character, meaning the entire name is compared.
```

When no separator character is provided the entire name must match exactly across all segments.

Otherwise the read names are truncated after the first readname_end_char, and the prefixes must match exactly. For example, use readname_end_char = "_" for typical Illumina _1/_2 suffixes.

Note that this validation requires at least two input segments so there is a
name to compare against, mbf-fastq-processor will return an error otherwise.

(Closely related in concept to [`ValidateName`]({{< relref "docs/reference/validation-steps/ValidateName.md" >}})).

## Automatic SpotCheckReadPairing

This step is injected automatically after your transformations when 

 - more than one input segment is defined
 - and [`options.spot_check_read_pairing`]({{< relref "docs/reference/options/_index.md" >}}) is set to `true` (the default)
 - and and no explicit `SpotCheckReadPairing` or `ValidateName` step is present. 

 See the [input section]({{< relref "docs/reference/input-section.md" >}}) for details.


