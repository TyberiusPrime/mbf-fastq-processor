# Head

```toml
[[step]]
    action = "Head"
    n = 1000 # positive integer, number of reads to keep
```

Output just the first n molecules.

### Demultiplex interaction

If present after a demultiplex step, includes n molecules in each stream.
