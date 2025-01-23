- why are we slow in decompressing ERR13885883
    - as is                 ~ 44.7 s  (43.07 without output)
    - recompressed gz       - 44.7 s (42.39)
    - zstd                  - 43.53 s (24) (60s after tuning...)


- test case for inspect missing!?

    


- exact case for FilterDuplicates


- reduce default thread coutn.
