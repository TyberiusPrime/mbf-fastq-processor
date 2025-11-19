#!/bin/bash
# Update template.toml from Rust source docstrings
# Run this after modifying docstrings in transformation structs

set -e

cd "$(dirname "$0")/.."

echo "Updating template.toml from Rust docstrings..."
python3 dev/update_template_from_rust_docs.py

echo "Done! template.toml has been updated."
echo "Please review the changes with: git diff mbf-fastq-processor/src/template.toml"
