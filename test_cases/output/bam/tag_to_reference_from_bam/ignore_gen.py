"""Generate input_reference.bam with reference sequences in the header."""
import pysam
import os

here = os.path.dirname(os.path.abspath(__file__))
out = os.path.join(here, "input_reference.bam")

header = pysam.AlignmentHeader.from_dict({
    "HD": {"VN": "1.6"},
    "SQ": [
        {"SN": "ref_alpha", "LN": 50},
        {"SN": "ref_beta",  "LN": 50},
        {"SN": "ref_gamma", "LN": 50},
    ],
})

with pysam.AlignmentFile(out, "wb", header=header) as f:
    pass  # no reads needed, just the header

print(f"Written: {out}")
