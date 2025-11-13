# Tag / Label

A tag is a piece of molecule-derived data that one step in the pipeline produces,
and others may use (or output).

Tag columns are identified by 'labels' and have one of these types:
    - Location+Sequence (which may loose their location data)
    - Sequence-only
    - numeric
    - boolean

For example  [ExtractIUPAC]({{< relref "docs/reference/tag-steps/extract/ExtractIUPAC.md" >}})
 produces a 'location' tag.

That location tag then allows filtering for it's presence using 
[FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}),
cutting the segment at it's location using [TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}) or replacing it's sequence with [LowercaseTag]({{< relref "docs/reference/modification-steps/LowercaseTag.md" >}}).

Within the [steps]({{< relref "docs/concepts/step.md" >}}), tags are consistently referered to as `in_label(s)`, `out_label`, depending on whether the step consumes or produces the tag.
