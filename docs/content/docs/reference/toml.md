---
weight: 3
title: TOML format

---
# TOML file format



```toml

# Our configuration file consists of at least two mandatory sections,
# and an arbitrary number of optional steps.

# Mandatory

[input]
    read1 = "file_1.fq" # we accept single files
    read2 = ["file_2.fq"] # or multiple files
    # multiple files will be 'cat'ed together
    # but the matching files must have the same number of reads


[output]
    prefix = "processed" # the prefix for the output files
    format = "Raw" # the output format, or 'None' to not output fastq

# Optional (but usually present)

# steps are applied to your FastQ data in the order they're defined,
# top to bottom.

[[step]]  # Note the 'array' syntax with double brackets here.
    # Define what step ot perform
    action = "CutStart" # the action to perform
    # arguments, depending on the action chosen.
    n = 3 
    target = "Read1"
```

Please see the reference section for detailed descriptions of the steps/actions available,
and full documentation for the input and output section.



## Why toml

We need something that allows data structures beyond key=value (e.g. nested
key=value for demultiplexing barcode definitions), provides an easy way to
order the steps, and allows for comments.

While CLI arguments can be order dependent, that's not the usual presumption.
They're also not standardized for more complex data structures and hard to comment.

JSON, while widely known, lacks comments.

TOML knowledge has spread wide with it's usage in python (and rust), 
and apart from it's insistence on one-line inline maps fit's our use case very well.
And that limitation can be worked around even when combined with the array-section syntax,
see the [demultiplex section](../demultiplex) for an example.


