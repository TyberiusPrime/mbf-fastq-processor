status: done
# we should introduce a marker that signals even after the fact that processing was finished (even if no reports are requested). Rename the output files or such..

We could do it the other way around and have a marker file that we 
delete upon finishing.

And then the 'output file already exists' checker could ignore existing output files
if the marker was still around.
