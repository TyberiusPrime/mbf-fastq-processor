#!/usr/bin/env bash

# make sure the index file input_read1.fq.gz.rapid_index exists
FN="input_read1.fq.gz.rapid_index"
if [ ! -f $FN]; then
	echo "Index file $FN not found!"
	exit 1
fi
