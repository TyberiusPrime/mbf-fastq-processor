status: closed
# Tag performance

We're currently have tags as Vec<TagValue>
and that means essentially every tag is a
a) at least 3 pointers,
b) we need to case on every single tag.

When really we'd be good with a 
```
enum TagValues {
    Location: Option<Vec<Hits>>
    ... 
}
```

And freeing the tags is slow, 
maybe an arena allocation would help.


