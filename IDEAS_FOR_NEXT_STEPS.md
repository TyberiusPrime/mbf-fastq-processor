# Ideas for Next Development Steps

Generated: 2025-11-18

This document outlines potential areas for improvement and future development for mbf-fastq-processor, organized by category and priority.

## 🎯 High Priority Items

### 1. Performance & Profiling
- **Profile report generation performance** (Issue #102)
  - Benchmarking shows regression when reporting on ERR9969633
  - Need to identify and fix performance bottlenecks
  - Consider using the new timing infrastructure to identify slow steps

### 2. Quality Metrics & Reporting
- **Insert size histogram analysis** (Issue #4)
  - fastp-style overlapping reads statistics
  - Already have merging capability, just need statistics collection
  - Provides library preparation quality metrics

- **Duplication analysis** (Issue #16)
  - Duplication distribution reporting (frequency of duplicates)
  - Consider fastp's sampling approach but avoid just sampling first 10k reads
  - Could extend existing dedup filter with frequency tracking

### 3. Cookbook Expansion (Issue #121)
High-impact cookbooks to add next:

**Immediate priorities:**
1. **Quality-Based Read Filtering** - Universal need
   - Expected error threshold filtering
   - Minimum quality score filtering
   - Low-quality end trimming

2. **Generic Adapter Trimming** - Extremely common
   - TruSeq and Nextera adapter removal
   - Standard preprocessing step

3. **Poly-A/T Tail Removal** - Complements existing QuantSeq cookbook
   - 3' RNA-seq tail removal
   - NextSeq poly-G artifact handling

4. **Paired-End Read Merging** - High value
   - Overlap paired reads
   - Quality improvement demonstration
   - Common in amplicon sequencing

**Medium priority cookbooks:**
5. Cell Barcode Demultiplexing (scRNA-seq growing importance)
6. Dual Index Demultiplexing (standard Illumina workflow)
7. Comprehensive Multi-Stage QC (best practices demonstration)
8. ATAC-seq Preprocessing
9. Small RNA-seq Processing
10. Amplicon Sequencing

## 🔬 Advanced Features

### 4. Sequence Analysis Features
- **Overrepresented sequence detection** (Issue #15)
  - k-mer counting and enrichment analysis
  - Similar to FastQC functionality
  - Useful for contamination detection

- **Read overlap detection (BD Rhapsody style)** (Issue #22)
  - Modified KMP substring search
  - Configurable mismatch rates
  - Prevent downstream mis-alignment

### 5. Merge Algorithm Improvements (Issue #120)
- Current fastp implementation has quirks
- Investigate pandaseq algorithms
- Add alternative merge strategies for different use cases

## 🧪 Testing & Quality Assurance

### 6. Fuzzing (Issue #127)
- Implement fuzzing based on seq_io approach
- Reference: https://github.com/markschl/seq_io/blob/HEAD/fuzz/README.md
- Improve robustness against malformed input

### 7. Test Coverage Improvements
- Current coverage: 87.6% regions, 83.0% functions, 92.5% lines
- Target: maintain >85% line coverage
- Focus on error handling paths and edge cases

### 8. Windows Testing (Issue #118)
- Figure out bash test execution on Windows
- Improve cross-platform reliability

## 🏗️ Architecture & Optimization

### 9. Memory Optimization (Issue #106)
- Investigate bumpalo arena allocator
- Reduce per-read allocation overhead
- Test memory usage with different allocators

### 10. Zero-Copy Optimizations (Issue #107)
- Zero-copy FASTA parser
- Reduce unnecessary data copying
- Improve overall throughput

### 11. Compression Investigation (Issue #12, #13)
- Parallel decompression opportunities
- Benchmark different compression backends
- Optimize I/O pipeline

## 📚 Documentation & UX

### 12. More Protocol-Specific Examples
- CRISPR guide RNA extraction workflow
- Ribosomal RNA filtering
- Contamination detection cookbook
- Small RNA-seq protocols

### 13. Interactive Mode Enhancements
- Already have basic interactive mode
- Could add live visualization of transformations
- Better error messages and suggestions

### 14. Enhanced Inspect Capabilities (Issue #124)
- Add tag data to inspect output lines
- Make it easier to debug pipeline issues
- Show intermediate transformation states

## 🐛 Bug Fixes & Technical Debt

### 15. Configuration Cleanup
- **IX separator duplication** (Issue #111)
  - Why is this stored in multiple places?
  - Consolidate and simplify

- **Undefined segments in output** (Issue #123)
  - Add error message for undefined segments
  - Better validation

- **Name without comment** (Issue #114)
  - Clarify naming conventions
  - Improve documentation

### 16. Tag System Improvements
- **Test using same tag twice** (Issue #122)
  - What happens when a tag is reused?
  - Document expected behavior or add validation

- **Length of missing tags** (Issue #126)
  - Define behavior when tags are not found
  - Consistent error handling

### 17. Mutant Analysis Follow-up (Issue #125, #126)
- Review mutational bugs found in cargo mutants analysis
- See issue #38089c45 commit for initial mutant analysis

## 🔍 Research & Benchmarking

### 18. Tool Comparisons
- **fastp reproducibility** (Issue #2)
  - Document differences and reproducibility issues
  - Validate our approach vs fastp

- **vals-umi investigation** (Issue #25)
  - Research their UMI handling approach
  - Consider adopting best practices

- **nxtrim investigation** (Issue #28)
  - Understand their Nextera trimming approach

### 19. Advanced Quality Metrics (Issue #14)
- Research state-of-the-art quality metrics
- Implement additional QC statistics
- Compare with modern tools

## 🎨 Nice-to-Have Features

### 20. External Tool Integration (Issue #20)
- Consider integration points with popular tools
- BAM annotation capabilities (Issue #5)
- Tool chaining and pipeline composition

### 21. Multi-file Input Support (Issue #10)
- Support processing multiple file sets
- Batch processing improvements

### 22. Read Name Manipulation Cookbook
- Extract information from read names using ExtractRegex
- Rename reads systematically
- Preserve metadata

## 📊 Current Strengths to Build On

- ✅ Strong timing infrastructure (recently added)
- ✅ Comprehensive transformation system
- ✅ Good test coverage (87.6% regions)
- ✅ Interactive mode for rapid development
- ✅ Flexible configuration system
- ✅ Multiple compression formats supported
- ✅ Growing cookbook collection

## 🎯 Recommended Immediate Actions

Based on impact and effort, here's a suggested order:

1. **Add 3-4 high-impact cookbooks** (Quality filtering, Adapter trimming, Poly-A removal, Paired-end merging)
   - High user value
   - Showcases existing capabilities
   - Relatively low effort

2. **Profile and fix report performance regression** (Issue #102)
   - Use new timing infrastructure
   - Address specific bottleneck
   - User-visible improvement

3. **Implement insert size histogram analysis** (Issue #4)
   - Building on existing merge capability
   - Standard metric users expect
   - Moderate complexity

4. **Add fuzzing tests** (Issue #127)
   - Improve robustness
   - Catch edge cases
   - One-time setup with ongoing benefits

5. **Duplication analysis enhancement** (Issue #16)
   - Extend existing dedup functionality
   - Valuable QC metric
   - Moderate effort

## 📝 Notes

- The project is in active development with 131 tracked issues
- Recent focus has been on timing/profiling infrastructure
- Strong emphasis on correctness, flexibility, and reproducibility
- Nix-based build system provides reproducibility
- Good balance of features vs complexity

---

*This document should be updated periodically as priorities shift and features are implemented.*
