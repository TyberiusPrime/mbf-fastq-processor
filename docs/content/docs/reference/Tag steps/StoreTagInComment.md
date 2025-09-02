---
weight: 50
---

# StoreTagInComment

Store currently present tags as comments on read names.

```toml
[[step]]
    action = "StoreTagInComment"
    label = "mytag" # if set, only store this tag
    target = "Read1" # Read1|Read2|Index1|Index2|All
    comment_insert_char = " " # (optional) char at which to insert comments
    comment_separator = "|" # (optional) char to separate comments
    region_separator = "_" # (optional) char to separate regions in a tag, if it has multiple
```

Comments are key=value pairs, separated by `comment_separator` which defaults to '|'. They get inserted at the first `comment_insert_char`, which defaults to space.

For example, a read name like:
```
@ERR12828869.501 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
```
becomes:
```
@ERR12828869.501|key=value|key2=value2 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
```

This way, your added tags will survive STAR alignment (STAR always cuts at the first space).