status: open
# Compression Investigation

- **Issue**: Slow decompression performance on ERR13885883
  - Current: ~44.7s (43.07s without output)
  - Recompressed gz: 44.7s (42.39s)
  - zstd: 43.53s (24s)
- **Investigation**: Compare with fastp performance
- **Potential Solution**: Explore `gzp` crate for parallel Gzip writing
