status: open
# investigate https://github.com/sequencing/NxTrim

" Software to remove Nextera Mate Pair junction adapters and categorise reads according to the orientation implied by the adapter location." From illumina itself, bsd license.

"Based on the location of the Nextera junction adapter (if detected), nxtrim produces four different "virtual libraries":

    mp: read pairs that are large insert-size mate-pairs, both mates will be reverse-complemented by nxtrim (from RF to FR) unless --rf commandline option is used
    pe: read pairs that are short insert-size paired-end reads due to the junction adapter occurring early in a read
    se: single reads (reads having no R1 or R2 counterpart)
    unknown: a library of read-pairs that are mostly large-insert mate-pair, but possibly contain a small proportion of paired end contaminants

Output is reverse-complemented such that the resulting reads will be in forward-reverse orientation.
"
