status: open
# speed up steps

We benchmarked, 
and while many of our steps are fast, 
we have some slow / medium slow ones.


terribly slow
7045.70ms step_benchmarks/pipeline/ExtractIUPACWithIndel
2542.20ms step_benchmarks/pipeline/MergeReads

slow
1933.40ms step_benchmarks/pipeline/Postfix (fixed to same as prefix, 1.15s)
1662.90ms step_benchmarks/pipeline/ConcatTags (15%).
1398.20ms step_benchmarks/pipeline/Report_count_oligios
1384.50ms step_benchmarks/pipeline/Report_base_statistics
1252.00ms step_benchmarks/pipeline/FilterReservoirSample
1239.50ms step_benchmarks/pipeline/Rename
1208.30ms step_benchmarks/pipeline/Prefix (got an easy 4% win here to 1.15s)
1185.10ms step_benchmarks/pipeline/Demultiplex
1122.80ms step_benchmarks/pipeline/StoreTagInSequence
1085.40ms step_benchmarks/pipeline/TrimAtTag
1042.30ms step_benchmarks/pipeline/StoreTagInComment
1033.90ms step_benchmarks/pipeline/QuantifyTag
1028.30ms step_benchmarks/pipeline/UppercaseTag
994.12ms step_benchmarks/pipeline/HammingCorrect
951.38ms step_benchmarks/pipeline/ConvertRegionsToLength
929.91ms step_benchmarks/pipeline/ReverseComplement
888.73ms step_benchmarks/pipeline/LowercaseTag
874.28ms step_benchmarks/pipeline/ExtractRegions
858.08ms step_benchmarks/pipeline/FilterByTag
844.82ms step_benchmarks/pipeline/ExtractRegion
770.10ms step_benchmarks/pipeline/ExtractIUPAC
704.15ms step_benchmarks/pipeline/Report_duplicate_count_per_fragment


only a little slow (2x...)

444.61ms step_benchmarks/pipeline/TagDuplicates
440.47ms step_benchmarks/pipeline/EvalExpression
427.89ms step_benchmarks/pipeline/ValidateSeq
394.00ms step_benchmarks/pipeline/TagOtherFileBySequence
385.10ms step_benchmarks/pipeline/TagOtherFileByName
381.13ms step_benchmarks/pipeline/ExtractRegex
361.97ms step_benchmarks/pipeline/ExtractLongestPolyX
324.08ms step_benchmarks/pipeline/Report_duplicate_count_per_read

