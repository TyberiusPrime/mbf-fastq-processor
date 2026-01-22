---
weight: 50
---

# ExtractRegionsOfLowQuality

Extract regions where bases have quality scores below threshold, with a minimum length requirement.

```toml
[[step]]
    action = "ExtractRegionsOfLowQuality"
    segment = "read1" # Any of your input segments
    min_quality = 60  # Quality threshold (Phred+33)
    min_length = 5    # Minimum region length (bp)
    out_label = "low_quality_regions"
```

This transformation scans through quality scores of the specified segment and identifies contiguous regions where quality scores are below the specified threshold. Each low-quality region that meets the minimum length requirement becomes a tagged region with location information (start position and length).

## Parameters

- `segment`: Which read to analyze for low-quality regions
- `min_quality`: Quality score threshold using Phred+33 encoding. See [Phred quality score](https://en.wikipedia.org/wiki/Phred_quality_score#Symbols) for ASCII character mapping
- `min_length`: Minimum length (in bases) for a region to be extracted. Must be >= 1
- `out_label`: Tag name to store the extracted regions

## Example

With `min_quality = 60` (ASCII '<') and `min_length = 5`, any contiguous stretches of 5+ bases with quality scores below '<' will be identified as low-quality regions. This is useful for masking or filtering poor-quality sequences.

## Notes
Note that one read may have multiple low-quality regions. 
[TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}})
will cut at the outmost one of them.
