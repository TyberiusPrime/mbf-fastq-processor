status: done
# investigate FastUing

[FastUinq](http://journals.plos.org/plosone/article?id=10.1371/journal.pone.0052249) ( duplicate reads for denovo analysis'?)

From the abstract, it's a an (old, 2012) dedup tool working on paired end reads.

Might work by sorting - but claims linear runtime?

Documentation says maximum 1000 pairs - is that a 1000 files... I think so.

Yeah, it's sorting based. Sort, then output only if they differ.


