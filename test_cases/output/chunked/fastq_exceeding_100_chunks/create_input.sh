##!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu
# 10 reads, we need 20000+1
#
TEN_READS=`cat input_read1.fq`
LINE_COUNT=`echo "$TEN_READS" | wc -l`
LINES_NEEDED=$(( (215 + 1) * 4 ))
MULTIPLIER=$(( (LINES_NEEDED / LINE_COUNT) + 1 ))

echo "Input has $LINE_COUNT lines, need $LINES_NEEDED lines, multiplying by $MULTIPLIER"
for i in $(seq 1 $MULTIPLIER); do
	echo "$TEN_READS"
done > input_read1.21k.excess.fq

WRITTEN=`wc -l < input_read1.21k.excess.fq`
echo "Actually written $WRITTEN lines"
# now truncate to lines needed
head -n $LINES_NEEDED < input_read1.21k.excess.fq > input_read1.21k.fq
rm input_read1.21k.excess.fq
rm input_read1.21k.fq.gz
gzip input_read1.21k.fq
