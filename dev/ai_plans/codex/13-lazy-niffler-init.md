# Goal
Delay niffler reader construction until a FASTQ file is actually parsed so we avoid buffering every compressed stream at once.

# Plan
1. Change `open_input_files` to return raw file handles instead of instantiated niffler readers.
2. Update `FastQParser` to create and cache the niffler reader only when consuming a file.
3. Propagate the new types through call sites and run `cargo check` to confirm the refactor builds.
