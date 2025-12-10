status: open
# ExtractRegion vs ExtractAnchor


a) we can't currently extract from the end of a read, can we? (yes, start is a usize).
b) We could combine ExtractRegion(s) and ExtractAnchor 
    by replacing 'segment', with 'source' and then going 'tag:whatever' for the region(s).


