# Cookbook 10: Adapter Identification

## Use Case

You have a FASTQ file and want to identify which sequencing adapter is present
before trimming — or to confirm no adapter contamination remains after
trimming. This is useful when the adapter type is unknown, when working with
data from multiple library prep kits, or when validating a trimming step.

## What This Pipeline Does

1. Runs a single `Report` step that counts exact occurrences of each common
   adapter sequence in every read (`count_oligos`)
2. Writes an [HTML](./reference_output/output.html) and JSON report — no reads are filtered or written to disk

## How count_oligos Works

`count_oligos` performs exact, full-sequence matching across every read. A read
is counted if the probe sequence appears verbatim anywhere within it. There are
no mismatches and no IUPAC wildcards. A non-zero count means reads carry at
least one complete copy of that adapter.

Because the probe must appear in full, very short reads that were already
partially trimmed will *not* match. Use a shorter prefix of the adapter (e.g.
the first 15–20 bp) as an additional probe if you expect heavily trimmed data.

Every adapter is scored separately, so overlapping adapters are counted multiple times.

## Input Files

- `input/fastp_606.fq.gz` — Single-end reads containing the Illumina TruSeq Read 2 adapter

## Output Files

- `reference_output/output.report_adapter_check.html` — HTML report with oligo counts
- `reference_output/output.report_adapter_check.json` — JSON report with oligo counts

No FASTQ output is written (`format = 'None'`).

## Expected Results

With the provided sample data the report shows:

| Adapter | Count |
|---|---|
| `illumina_truseq_r2` | 1393 |
| all others | 0 |

This identifies the library as using the Illumina TruSeq Read 2 adapter (`AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT`).

## Next Steps

Once you have identified the adapter, add an `ExtractIUPAC` + `TrimAtTag` step to your pipeline. See [Cookbook 06: Adapter Trimming](../06-adapter-trimming) for a complete trimming example, and the [adapters reference]({{< relref "docs/reference/adapters.md" >}}) for copy/pastable `count_oligos` and `ExtractIUPAC` snippets.

## When to Use This

- Before trimming, to confirm which adapter is present
- After trimming, to verify that adapter contamination has been removed (counts should drop to zero)
- When processing data from an unknown or mixed source
