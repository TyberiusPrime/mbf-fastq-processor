# Tag

A tag is a piece of molecule derived data that one step in the pipeline produces,
and others may use (or output).

For example  [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}})
 produces a 'location' tag.

That location tag then allows filtering for it's presence using 
[FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}),
cutting the segment at it's location using [TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}) or replacing it's sequence with [LowercaseTag]({{< relref "docs/reference/modification-steps/LowercaseTag.md" >}}).
