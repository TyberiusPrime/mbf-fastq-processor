---
weight: 55
---

# StoreTagInSequence

Insert a tag's string value into a read sequence at the position defined by another location tag.

```toml
[[step]]
    action = "StoreTagInSequence"
    in_value_label    = "mytag"    # location or string tag to insert
    in_position_label = "mytag2"   # location tag defining where to insert
    anchor = "Start"               # "Start"/"left" or "End"/"right" or 'replace'
    on_lost = "Complain" # Ignore|Complain
```

## Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `in_value_label` | location or string tag | yes | Tag whose sequence is inserted into the read |
| `in_position_label` | location tag | yes | Tag that defines the insertion position |
| `anchor` | `"Start"` / `"left"` / `"End"` / `"right"` / `"Replace"` | yes | Whether to insert before the leftmost position (`Start`), after the rightmost end (`End`) of the position tag, or replace the tag from start..end (single location tags or same length tags only)|

## How it works

`in_position_label` is a location tag pointing to a region (or multiple regions) in a
read. The insertion point is derived from the anchor:

- **`Start` / `left`** — insert *before* the leftmost `start` coordinate across all regions.
- **`End` / `right`** — insert *after* the rightmost `start + len` coordinate across all regions. - 
- ** `Replace` — replace the position's tag sequence.

The bytes inserted come from `in_value_label`:

- If it is a **location tag**, its sequences are joined (without spacer) and used.
- If it is a **string tag**, the string value is used directly.

Quality scores for the inserted bases are taken from the in_value 
if available, otherwise set to `~` (Phred+33 = Q93, maximum Sanger quality).

For replace, the positon tag must be a single consecutive location, 
or have the same length as replacement strings (individually for each read).
Otherwise, a run-time failure will be issued. 

## Location tag adjustment

Location tag's are tracked through all edits, though 

## Behaviour when tags are missing


If `in_value_label` is `Missing` or produces an empty sequence, or if `in_position_label`
carries no location information, the read is left unchanged and no error is raised.

## Behaviour when the location is no longer available

If reads have been manipulated in away that this tag's bases are (even partially lost), 
the on_lost handling comes to pass. By default, a run-time error is raised.
Set it to 'Ignore' to instead leave the reads unchanged.


## Example

Given read1 = `AAACCCGGG` (quality `IIIIIIIII`), with:

- `val_tag` extracting bases 0–2 (`AAA`, location [0,3])
- `pos_tag` extracting bases 3–5 (`CCC`, location [3,3])

```toml
# ignore_in_test
[[step]]
    action = "StoreTagInSequence"
    in_value_label    = "val_tag"
    in_position_label = "pos_tag"
    anchor = "Start"
```

Result: `AAA` inserted before position 3 → `AAAAAACCCGGG` (quality `III~~~IIIIII`).

With `anchor = "End"`:

Result: `AAA` inserted after position 6 → `AAACCCAAAGGG` (quality `IIIIII~~~III`).
