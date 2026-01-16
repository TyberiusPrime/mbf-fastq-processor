status: done
# We should either make the initial capacity of the cuckoo filters configurable,
  or estimate it based on the input file size and the first block we've read?

Doesn't need to be exact, but there's a ton of runtime difference between
having one-ish cuckoo filter and two.

-- 
newish estimation code is good enough
