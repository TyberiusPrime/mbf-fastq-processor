# outcome: servicable, but I had in mind to reuse the memchr implementation we have in ValidateNames. My bad.
# TagOtherFileByName distinct readname separators

1. Audit the current separator handling
   - Trace how `readname_end_chars` is parsed in `src/transformations/extract/tag/other_file_by_name.rs` and confirm no other modules rely on the field.
   - Review existing configs (`src/template.toml`, docs) and integration fixtures under `test_cases/` to understand present expectations and avoid regressions.
2. Extend the configuration surface
   - Introduce `fastq_readname_end_chars` and `reference_readname_end_chars` fields on the `OtherFileByName`, removing with the legacy `readname_end_chars` .
3. Refactor separator application logic
   - Extract a helper that trims a read name given an optional separator set to remove duplication between `init` and `apply`.
   - Update the filter population loop to use the reference separators, leaving the raw value untouched when `None`.
4. Update the FastQ tagging logic
   - Apply the helper with the fastq separators inside the `extract_bool_tags` closure.
   - Ensure `ApproxOrExactFilter` receives the trimmed slices and that existing exact/approximate behavior remains intact.
5. Propagate configuration changes to user-facing docs
   - Refresh `src/template.toml` and `docs/content/docs/reference/tag-steps/generation/TagOtherFileByName.md` to describe the two new knobs.
6. Expand automated coverage
   - Add unit coverage for the new trimming helper, covering `None`, single char, multi-char, and empty separator cases.
   - Introduce or adjust an integration fixture that uses distinct fastq vs reference separators so we observe the new behavior end-to-end.
7. Polish and validate

   - Run `cargo fmt`, `cargo clippy --all-targets -- -D clippy::pedantic`, and the relevant `cargo test` suites (including the updated integration scenario).

## Test plan
- `cargo test other_file_by_name` (or targeted module path) to exercise unit coverage.
- Updated/added integration test via `cargo test --test integration_tests -- filter_other_file_by_name` after regenerating fixtures if needed.
- Full `cargo clippy --all-targets -- -D clippy::pedantic` to ensure lint cleanliness.
