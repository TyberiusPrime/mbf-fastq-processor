nix run nixpkgs#fastp -- --in1 ../reads_1.fq.gz --in2 ../reads_2.fq.gz -m --merged_out merged.fastp.gz --out1 read1.fastp.gz --out2 read2.fastp.gz -A -G -Q -L
rm fastp.json fastp.html # we don't care about records
