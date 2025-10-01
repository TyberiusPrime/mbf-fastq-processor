# ConvertPhred


```toml
[[step]]
    action = "ConvertPhred"
    from = "Illumina1.8|Sanger|Solexa"
    to = "Illumina1.8|Sanger|Solexa"

```

Convert quality scores between various encodings / meanings.

See https://en.wikipedia.org/wiki/Phred_quality_score

Will error if from == to.


## Corresponding options in other software 

- trimmomatic TOPHRED33


