#!/usr/bin/env python3
from pathlib import Path
import re


for search_dir in "cookbooks", "test_cases":
    for cookbook_dir in Path(search_dir).glob("*"):
        if cookbook_dir.is_dir():
            for fn in list(cookbook_dir.glob("**/*.json")) + list(
                cookbook_dir.glob("**/*.html")
            ):
                if fn.is_file():
                    input = fn.read_text()
                    # censor /home/<path>s...
                    output = re.sub(
                        r"/home/[^\"]+", f"/home/user/{cookbook_dir.name}", input
                    )
                    if output != input:
                        print("Censored", fn)
                    fn.write_text(output)
