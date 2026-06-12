---
weight: 50
---

# StoreTagBackInSequence


```toml
[[step]]
    action = "StoreTagBackInSequence"
    in_label = "mytag"
    on_lost = "Complain" # Ignore|Complain
```

This transformation stores a tag's value back into the sequence, replacing
the original sequence at that location.

This only works if either the tag's length is unchanged, or it's a single region tag,
since for a multi-region tag it would be undefined where to add or remove bases.

If reads have been manipulated in away that this tag's bases are (even partially lost), 
the on_lost handling comes to pass. By default, a run-time error is raised.
Set it to 'Ignore' to instead leave the reads unchanged.



This is useful for example after you have applied 
[Lowercase]({{< relref "docs/redirects/Lowercase.md" >}}) to a tag, 
to lowercase precise regions in a read.

Compare with
[StoreTagInSequence]({{< relref "docs/redirects/StoreTagInSequence.md" >}}) 
which stores one tag's values in another tag's location.

