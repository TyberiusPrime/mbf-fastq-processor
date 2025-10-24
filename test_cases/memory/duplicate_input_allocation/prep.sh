#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

script_dir=$(cd "$(dirname "$0")" && pwd)
target="${script_dir}/../../integration_tests/inspect_index1/input_read2.fq.zst"
ln -sfn "$target" input_data.fq.zst