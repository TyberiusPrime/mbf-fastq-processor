
### Inspect

Dump a few reads to a file for inspection at this point in the graph.

```toml
[[step]]
    action = "Inspect"
    n  = 1000 # how many reads
    infix = "inspect_at_point" # output is output_prefix_infix.fq
    target = "Read1" # Read1|Read2|Index1|Index2
```


