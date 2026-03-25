## faster2

https://github.com/angelovangel/faster2
new version of faster  
faster2 outputs
    'gc content per read' (really per read)
    -read lengths (again per read)j
    -avg qual per read (again per read)
    -nx50 (ie. shortest length af 50% of the bases covered, I believe). Useful for pacbio/oxford nanopore I suppose.
      (How can I calculate that 'streaming')
    -percentage of q score of x or higher (1..93???)
    --table gives us:
            file    reads    bases    n_bases    min_len    max_len    N50    GC_percent    Q20_percent
            ERR12828869_1.fastq.gz    25955972    3893395800    75852    150    150    150    49.91    97.64
    (and goes about 388k reads/s from an ERR12828869_1.fastq.gz, single core. 67s for the file. 430k/s for uncompressed. no Zstd)



