#!/usr/bin/env python3
"""Regenerate the committed JSON schema shipped by the docs site.

The GitHub Pages workflow does not build the binary; it serves
`docs/static/schema.json` verbatim. So that file is the source of truth for the
published schema and must be regenerated whenever the config types change, just
like the other generated artifacts. The `committed_schema_is_up_to_date` test in
fastqrab-steps fails if this wasn't run.
"""

from pathlib import Path
import subprocess

assert Path("docs").exists(), "Starting from the wrong dir, docs not found"

schema = subprocess.check_output(
    ["cargo", "run", "-q", "--bin", "fastqrab", "--", "json-schema"],
    text=True,
)

out_path = Path("docs/static/schema.json")
out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(schema, encoding="utf-8")
print(f"Wrote {out_path}")
