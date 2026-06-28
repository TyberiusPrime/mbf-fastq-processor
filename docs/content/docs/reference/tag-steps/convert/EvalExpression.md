---
weight: 58
---

# EvalExpression

```toml
[[step]]
    action = "EvalExpression"
    out_label = "outtag"
    expression = "log(2, mytag + 1)" # log to base 2
    result_type = "Numeric" # Optional: 'Numeric' or 'Boolean'.

```

Calculate a [fasteval](https://docs.rs/fasteval/latest/fasteval/) expression on your tags, 
which you can then pass to .[FilterByTag]({{< relref "docs/reference/filter-steps/FilterByTag.md" >}}).

You can use any tags previously defined on the molecule as variables in the expression.

Location/String tags get converted into numeric-Booleans (false=0, true=1) based on whether
they are present (=not missing). 

That means you can for example chain this after 
[HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}})
with `on_no_match` = 'remove'.

Additional, there's a series of 
[virtual tags]({{< relref "docs/concepts/tag.md" >}}#virtual-tags) available

* `len_<segment-name>` - the length of the specified segment (e.g. `len_read1`).
* `len_<tag-label>` - the length of the specified tag (e.g. `len_mytag`). For location tags, 
  this is the length of the underlying matched regions (which may change / be lost when reads are truncated - eval before truncation if necessary). For string tags (= [ExtractRegex]({{< relref "docs/reference/tag-steps/extract/ExtractRegex.md" >}}) with `source=name:...`) this is the length of the *replaced* string.
* `read_no` - the running number of the read (starting with 0)


## Language

Besides the regular arithmetic operators (+, -, *, /, %, ^) this supports
boolean operations (>, <, >=, <=, ==, !=) and the following functions are supported:
log(base, val), e(), pi(), int(), ceil(), floor(), round(), abs(), sign(),
min(a,b,...), max(a,b,...) sin(radians), cos(radians), tan(radians),
sinh(radians), cosh(radians), tanh(radians), 

Use any defined tag by name.

Location/string tags are converted to Booleans by their presence.

You can also use `len_<segment>` or `len_<tagname>` to access the length of tags and segments.

There is no not operator, but you can use `== 0` for 'is false' and `!= 0` for 'is true'.

## Result type
By default, the result type is deduced from the expression. 
You can overwrite this by forcing `result_type` to a specific 
[column type]({{< relref "docs/concepts/tag.md" >}}#tag-types), `Numeric` or `Boolean`
