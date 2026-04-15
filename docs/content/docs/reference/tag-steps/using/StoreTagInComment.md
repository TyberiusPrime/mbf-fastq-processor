---
weight: 50
---

# StoreTagInComment

Store currently present tags as comments on read names.

```toml
[[step]]
    action = "StoreTagInComment"
    in_labels = ["mytag"] # Store these tags. Not optional. May be a single string as well. Alias 'in_label'
    segment = "read1" # Any of your input segments, or 'All'
    comment_insert_char = " " # (optional) char at which to insert comments
    comment_separator = "|" # (optional) char to separate comments
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple
```

Comments are key=value pairs, separated by `comment_separator` which defaults to '|'. 
They get inserted before the first `comment_insert_char`, which defaults to 
[`input.options.read_comment_char`]({{< relref "docs/reference/input-section.md#input-options" >}}).

For example, a read name like:
```
@ERR12828869.501 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
```
becomes:
```
@ERR12828869.501|key=value|key2=value2 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
```

This way, your added tags will survive STAR alignment (STAR always cuts at the first space).

Numeric values are output at most in scientific notation with 4 significant digits.


