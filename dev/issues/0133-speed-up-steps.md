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


# 2025-12-15
(after making sure the workpool uses ~+ a similar number of threads as 
the previous implementation. Still most useful for relative,
not absolute measurements):


2592.60ms ExtractIUPACWithIndel
1944.30ms Report_count_oligios
1918.40ms Rename
1801.40ms FilterReservoirSample
1692.80ms ConcatTags

1186.90ms step_benchmarks/pipeline/Report_duplicate_count_per_fragment:
908.83ms HammingCorrect
905.40ms Demultiplex
859.17ms StoreTagInSequence
743.61ms TrimAtTag
725.05ms QuantifyTag
704.68ms StoreTagLocationInComment
702.65ms MergeReads


691.08ms StoreTagInComment
684.80ms UppercaseTag
681.20ms step_benchmarks/pipeline/Report_duplicate_count_per_read:
659.23ms ExtractRegex
620.23ms TagDuplicates
619.42ms ConvertRegionsToLength
531.71ms FilterByTag
526.09ms LowercaseTag
514.62ms ExtractRegion
502.24ms ReplaceTagWithLetter
498.61ms ExtractRegions
477.21ms EvalExpression
388.73ms Report_base_statistics
380.06ms ReverseComplement
352.31ms ExtractIUPAC
304.53ms ValidateName
215.64ms ExtractLongestPolyX
209.48ms ExtractLowQualityEnd
207.28ms ValidateSeq
196.47ms ExtractLowQualityStart
190.81ms Postfix
172.78ms SpotCheckReadPairing
172.69ms Swap
166.42ms Prefix
158.80ms ConvertQuality
158.20ms ExtractPolyTail
158.02ms Report_length_distribution
153.09ms CalcComplexity
153.07ms TagOtherFileBySequence
150.06ms CalcGCContent
149.37ms CalcLength
147.20ms TagOtherFileByName
140.28ms ExtractRegionsOfLowQuality
139.93ms CalcQualifiedBases
138.66ms CalcBaseContent
137.49ms CalcExpectedError
136.57ms ExtractIUPACSuffix
136.45ms CalcNCount
134.07ms ValidateQuality
128.60ms FilterSample
117.46ms LowercaseSequence
114.71ms UppercaseSequence
110.16ms CalcKmers
108.80ms Skip
106.57ms FilterEmpty
104.83ms Report_count
102.38ms Progress
101.75ms CutStart
98.83ms Truncate
96.69ms CutEnd
91.84ms FilterByNumericTag
26.11ms Head

