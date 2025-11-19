#!/bin/bash
# Update all documentation from Rust source docstrings
# Run this after modifying docstrings in transformation/config structs

set -e

cd "$(dirname "$0")/.."

echo "Updating documentation from Rust docstrings..."
python3 dev/update_docs_from_rust.py --all

echo "Done! Documentation has been updated."
echo ""
echo "Files updated:"
echo "  - template.toml (field comments)"
echo "  - markdown docs (if markers are present)"
echo ""
echo "Review changes with: git diff"
