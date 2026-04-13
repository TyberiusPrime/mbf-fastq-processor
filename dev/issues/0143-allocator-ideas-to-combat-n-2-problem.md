status: closed
# Allocator ideas to combat n^2 problem

right now, if we repeatedly grow a read,
we're n^2 in both time and space.

If we split the current 'size' u64 into two u32, 
size & capacity, we instantly gain the ability to 
handle tiny increases without allocating (since we can
'reuse' the unused newlines & + bytes in FASTQ).

Now, if we also look at the final read sizes,
we can send back an estimate of the necessary size per read
(median, 90th quantile, something) for the next blocks to be read.

They can then, upon the first expansion of the read (seq & qual...)
allocate to the estimated final size.


Draw back: if it's not a constant size extension, we might waste some bytes.
Advantage: O(n) in both time and space.

I think we can negate the overalloc by being smart about it,
and most cases will end up with a fixed number of bytes added to a read,
won't they (so constant growth, not necessarily constant final length.)


-- 
I actually think that's not what's happening.
Vec allocation in rust is self-scaling, promising O(1) amortized push.,
and we are growing the arena if necessary.

We could query the steps for 'expected growth', 
but it's a hyperoptimiziation.
