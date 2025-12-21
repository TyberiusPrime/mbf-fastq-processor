status: closed
# output.incomplete doesn't work

We are generating the marker too early,
so when doing it a 2nd time we will clobber the files even though they
were not ours.

