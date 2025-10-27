#!/usr/bin/env bash
set -euo pipefail

"$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)" < input_read1.fq
