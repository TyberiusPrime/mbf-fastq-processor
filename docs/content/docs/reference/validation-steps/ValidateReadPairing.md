# ValidateReadPairing

Confirms for every `sample_stride`th read 'pair' that the names are 
identical but for one letter.

```toml
[[step]]
    action = "ValidateReadPairing"
    sample_stride = 1000 # Check every nth fragment, default 1000. Must be > 0. Starts with first read
```

Ensures
 - read names between segments have the same length
 - read names between segments have a hamming distance of at most one.


Note that this validation requires at least two input segments.

(See also: [`ValidateName`]({{< relref "docs/reference/validation-steps/ValidateName.md" >}}), 
which validates after truncating on a character occurance).

## Automatic ValidateReadPairing

This step is injected automatically after your transformations when 

 - more than one input segment is defined
 - and [`options.spot_check_read_pairing`]({{< relref "docs/reference/options/_index.md" >}}) is set to `true` (the default)
 - and no explicit `ValidateReadPairing` or `ValidateName` step is present. 

 See the [input section]({{< relref "docs/reference/input-section.md" >}}) for details.