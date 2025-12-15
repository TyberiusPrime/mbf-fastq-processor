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

I think the key takeaway here is that allocs (& frees) hurt:
Our biggest time sinks are tag allocation & free, especially
with the convoluted 'Vec<Hit>' based tags.

Secondary, there is a number of steps that would benefit from better 
alignment, or a smarter algorithm (Count Oligos!).

Last, the fast once are needs_serial = false, so we can actually multicore them...


2592.60ms ExtractIUPACWithIndel ( needs better alignment)
1944.30ms Report_count_oligios (much improved to ~ 900ms, then allowed multicore, now down to 170ms )
1918.40ms Rename # greatly sped up if no {{READ_INDEX}} is present.
1801.40ms FilterReservoirSample # there's terribly little to do about this. 
            The worst part are all the tiny allocations & the final 'drop'
            We might get away with having our own FastQBlock, and reusing memory
            instead of replacement?
            Yeah, that improved by -40%.

1692.80ms ConcatTags # alloc limited.

1186.90ms Report_duplicate_count_per_fragment:
908.83ms HammingCorrect # alloc limited. ExtractRegions + hamming correct + forgetTags...
905.40ms Demultiplex # alloc, extractRegion, + forgetTags (), then hits_joined_sequence (!)
            there is a number off achievable gains here
f 859.17ms StoreTagInSequence # 30% etxract regions, 54 StoreTagInSequencs. Tiny improvements made.
f 743.61ms TrimAtTag # was 38% extract regions.  Now 67% extract regios
t 725.05ms QuantifyTag # 54% extract regions. 29% hits joined_sequence
f 704.68ms StoreTagLocationInComment  #30% extractRegion, 30% Forgettag. Rest actual store tag
f 702.65ms MergeReads # 84 in find_best_overlap_fastp - and that's all hamming distance calculations. 
                    Little hope for the fastp variant, might be better when doing actual alignment?


f 691.08ms StoreTagInComment
f 684.80ms UppercaseTag
t 681.20ms Report_duplicate_count_per_read:
f 659.23ms ExtractRegex
t 620.23ms TagDuplicates
f 619.42ms ConvertRegionsToLength
f 531.71ms FilterByTag
f 526.09ms LowercaseTag
f 514.62ms ExtractRegion
f 502.24ms ReplaceTagWithLetter
f 498.61ms ExtractRegions
t 477.21ms EvalExpression ( can run in parallel, down to 155 ms)
t 388.73ms Report_base_statistics
f 380.06ms ReverseComplement
f 352.31ms ExtractIUPAC
f 304.53ms ValidateName
f 215.64ms ExtractLongestPolyX
f 209.48ms ExtractLowQualityEnd
f 207.28ms ValidateSeq
f 196.47ms ExtractLowQualityStart
f 190.81ms Postfix
f 172.78ms SpotCheckReadPairing
f 172.69ms Swap
f 166.42ms Prefix
f 158.80ms ConvertQuality
f 158.20ms ExtractPolyTail
f 158.02ms Report_length_distribution (why?)
f 153.09ms CalcComplexity
f 153.07ms TagOtherFileBySequence
f 150.06ms CalcGCContent
f 149.37ms CalcLength
f 147.20ms TagOtherFileByName
f 140.28ms ExtractRegionsOfLowQuality
f 139.93ms CalcQualifiedBases
f 138.66ms CalcBaseContent
f 137.49ms CalcExpectedError
f 136.57ms ExtractIUPACSuffix
f 136.45ms CalcNCount
f 134.07ms ValidateQuality
f 128.60ms FilterSample
f 117.46ms LowercaseSequence
f 114.71ms UppercaseSequence
f 110.16ms CalcKmers
f 108.80ms Skip
f 106.57ms FilterEmpty
t 104.83ms Report_count (changed to multicore for 5% speed gain)
t 102.38ms Progress
f 101.75ms CutStart
f 98.83ms Truncate
f 96.69ms CutEnd
f 91.84ms FilterByNumericTag
t 26.11ms Head (but absolutely must be serial)

