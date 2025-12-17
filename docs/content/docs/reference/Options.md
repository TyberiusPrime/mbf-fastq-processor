---
weight: 100
---

# Options

There is a small set of runtime knobs exposed under `[options]`. 

Most workflows can rely on the defaults.

```toml
[options]
    thread_count = 10
    max_blocks_in_flight = 100
    block_size = 10000
    buffer_size = 102400
    accept_duplicate_files = false
    spot_check_read_pairing = true
```

| Key                      | Default | Description |
|--------------------------|---------|-------------|
| `thread_count`           | (auto)    | Worker threads for transformations. See [threading]({{< relref "docs/reference/threading.md" >}}). |
| `max_blocks_in_flight`    | `100`    | How many blocks may be concurrently being processed. Lowering this limits RAM usage. |
| `block_size`             | `10000` | Number of fragments pulled per batch. Increase for very large runs when IO is abundant; decrease to reduce peak memory use. |
| `buffer_size`            | `102400` | Initial bytes reserved per block. The allocator grows buffers on demand, so tuning is rarely necessary. |
| `accept_duplicate_files` | `false` | Permit the same path to appear multiple times across segments. Useful for fixtures or synthetic tests; keep disabled to catch accidental copy/paste errors. |
| `spot_check_read_pairing` | `true` | Sample every 1000th fragment to ensure paired reads still share a name prefix; disable when names are intentionally divergent or rely on `ValidateName` to customise the separator. |

Changing these knobs can affect memory pressure and concurrency behavior. 

Measure before and after if you deviate from defaults.
