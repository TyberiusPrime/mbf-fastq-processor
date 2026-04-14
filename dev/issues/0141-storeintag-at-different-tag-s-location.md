status: open
# StoreTagInSequence at different tag's location

This will enable a cookbook that takes UMI from read name
and stores it back into the read at arbitrary position,
by doing an ExtractRegion (start=x, len=0)

Problem StoreTagInSequence 
supports 'split regions', and I don't see how that will 
work with arbitary 'position targets'.

So maybe another step?

Rename this one 'StoreTagBackInSequence'

Then StoreTagInSequence tages two labels,
in_value_label, in_position_label, 
and an anchor (left,right) and stores 
it there.
