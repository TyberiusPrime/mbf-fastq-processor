#!/usr/bin/env bash
set -euo pipefail

ls -la
# we need to sort because of threading randomness
PROGRESS=$(cat output_progress.progress | sort | sed -E 's/[0-9]+/<number>/g')
printf '%s' "$PROGRESS" >output_progress.progress

