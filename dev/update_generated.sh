#!/usr/bin/env bash
THISDIR=$(dirname "$(readlink -f "$0")")
python "$THISDIR/_update_tests.py"
python "$THISDIR/_update_cookbooks.py"
