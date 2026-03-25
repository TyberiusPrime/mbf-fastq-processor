# SortmeRNA 

https://github.com/biocore/sortmerna

Older tool (2012, but still gets commits and a release in 2024)


Main application: 
Filter reads for rRNA contamination (metatranscriptomic data), 
using rRNA databases.

They invested some work into getting the databases non-redundant-ish.


Paper at https://github.com/biocore/sortmerna/releases/download/v4.3.4/database.tar.gz


"We scan each read with a sliding window, and the accepted reads are those that have more than a threshold number of windows present in the database. For a given read and a given window on the read, we authorize one error (substitution, insertion or deletion) between the window and the rRNA database."

So... essentially a fuzzy overlapping kmer counter, using Burst Trie?



I strongly suspect, that would work just as well if you just used a 
probablistic set member ship test for each kmer...

