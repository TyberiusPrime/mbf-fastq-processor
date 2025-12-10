#!/usr/bin/env bash
set -euo pipefail

PATH="" "$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)" 
