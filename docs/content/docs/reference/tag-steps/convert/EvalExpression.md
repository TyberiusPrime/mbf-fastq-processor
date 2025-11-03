---
weight: 58
---

# EvalExpression

```toml
[[step]]
    action = "EvalExpression"
    label = "outtag"
    expression = "log(2, mytag + 1)" # log to base 2

```

Calculate a [fasteval](https://docs.rs/fasteval/latest/fasteval/) expression on your tags.

