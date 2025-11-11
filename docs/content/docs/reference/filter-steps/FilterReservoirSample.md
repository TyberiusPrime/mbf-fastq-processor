# FilterReservoirSample

```toml
[[step]]
    action = "FilterReservoirSample"
    n = 10_000
    seed = 59014
```

Filter for a fixed number of reads based on [reservoir sampling](https://en.wikipedia.org/wiki/Reservoir_sampling), that is all input reads have an equal probability of being selected.

This means we need to keep n reads in memory, and they get processed as one 
large block at the end by all downstream steps.

That means it's not the right tool if you want to sample to millions of reads,
but it is the right tool if you want to have lowish fixed number of reads.

The sampling process does not preserve the order of reads between input and output.

