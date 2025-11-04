---
weight: 11
bookCollapseSection: true
---

## Tag Extraction

Finding 'things' in reads, and then using that information is called 'tagging'.

Extraction and downsteam steps are tied together with arbitrary 'labels'.

This allows you to efficiently perform multiple actions with one search, for example
trim adapter tags and keep only reads that contain the adapter.

mbf-fastq-processor errors early if a step introduces a label that is never used or removed by later transformations.


Tags generating steps are split into three sections, depending on their output:

- [calc]({{< relref "docs/reference/tag-steps/calc/_index.md" >}}) for steps that create numeric labels,
- [convert]({{< relref "docs/reference/tag-steps/convert/_index.md" >}}) for steps that reshape existing tags into new ones,
- [extract]({{< relref "docs/reference/tag-steps/extract/_index.md" >}}) for steps that define 'regions' within your reads,
- [tag]({{< relref "docs/reference/tag-steps/tag/_index.md" >}}) section for steps that add boolean values to your reads.

Finally, see
and the [tag usage section]({{< relref "docs/reference/tag-steps/using/_index.md" >}}) for helpers that consume or export those labels.

There are further tag using steps in the [modification steps]({{< relref "docs/reference/modification-steps/_index.md" >}}) and [filter steps]({{< relref "docs/reference/filter-steps/_index.md" >}}) sections.


## Rules for Tag labels

Tag labels must conform to '[a-zA-Z_][a-zA-Z0-9_]*$' and are case sensitive (this is so they can be used in [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}).

Tag labels may not be 'ReadName' (first column in [StoreTagsInTable]({{< relref "docs/reference/tag-steps/using/StoreTagsInTable.md" >}})), nor may they start with 'len_' (used by [EvalExpression]({{< relref "docs/reference/tag-steps/convert/EvalExpression.md" >}}) as virtual tags.
