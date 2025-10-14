# outcome: worked, mostly
# CalcExpectedError

1. Review existing numeric tag helpers (e.g., `extract_numeric_tags_plus_all`) and the `Q_LOOKUP` usage to confirm how to reuse or extend them for expected-error aggregation.
2. Implement the `CalcExpectedError` transform, wiring it into the transformation enum, enforcing PHRED+33 validation, and surfacing a `ConvertQuality` hint when scores are out of range.
3. Extend coverage: add a template/docs entry, update helper detection logic, and create test cases that cover mixed quality levels plus an out-of-range failure.
4. Run `cargo fmt` and the appropriate tests to confirm everything passes.
