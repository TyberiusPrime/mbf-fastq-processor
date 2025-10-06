
# Progress

Report progress to stdout (default) or a .progress log file,
if output_infix is set. (filename is {output_prefix}_{infix}.progress)

```toml
[[step]]
   action = "Progress"
   n = 100_000
   output_infix = "progress" # optional
```

Every `n` reads, report on total progress, total reads per second. 
At the end, report final runtime and reads/second.

