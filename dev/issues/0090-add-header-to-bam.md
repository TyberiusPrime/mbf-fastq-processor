status: closed
# Add header to bam

Can we dump the complete config into the PG header of a BAM file?
or does in not do multi-line data?

I think it doesn't do multi-line data. 

Can you single line toml?
No you can't.

We could base64 encode it, but that is absolutely user unreadable.
Maybe web url style encoding?

And add a comment line param to actually read that instead of an input.toml?

If that doesn't work out (how large can a @PG line be? BAM says <2^31 bytes
for the header text. Including the final 0x0.),
we should add a 'verify-toml-hash' command line option,
and record that in CL.


-- 
Closed, @PG is not a good idea in the first place.
