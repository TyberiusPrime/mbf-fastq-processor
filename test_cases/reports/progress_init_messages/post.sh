#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

ls -la
# we need to sort because of threading randomness
PROGRESS=$(cat output_progress.progress | sort | sed -E 's/[0-9]+/<number>/g')
printf '%s' "$PROGRESS" >output_progress.progress