---
weight: 110
---

# Benchmark


For profiling and benchmarking (individual) steps, 
mbf-fastq-processor has a special benchmark mode.

This mode focuses on benchmarking the steps,
and avoids (most) input and output runtime.

Enable it by adding this TOML section. 
The output section becomes optional (and ignored)
when benchmarking is enabled.

```toml
[benchmark]
    enable = true
    molecule_count = 1_000_000
```

Benchmark mode:

- Disables (regular) output
- runs in a temp directory,
- repeats the first molecule 'block' of [`Options.block_size`]({{< relref "docs/reference/Options.md" >}}) reads
  until `molecule_count` has been exceeded.


The last point means that we will spent very little time in 
reading & decompression (without rapidgzip / parallel BAM processing the largest
runtime parts), and focus on the steps. The drawback here is that your pipeline
sees the same reads over and over, which of course will lead to a different
'hit' profile for set based tests such as duplication counting, 
[TagOtherFileByName]({{< relref "docs/reference/tag-steps/tag/TagOtherFileByName.md" >}}),  
and [Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}})  

