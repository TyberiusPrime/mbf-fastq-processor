# Matches 

```toml
[[step]]
  action = "Matches"
  out_label = 'of_interest'
  source = 'name:read1' # or tag:<tag_label> or <segment>
  accepted = ['alpha1', 'beta1', 'nosuch']
```

Set a bool tag whether the source matched one of the accepted values
(exactly, case sensitive).

Useful when you need to pick out some reads for debugging.

Often followed with by [FilterByTag]({{< relref "docs/redirects/FilterByTag.md" >}}).

