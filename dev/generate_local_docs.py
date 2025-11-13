#!/usr/bin/env python3
"""
Generate local documentation content for hugo serve.
This creates cookbook pages and template.toml in docs/content for local development.
"""

from pathlib import Path
import sys

from doc_utils import generate_cookbook_docs, copy_template_toml, copy_sample_report


def main() -> int:
    script_path = Path(__file__).resolve()
    repo_root = script_path.parent.parent
    docs_dir = repo_root / "docs"
    cookbooks_src = repo_root / "cookbooks"

    print("Generating local documentation content...")

    copy_template_toml(repo_root, docs_dir)
    generate_cookbook_docs(cookbooks_src, docs_dir)
    copy_sample_report(cookbooks_src, docs_dir)

    print("\nDone! You can now run 'cd docs; hugo serve'")
    return 0


if __name__ == "__main__":
    sys.exit(main())
