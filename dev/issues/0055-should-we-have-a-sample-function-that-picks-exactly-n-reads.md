status: done
# should we have a sample function that picks exactly N reads?

- what happens if there are not enough reads.
- how does it differ from head, or a head/tail combo, 
- how does it actually decide which reads to keep? Given each read 
an equal probability is difficult when you don't know how many reads will be there
And making an approximate but almost exact estimator for the number of reads
we are going to see is too much work for this.


There is a method called reservoir sampling, and wikipedia has an optimal algorithm .


https://docs.rs/pdatastructs/latest/pdatastructs/reservoirsampling/struct.ReservoirSampling.html 
might fit the bill.

one issue though is that reservoir sampling won't have the final decisions 
until the end of the input, so it basically would block further processing until
you've seen all the reads.


