# Segments

Modern sequencers, particularity Illumina sequencers can read multiple times 
from one (amplified) DNA molecule, producing multiple 'segments' that together form a 'molecule'.

The user's define the available segments in the [input-section]({{< relref "docs/reference/input-section.md" >}}), 
common names are 'read1', 'read2' (for paired-end reads) and 'index1', 'index2' (for dual-indexed libraries).

These are then commonly available to steps taking a `segment` or a  [`source`]({{< relref "docs/concepts/source.md" >}}) option. Often there is the option to work on all all defined segments at once by using the 'All' 'segment/source'.


