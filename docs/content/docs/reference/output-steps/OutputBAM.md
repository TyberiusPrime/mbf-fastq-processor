---
title: Output BAM
weight: 72
---

# OutputBAM

Write reads to BAM file(s).

```toml
[input]
    read1 = "input.fq"

[[step]]
    action = "OutputBAM"
    output = ["read1"]            # segments to write to individual files (alias: segments). Defaults to all input segments.
    suffix = "bam"                # (optional) override the file suffix
    compression_level = 6         # (optional) BGZF compression level 0-9
    # interleave = ["read1","read2"]  # segments to interleave into one file (alias: interleaved)
    # chunksize = 1000000         # (optional) split into files of N molecules
    output_hash_compressed = false    # write a compressed-content hash sidecar
    options = {
        comment_separation_char = ' ',
        tag_to_bam_tag = {'mytag'= 'XX'},
        tag_to_reference = {
           tag = "mytag",
            references_from_bam = "another_bam_filename.bam"
            # or 
            #references_from_barcodes = "barcode-section"
        },
        merge_demultiplexed = false, # requires tag_to_reference
        index_merged = false
    }

# [ output ]
#     prefix = "output"
#    compression_threads = 1       # output worker threads (per bam file)
```

### References

By default, BAM output is _unaligned_, and no references are defined in the BAM header.

You can define references, either from a template BAM, or from a set of barcodes
(length = barcode length) using `output.bam.tag_to_reference.references_from_bam`
or `output.references_from_barcodes`, which then get embedded in the BAM header.

This then enables using a string tag as the reference of a read. The tag 
must match one of the defined reference (or be empty/missing = unaligned).
Position is always 1.

This is useful for example to later quantify 'looked up' reads with [mbf-bam-quantifier](https://tyberiusprime.github.io/mbf-bam-quantifier/)
in 'preassigned [bam tag' mode](https://tyberiusprime.github.io/mbf-bam-quantifier/docs/reference/input-section/#bam_tag).

Please note the [#merge_demultiplexed](merge demultiplexed) section below.

### Other tags

You can store arbitrary tags into BAM tags using `output.bam.tag_to_bam_tag'.
It is a dict of 'our-tag-name' -> 'two-letter-bam-tag-name'. See the [SAM/BAM
tag spec](https://samtools.github.io/hts-specs/SAMtags.pdf) for predefined
codes. This works for string & location tags (-> type Z), numeric tags (-> type
float), and boolean tags (-> u8 0/1). Virtual tags are currently unsupported.


### Read name considerations

BAM read names are required to be ascii-readable-letters-minus-@ [!-?A-~] and shorter than 
254 bytes. We enforce this on output. You can enforce it earlier using the 
[ValidateReadNamesPrintable]({{< relref "docs/reference/validation-steps/ValidateReadNamesPrintable.md" >}})
step.

If you read name get's too long because of tags, consider shuffling them into a comment tag
by setting `output.bam.comment_separation_char` which places everything beyond this 
character in a BAM comment tag called 'CO', or not storing them in the read name 
but in BAM tags instead (see previous section).


### Interleaved BAM output
Interleaved produces one paired BAM with appropriate SAM flags for first/last segment.

### Other

BAM output cannot be streamed to stdout and requires `output_hash_uncompressed = false` (compressed hashes continue to work).

### Merge demultiplexed

When assigning reads to references using, you might wish to produce a sorted BAM output file.

To do so, you need to 
* create a tag with reference names  using
    [HammingCorrect]({{< relref "docs/redirects/HammingCorrect.md" >}})
* Demultiplex on that tag
* set the output to BAM
* store the tag in references using `output.bam.tag_to_reference.tag`
* merge the BAM files (which then will be trivially sorted) using `output.bam.merge_demultiplexed = true`

See the cookbook (todo) for more details.

## Chunking

When `chunksize` is set, start a new file every N molecules.
File names will end on .%number%, left padded with zeros to 
the actual needed number of digits.
