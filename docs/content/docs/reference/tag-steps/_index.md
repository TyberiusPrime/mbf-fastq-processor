---
weight: 11
bookCollapseSection: true
---
## Tag Extraction

Finding 'things' in reads, and then using that information is called 'tagging'.

Extraction and downsteam steps are tied together with arbitrary 'labels'.

This allows you to efficiently perform multiple actions with one search, for example 
trim adapter tags and keep only reads that contain the adapter.

mbf-fastq-processor errors early if an `Extract*` step introduces a label that is never used or removed by later transformations. 

You first use one of the following steps to extract read information:
 
{{<mynav>}}

### Tag generation steps

- [ExtractAnchor]({{< relref "docs/reference/tag-steps/ExtractAnchor.md" >}}) locates a sequence relative to a fixed anchor.
- [ExtractGCContent]({{< relref "docs/reference/tag-steps/ExtractGCContent.md" >}}) stores the GC ratio for later filtering.
- [ExtractIUPAC]({{< relref "docs/reference/tag-steps/ExtractIUPAC.md" >}}) matches an IUPAC motif and records the hit.
- [ExtractIUPACSuffix]({{< relref "docs/reference/tag-steps/ExtractIUPACSuffix.md" >}}) keeps the sequence following an IUPAC pattern.
- [ExtractLength]({{< relref "docs/reference/tag-steps/ExtractLength.md" >}}) captures the length of a segment.
- [ExtractLowComplexity]({{< relref "docs/reference/tag-steps/ExtractLowComplexity.md" >}}) tags reads with low-complexity regions.
- [ExtractLowQualityEnd]({{< relref "docs/reference/tag-steps/ExtractLowQualityEnd.md" >}}) finds low-quality trailing bases.
- [ExtractLowQualityStart]({{< relref "docs/reference/tag-steps/ExtractLowQualityStart.md" >}}) finds low-quality leading bases.
- [ExtractMeanQuality]({{< relref "docs/reference/tag-steps/ExtractMeanQuality.md" >}}) keeps the average phred score.
- [ExtractNCount]({{< relref "docs/reference/tag-steps/ExtractNCount.md" >}}) counts ambiguous bases.
- [ExtractPolyTail]({{< relref "docs/reference/tag-steps/ExtractPolyTail.md" >}}) records homopolymer runs.
- [ExtractQualifiedBases]({{< relref "docs/reference/tag-steps/ExtractQualifiedBases.md" >}}) counts bases above a quality threshold.
- [ExtractRegex]({{< relref "docs/reference/tag-steps/ExtractRegex.md" >}}) tags arbitrary regular-expression matches.
- [ExtractRegion]({{< relref "docs/reference/tag-steps/ExtractRegion.md" >}}) extracts a single interval.
- [ExtractRegions]({{< relref "docs/reference/tag-steps/ExtractRegions.md" >}}) extracts multiple intervals.
- [ExtractRegionsOfLowQuality]({{< relref "docs/reference/tag-steps/ExtractRegionsOfLowQuality.md" >}}) records stretches below a quality cutoff.
- [TagDuplicates]({{< relref "docs/reference/tag-steps/TagDuplicates.md" >}}) labels likely duplicates for follow-up filters.
- [TagOtherFileByName]({{< relref "docs/reference/tag-steps/TagOtherFileByName.md" >}}) marks matches against another file's read names.
- [TagOtherFileBySequence]({{< relref "docs/reference/tag-steps/TagOtherFileBySequence.md" >}}) marks matches against another file's sequences.

### Tag using steps

- [HammingCorrect]({{< relref "docs/reference/tag-steps/HammingCorrect.md" >}}) normalises extracted tags against a barcode list.
- [RemoveTag]({{< relref "docs/reference/tag-steps/RemoveTag.md" >}}) drops labels that are no longer needed.
- [StoreTagInComment]({{< relref "docs/reference/tag-steps/StoreTagInComment.md" >}}) copies a tag into the FASTQ comment.
- [StoreTagInFastQ]({{< relref "docs/reference/tag-steps/StoreTagInFastQ.md" >}}) duplicates tags into additional FASTQ outputs.
- [StoreTagInSequence]({{< relref "docs/reference/tag-steps/StoreTagInSequence.md" >}}) writes a tag back into the read sequence.
- [StoreTagLocationInComment]({{< relref "docs/reference/tag-steps/StoreTagLocationInComment.md" >}}) reports where a tag was found.

And then you get to use one of the following modification steps to act on tags directly:

- [TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}) trims the read where the tag occurs.
- [LowercaseTag]({{< relref "docs/reference/modification-steps/LowercaseTag.md" >}}) highlights tagged regions by lower-casing them.
- [ExtractToName]({{< relref "docs/reference/modification-steps/ExtractToName.md" >}}) appends tag information to the read name.
- [FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}) keeps or removes reads based on a tag value.
