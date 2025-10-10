1. Review the current Inspect report implementation and identify the hooks needed to support selecting all segments with interleaved output.
2. Refactor Inspect to accept SegmentOrAll, collect reads from the chosen segments, and emit interleaved FASTQ when multiple segments are requested.
3. Add an integration fixture exercising Inspect with segment = "all" plus the expected output, and regenerate the generated test harness.
4. Update Inspect documentation/templates so users know how to request multi-segment interleaving and what filenames to expect.
5. Run formatting and the full cargo test suite to validate the changes.
