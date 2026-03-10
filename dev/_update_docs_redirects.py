#!/usr/bin/env python3
"""Generate redirect pages in docs/content/docs/redirects/.

Each page uses a custom Hugo layout that emits a bare <head> meta-refresh,
allowing link_docs() to use a stable /redirects/{name} URL that forwards
to the real documentation location.
"""

import shutil
from pathlib import Path

assert Path("docs/content/docs/reference").exists(), (
    "Starting from the wrong dir, docs not found"
)

docs_dir = Path("docs/content/docs")
reference_dir = docs_dir / "reference"
redirects_dir = docs_dir / "redirects"

# Rebuild the redirects directory from scratch
if redirects_dir.exists():
    shutil.rmtree(redirects_dir)
redirects_dir.mkdir()

# Hidden index so Hugo doesn't show this folder in navigation
(redirects_dir / "_index.md").write_text("---\nbookHidden: true\n---\n")

redirect_count = 0
for md_file in sorted(reference_dir.rglob("*.md")):
    if md_file.name == "_index.md":
        continue

    page_name = md_file.stem  # e.g. "FilterEmpty"

    # Relative URL from redirects/{page_name}/ to the target page
    rel = md_file.relative_to(reference_dir).with_suffix("")
    target_url = f"../reference/{rel}/"

    content = f"""\
---
title: "{page_name}"
type: redirect
redirect_to: "{target_url}"
---
"""
    (redirects_dir / f"{page_name}.md").write_text(content)
    redirect_count += 1

print(f"Generated {redirect_count} redirect pages in {redirects_dir}")
