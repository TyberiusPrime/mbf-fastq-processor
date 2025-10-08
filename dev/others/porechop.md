Porechop
https://github.com/rrwick/Porechop

"Adapters on the ends of reads are trimmed off, and when a read has an adapter in its middle, it is treated as chimeric and chopped into separate reads. "

Unsupported as of 2018.


Their known issues section is interesting:


- adapter auto detection is broken, use fixed adapters

- seqan is too slow (???), usgegst using edlib.
https://github.com/Martinsos/edlib
'sequence alignment using levenshtein distance'
