---
weight: 58
---

# EvalExpression

```toml
[[step]]
    action = "EvalExpression"
    out_label = "outtag"
    expression = "log(2, mytag + 1)" # log to base 2
    result_type = "numeric" # or bool.

```

Calculate a [fasteval](https://docs.rs/fasteval/latest/fasteval/) expression on your tags, 
which you can then pass to .[FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}).

You can use any tags previously defined on the molecule as variables in the expression.

Additional, there's a series of virtual tags available:

* `len_<segment-name>` - the length of the specified segment (e.g. `len_read1`).
* `len_<tag-label>` - the length of the specified tag (e.g. `len_mytag`). For location tags, 
  this is the length of the underlying matched regions (which may change / be lost when reads are truncated - eval before truncation if necessary). For string tags (= [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}}) with `source=name:...`) this is the length of the *replaced* string.
* `read_no` - the running number of the read (starting with 0)


## Language

Besides the regular arithmetic operators (+, -, *, /, %, ^)
this supports log(base, val), e(), pi(), int(), ceil(), floor(), round(), abs(), sign(), min(a,b,...), max(a,b,...)
sin(radians), cos(radians), tan(radians), sinh(radians), cosh(radians), tanh(radians), 
Use any defined tag by name. Location/string tags are converted to booleans by their presence.

You can also use `len_<segment>` or `len_<tagname>` to access the length of tags and segments.
