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

And then you get to use one of the following to make use of it :

 * [TrimTag](../modification-steps/trimtag) to trim the read at the tag
 * [LowercaseTag](../modification-steps/lowercasetag) to trim the read at the tag
 * [AddTagSequenceToName](../modification-steps/addtagsequencetoname) to add the tag sequence to the read name.
 * [FilterTag](../filter-steps/filtertag) to keep or remove reads matching the tag




