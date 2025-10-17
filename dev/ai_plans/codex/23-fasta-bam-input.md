# Plan

1. Inspect current input/config/plumbing to understand how format detection and parsing work today.
2. Implement parser trait + format detection and refactor input handling to add FASTA/BAM support, updating validation and runtime code.
3. Backfill tests and documentation, including template updates and new fixtures covering FASTA/BAM scenarios.
