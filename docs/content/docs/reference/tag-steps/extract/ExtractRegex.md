---
weight: 50
---

# ExtractRegex

Extract a regexp result. Stores an empty string if not found.

```toml
[[step]]
    action = "ExtractRegex"
    out_label = "mytag"
    search = "^CT(..)CT"
    replacement = "$1"  # standard regex replacement syntax
    source = "read1" # An input segment (to read from sequence), or name:<segment> to read from a tag
```

This transformation searches for a regular expression pattern in the specified read and extracts the matching portion as a tag.

The value actually 'extracted' is after replacement has been performed.

See the [regex documentation](https://docs.rs/regex/latest/regex/#syntax) for details on the syntax supported.

Note that you need to 'double up' on backslashes for character classes inside the search string.

