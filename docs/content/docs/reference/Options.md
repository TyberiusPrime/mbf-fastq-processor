---
weight: 100
---

# Options

There is a small set of runtime knobs exposed under `[options]`. Most workflows can rely on the defaults.

```toml
[options]
    thread_count = -1
    block_size = 10000
    buffer_size = 102400
    accept_duplicate_files = false
    spot_check_read_pairing = true
```

| Key                      | Default | Description |
|--------------------------|---------|-------------|
| `thread_count`           | `-1`    | Worker threads for transformations. `-1` autotunes per CPU; most runtime is still dominated by decompression threads, so gains are modest. |
| `block_size`             | `10000` | Number of fragments pulled per batch. Increase for very large runs when IO is abundant; decrease to reduce peak memory use. |
| `buffer_size`            | `102400` | Initial bytes reserved per block. The allocator grows buffers on demand, so tuning is rarely necessary. |
| `accept_duplicate_files` | `false` | Permit the same path to appear multiple times across segments. Useful for fixtures or synthetic tests; keep disabled to catch accidental copy/paste errors. |
| `spot_check_read_pairing` | `true` | Sample every 1000th fragment to ensure paired reads still share a name prefix; disable when names are intentionally divergent or rely on `ValidateName` to customise the separator. |

Changing these knobs can affect memory pressure and concurrency behaviour. Measure before and after if you deviate from defaults.
