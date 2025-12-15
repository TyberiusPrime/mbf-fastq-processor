status: closed
# More cookbooks

4. Quality-Based Read Filtering

    Filter reads by expected error threshold (using CalcExpectedError + FilterByNumericTag)
    Filter by minimum quality scores (CalcQualifiedBases)
    Remove low-quality bases from ends (ExtractLowQualityStart/End + TrimAtTag)
    Real-world use: Standard QC for RNA-seq/DNA-seq before alignment

5. Read Length Filtering

    Filter by minimum/maximum read length (CalcLength + FilterByNumericTag)
    Remove empty reads (FilterEmpty)
    Use case: Remove degraded RNA reads or trim artifacts

6. Complexity Filtering

    Remove low-complexity reads (CalcComplexity + FilterByNumericTag)
    Useful for removing homopolymer runs and repetitive sequences
    Common in metagenomics and variant calling pipelines

7. N-Content Filtering

    Remove reads with excessive Ns (CalcNCount + filtering)
    Replace N regions with masking (ReplaceTagWithLetter)
    Standard QC practice

Adapter & Contamination Removal

8. Poly-A/T Tail Removal

    Trim poly-A tails using ExtractPolyTail + TrimAtTag
    Essential for 3' RNA-seq (similar to QuantSeq but more general)
    Also handle poly-G artifacts from NextSeq

9. Generic Adapter Trimming

    Detect and trim standard Illumina adapters using ExtractIUPACSuffix
    Show TruSeq, Nextera adapter removal
    Most common preprocessing step

10. Ribosomal RNA Filtering

    Use TagOtherFileBySequence to filter against rRNA sequences
    Essential for metagenomics and total RNA-seq
    Could provide example rRNA reference set

Paired-End Processing

11. Paired-End Read Merging

    Use MergeReads action to overlap paired reads
    Show quality improvement in overlapping regions
    Common in amplicon sequencing and short-insert libraries

12. Paired-End Quality Validation

    Demonstrate ValidateName and SpotCheckReadPairing
    Catch mismatched pair files early

Single-Cell & Barcode Processing

13. Cell Barcode Demultiplexing

    Extract cell barcodes from specific positions (ExtractRegions)
    Error correction with HammingCorrect
    Demultiplex with Demultiplex
    Valuable for plate-based scRNA-seq

14. Dual Index Demultiplexing

    Use both index1 and index2 for sample demultiplexing
    Common Illumina workflow
    Show barcode correction

15. UMI Deduplication Marking

    Tag duplicates based on UMI + position (TagDuplicates)
    Store UMI in comments for downstream tools
    Essential for accurate quantification

Protocol-Specific Workflows

16. ATAC-seq Preprocessing

    Trim Tn5 adapters (specific sequences)
    Filter by fragment length
    Paired-end merging for short fragments

17. Small RNA-seq

    Trim 3' adapters (common issue with short inserts)
    Length filtering (18-35bp typically)
    Remove 5' adapters if present

18. Amplicon Sequencing

    Trim primers from both ends using ExtractIUPAC
    Length validation
    Merge paired reads

19. CRISPR Guide RNA Extraction

    Extract guide sequences from specific positions
    Quality filtering
    Quantify with QuantifyTag

Advanced QC & Reporting

20. Comprehensive Multi-Stage QC

    Reports at multiple pipeline stages
    Compare before/after statistics
    Demonstrate progressive filtering impact

21. Contamination Detection

    Use TagOtherFileBySequence against contaminant databases
    Filter PhiX, adapter dimers, or specific organisms
    Report contamination levels

22. Duplicate Analysis

    Different duplicate detection strategies
    Fragment-level vs read-level
    Impact on downstream analysis

Data Sampling & Subsetting

23. Stratified Sampling

    Random sampling with FilterSample
    Reproducible with seed
    Useful for testing pipelines

24. Head/Skip for Large Datasets

    Quick dataset preview with Head
    Skip problematic initial reads
    Useful for development

More Advanced Operations

25. Read Name Manipulation

    Extract information from read names using ExtractRegex
    Rename reads systematically with Rename
    Preserve metadata

26. Quality Score Conversion

    Convert between Phred encodings using ConvertQuality
    Validate quality scores with ValidateQuality
    Handle legacy data

27. Reverse Complement Operations

    Show ReverseComplement usage
    Useful for specific protocols (e.g., strand-specific prep issues)

28. Sequence Masking & Modification

    Mask low-quality regions as N
    Lowercase/uppercase operations
    Custom sequence additions with Prefix/Postfix

My Recommendations for Next Cookbooks

Based on popularity and impact, I'd suggest implementing these in order:

    Quality-Based Filtering - Most universal need
    Adapter Trimming - Extremely common requirement
    Poly-A Tail Removal - Complements existing QuantSeq cookbook
    Paired-End Read Merging - High value for error correction
    Cell Barcode Demultiplexing - Growing importance with scRNA-seq
    Dual Index Demultiplexing - Standard Illumina workflow
    Comprehensive Multi-Stage QC - Shows best practices

These would provide broad coverage of common bioinformatics workflows while showcasing mbf-fastq-processor's capabilities.


- fastp default invocation

