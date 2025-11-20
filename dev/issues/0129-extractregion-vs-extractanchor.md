status: open
# ExtractRegion vs ExtractAnchor


a) we can't currently extract from the end of a read, can we? (yes, start is a usize).
b) We could combine ExtractRegion and ExtractAnchor by replacing 'segment', with
'source' and then going 'tag:whatever'.


Problem: in ExtractIUPAC we use 'anchor' to mean where the extraction searches.
