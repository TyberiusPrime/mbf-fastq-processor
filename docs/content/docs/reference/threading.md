
---
weight: 105
not-a-transformation: true

---
# Multithreading considerations

Mbf-fastq-processor is inherently multi-threaded, and strives
to make full use of your machine's cores.

Usually, you should not need to influence the thread counts.

If you want to limit CPU usage, we suggest either systemd based resource control
or tools like [cpulimit] (https://github.com/opsengine/cpulimit) instead of coarsely
changing thread counts.


## Threading architecture

Mbf-fastq-processor runs the following thread stack for a (non-interleaved configuration):

```
[decompression / reader threads]
    ↓
[parsers]
    ↓
combining thread
    ↓
[workpool handling steps]
    ↓
output thread
```

### Decompression / reader threads
The number of decompression / reading threads is 
(in order of precedence)

- 1 per segment if the input is not compressed
- 1 per segment if the input is in BAM format (BAM multi-threading currently is slower)
- the value of `input.options.threads_per_segment` if set.
- the minimum of (half your cores / segment count) and 5.

Our benchmarking suggests that speed gains after 5 threads are 
essentially non existent. For interleaved configuration,
segment count is 1. 

Note that if this ends up being one, rapidgzip is disabled, 
since at one core it's slower than the single core gzip library we're using.


### Parser threads
We have one thread per segment (exactly one thread for interleaved input).

### Combining therad
Exactly one (or zero if interleaved input). Very light weight computationally. 

### Work pool

The treatment of your reads by steps is done in a pool of threads which
automatically distribute the load between steps that can run in parallel and
those that must see every read in sequence.

The number of work pool threads is
(in order of precedence)

- the value of `options.thread_count`.
- the maximum of (cores / 2) and (cores - threads used by decompression / reader threads)

Note that while over subscription of CPU cores is not ideal,
it's also not going to slow things down much.

### output thread
Exactly one.


## Reports
The TOML representation in JSON reports reflects the actually used cores.





