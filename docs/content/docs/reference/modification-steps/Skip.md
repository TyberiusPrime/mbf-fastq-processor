# Skip

```toml
[[step]]
    action = "Skip"
    n = 1000 # positive integer, number of reads to skip
```

Skips the first n molecules.

### Demultiplex interaction

If present after a demultiplex step, skips the first n molecules in that stream
