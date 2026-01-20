#!/usr/bin/env bash
set -euo pipefail
mv input_read1.fq input_read1_original.fq
ln -s input_read1_original.fq input_read1.fq
