---
weight: 50
---

# ExtractRegionsOfLowQuality

Extract regions where bases have quality scores below threshold.

```toml
[[step]]
    action = "ExtractRegionsOfLowQuality"
    target = "Read1" # Read1|Read2|Index1|Index2
    min_quality = 60  # Quality threshold (Phred+33)
    label = "low_quality_regions"
```

This transformation scans through quality scores of the specified target and identifies contiguous regions where quality scores are below the specified threshold. Each low-quality region becomes a tagged region with location information (start position and length).

## Parameters

- `target`: Which read to analyze for low-quality regions
- `min_quality`: Quality score threshold using Phred+33 encoding. See [Phred quality score](https://en.wikipedia.org/wiki/Phred_quality_score#Symbols) for ASCII character mapping
- `label`: Tag name to store the extracted regions

## Example

With `min_quality = 60` (ASCII '<'), any bases with quality scores below '<' will be identified as low-quality regions. This is useful for masking or filtering poor-quality sequences.