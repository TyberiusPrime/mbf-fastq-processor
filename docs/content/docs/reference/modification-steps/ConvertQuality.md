# ConvertQuality


```toml
[[step]]
    action = "ConvertQuality"
    from = "Illumina1.8"# Illumin1.8|Illumina1.3|Sanger|Solexa"
    to = "Solexa" # same range as from. Illumina1.8 is an alias for Sanger

```

Convert quality scores between various encodings / meanings.

See https://en.wikipedia.org/wiki/Phred_quality_score

Will error if from == to.

This step introduces a  [ValidateQuality]({{< relref "docs/reference/validation-steps/ValidateQuality.md" >}}) step automatically before it.


## Corresponding options in other software 

- trimmomatic TOPHRED33


