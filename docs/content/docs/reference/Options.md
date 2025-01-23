---
weight: 100
---

# Options

There is a small number of nuisance parameters that can be configured.

You should not have to touch these during normal operations.

They are just documented here to be thorough.

```toml
[options]
    thread_count = -1 # (optional)
    block_size = 10000 # (optional)
    buffer_size = 102400 # (optional)
    accept_duplicate_files = false #(optional)
```

`thread_count` decides how many in-parallel processing threads get allocated.
Since most of the wall clock time is actually in decompressing the FastQ,
which happens in 1..4 threads (depending if you have read2, etc),
this has very little effect on the actual run time.

`block_size` is the number of reads that are processed in one go.

`buffer_size` is the initial size of the buffer which will receive one block
worth of reads. It also has a minor effect on the actual runtime.

`accept_duplicate_files` - for testing, it is often nice to use the same
file in multiple positions. During normal operations, this is rejected
to prevent accidental copy/paste errors.
