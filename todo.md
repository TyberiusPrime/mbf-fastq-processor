- why are we slow in decompressing ERR13885883
    - as is                 ~ 44.7 s  (43.07 without output)
    - recompressed gz       - 44.7 s (42.39)
    - zstd                  - 43.53 s (24) (60s after tuning...)

- 

- why are the reports on ERR2224054 so slow. We're even doing less than fastp.
- 

- generally, reporting is much slower than just a stright cat? Trim for example is essentially free
    but 
