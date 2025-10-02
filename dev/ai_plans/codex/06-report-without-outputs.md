# outcome: success
# Plan: Handle Report Without Output Paths

1. Inspect existing report-related validation fixtures (e.g., `test_cases/input_validation/report_names_distinct`) to match directory structure and expected panic formatting.
2. Extend `Config::check_reports` in `src/config/mod.rs` so that when any `Report`/ transform is present but neither `output.report_json` nor `output.report_html` is enabled, it pushes a descriptive error such as `"[output]: Report step configured, but neither output.report_json nor output.report_html is true."`.
4. Create a new fixture directory `test_cases/input_validation/report_without_output_flags/` containing:
   - `input.toml` mirroring other fixtures but omitting report output flags while defining a `Report` step.
   - `expected_panic.txt` populated with the error message from step 2.
5. Execute `dev/update_tests.py` so the integration harness picks up the new fixture and updates `tests/generated.rs`.
6. Run `cargo test -- test_case_input_validation_report_without_output_flags` (or outline equivalent validation) to ensure the new test passes and report results in the final summary.
7. Run `cargo test` verifying all tests passe
