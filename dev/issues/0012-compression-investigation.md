status: open
# Parallel decompression

- **Issue**: Slow decompression performance on ERR13885883
  - Current: ~44.7s (43.07s without output)
  - Recompressed gz: 44.7s (42.39s)
  - zstd: 43.53s (24s)
- **Investigation**: Compare with fastp performance
- **Potential Solution**: Explore `gzp` crate for parallel Gzip writing

It's a 7 gigs compressed fastq, 17.32 gigs uncompressed.

Benchmarks (cat) at about 75s.  Fastp 0.24.0 is slower.


https://github.com/mxmlnkn/rapidgzip
suggests in it's graph that gzip (on their benchmarking platform)
runs about 750 MB/s. That'd be 23 seconds for 17.3 gigs.
But of course that's also highly data dependend.


https://arxiv.org/pdf/2308.08955

Rapidgzip: Parallel Decompression and Seeking in Gzip Files
Using Cache Prefetching

It even has a fastq file as an example where it gains
a lot of power by going multicore.

They do offer a c++ library, which I guess 
we could somehow repurpose, but it won't be trivial.

RabbitTrim seems to use it though,
still calling in 'pragzip'. Might be useful to understand how to use it.

