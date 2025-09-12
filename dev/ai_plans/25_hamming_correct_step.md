Currently, hamming correction is embedded in the demultiplex step.

We should add it as it's own step (takes a tag, produces a tag at some correction,
just with replaced sequence, remove the hit for no match), sharing implementation code with 
demultiplex as far as possible.

We don't want to remove it totally from demultiplex, since then the user 
would have to specify the barcodes twice.


