#!/usr/bin/env bash

# make sure the index file input_read1.fq.gz.rapid_index exists
ls -la
cat config.toml

FN="input_read1.fq.gz.rapidgzip_index"
if [ ! -f $FN ]; then
	echo "Index file $FN not found!"
    echo "stderr"
    cat stderr
	exit 1
fi

