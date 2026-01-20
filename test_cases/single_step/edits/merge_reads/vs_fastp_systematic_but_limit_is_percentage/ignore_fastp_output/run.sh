#!/usr/bin/env bash
nix run nixpkgs#fastp -- \
  --in1 ../reads_1.fq.gz \
  --in2 ../reads_2.fq.gz\
  -m --merged_out merged.fastp.gz \
  --out1 read1.fastp.gz \
  --out2 read2.fastp.gz -A -G -Q -L \
  --overlap_diff_percent_limit 2 \
  --overlap_diff_limit 10
rm fastp.json fastp.html # we don't care about records
