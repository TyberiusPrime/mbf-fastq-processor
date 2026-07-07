#!/usr/bin/env bash
set -euo pipefail

CARGO_TARGET_DIR=target cargo mutants -j 5 \
  --test-workspace true \
  --workspace \
  --baseline skip  \
  --copy-target true \
  --profile dev \
  $@

python dev/scripts/filter_mutants.py >mutants.out/missed_filtered.txt
