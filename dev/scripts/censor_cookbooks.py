#!/usr/bin/env python3
from pathlib import Path
import re


for search_dir in "cookbooks", "test_cases":
    for cookbook_dir in Path(search_dir).glob("*"):
        if cookbook_dir.is_dir():
            for fn in (
                list(cookbook_dir.glob("**/*.json"))
                + list(cookbook_dir.glob("**/*.html"))
                + list(cookbook_dir.glob("**/*.txt"))
            ):
                if fn.is_file():
                    try:
                        input = fn.read_text()
                    except UnicodeDecodeError:
                        # Some fixtures carry a text extension but hold binary
                        # data (e.g. gzipped output saved as .txt); nothing to
                        # censor there.
                        continue
                    # censor /home/<path>s...
                    # Stop at whitespace as well as `"` so unquoted paths in
                    # plain-text fixtures (e.g. expected_runtime_error.txt) don't
                    # swallow the rest of the file across newlines.
                    output = re.sub(
                        r"/home/[^\s\"]+", f"/home/user/{cookbook_dir.name}", input
                    )
                    if output != input:
                        print("Censored", fn)
                    fn.write_text(output)
