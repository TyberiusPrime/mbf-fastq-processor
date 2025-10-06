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

{{<mynav>}}

Explore the [tag generation section]({{< relref "docs/reference/tag-steps/generation/_index.md" >}}) for steps that create new labels, and the [tag usage section]({{< relref "docs/reference/tag-steps/using/_index.md" >}}) for helpers that consume or export those labels.

There are further tag using steps in the [modification steps]({{< relref "docs/reference/modification-steps/_index.md" >}}) and [filter steps]({{< relref "docs/reference/filter-steps/_index.md" >}}) sections.

