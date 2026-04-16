import pysam


def write_sam_from_tuples(tuples, out_path, ref_name="chr1", ref_len=10_000):
    """
    tuples: iterable of (name, seq)
    out_path: output SAM file path
    """

    # minimal header with one reference
    header = {
        "HD": {"VN": "1.6"},
        "SQ": [{"LN": ref_len, "SN": ref_name}],
    }

    with pysam.AlignmentFile(out_path, "wb", header=header) as outf:
        for name, seq in tuples:
            a = pysam.AlignedSegment()
            a.query_name = name
            a.query_sequence = seq

            # constant high quality ("I" = Phred 40)
            a.query_qualities = pysam.qualitystring_to_array("I" * len(seq))

            # minimal alignment fields (unmapped)
            a.flag = 4  # unmapped
            a.reference_id = -1
            a.reference_start = -1
            a.mapping_quality = 0
            a.cigar = ()

            outf.write(a)


write_sam_from_tuples(
    [
        (
            "ref_alpha",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ),
        (
            "ref_alpha Repeated",
            "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTAC",
        ),
        (
            "ref_beta",
            "TGCATGCATGCATGCATGCATGCATGCATGCATGCATGCATGCATGCATG",
        ),
        (
            "ref_gamma",
            "CGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCG",
        ),
    ],
    "reference.bam",
)
