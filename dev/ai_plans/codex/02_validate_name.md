# outcome success
# not quite what I wanted, but very close
# ValidateName Step Implementation Plan (Codex)

## Context
- The validation pipeline lives in `src/transformations/validation/` with per-step modules (`quality.rs`, `seq.rs`) wired through `src/transformations/validation.rs` and the main `Transformation` enum in `src/transformations.rs`.
- We need a new `ValidateName` step that checks every read tuple in `FastQBlocksCombined` to ensure all present segments share the same name, optionally comparing only the prefix before configured separator characters.
- The workspace uses `eserde` + `BString` helpers, golden test fixtures under `test_cases/`, auto-generated integration wiring via `dev/update_tests.py`, and public docs under `docs/content/docs/reference/Validation Steps/`.

## Implementation Tasks

1. **Wire the module**
   - Add `src/transformations/validation/name.rs` and register it from `src/transformations/validation.rs` (`mod name;` + `pub use name::ValidateName;`).
   - Extend `Transformation` in `src/transformations.rs` with a `ValidateName(validation::ValidateName)` variant and ensure `serde` + `enum_dispatch` imports stay sorted.
   - Update any helper enums or match arms that need exhaustive handling (no extra logic expected beyond adding the new variant to the default `res.push` paths).

2. **Define the configuration struct**
   - Create `ValidateName` with `#[derive(eserde::Deserialize, Debug, Clone)]` and `#[serde(deny_unknown_fields)]`.
   - Fields:
     - `#[serde(default = "default_readname_end_chars", deserialize_with = "crate::config::deser::option_bstring_from_string")] pub readname_end_chars: Option<BString>` where `default_readname_end_chars()` returns `Some(super::super::default_name_separator())`.
     - Optional future-proof knob (e.g. `case_sensitive: bool`) is *not* needed unless we discover a requirement; keep scope tight.
     - Cache a separator lookup structure after deserialization (e.g. `Option<bstr::ByteSet>`) in a skipped field so runtime checks are fast.
   - Implement `Step::validate_segments` to confirm at least one segment exists in the input definition (mirrors checks in other validations) and to precompute the separator lookup.

3. **Implement `Step::apply`**
   - Use `FastQBlocksCombined::apply_mut` to iterate per read index and borrow all segments simultaneously.
   - For each tuple of `WrappedFastQReadMut`:
     - Capture the reference name from the first segment; bail early if any name is empty.
     - If separators are configured, find the first occurrence (use `ByteSet::find` or `position(|b| separators.contains(b))`) to compute the canonical prefix; otherwise compare the entire name.
     - Ensure every other segment has a name starting with the same prefix and, when in exact mode, matches length as well.
   - On mismatch, return an error via `anyhow::bail!` with a message that includes the offending names, the computed prefix, and the 0-based read index; mirror the formatting style used in `ValidateSeq`/`ValidateQuality` for consistency.
   - Leave `needs_serial` and `transmits_premature_termination` at their defaults (no override required) so the step stays parallel-friendly.

4. **Surface the step to users**
   - Document the new configuration snippet in `src/template.toml` within the Validation section, including examples for prefix mode (default `_`) and exact matching (set `readname_end_chars = ""` or explicitly `[]` once semantics are finalized).
   - If any default transformation expansion relies on validation steps, verify whether `expand()` in `src/transformations.rs` needs to consider `ValidateName`; right now no automatic insertion is expected.

5. **Tests & fixtures**
   - Add golden test cases under `test_cases/validation/`:
     - `validate_name` (happy path: identical names across segments, default underscore prefix tolerance, single-read scenarios).
     - `validate_name_custom_separator` (custom `readname_end_chars` demonstrating multiple separators).
     - `validate_name_fail` (mismatched prefix and missing separator coverage with explicit failure message expectation).
   - Each fixture needs `input.toml`, input FASTQs, and (for failures) the expected stderr fragment described in `tests/test_runner.rs` conventions.
   - Regenerate `tests/generated.rs` using `python3 dev/update_tests.py` so the new cases are registered, then run `cargo test` (including `-- --ignored` if slow cases are touched).

6. **Documentation**
   - Create `docs/content/docs/reference/Validation Steps/ValidateName.md` with a short description and TOML snippet; update `_index.md` if the page list is manually curated.
   - Cross-link from any relevant troubleshooting or validation overview pages if those lists are maintained manually.

7. **Verification & polish**
   - Run `cargo fmt`, `cargo clippy --all-targets -- -D clippy::pedantic`, and `cargo test` to ensure the new step passes linting and tests.
   - Consider adding a focused unit test (e.g. in `name.rs`) for the separator parsing logic to catch edge cases beyond the golden fixtures.
   - Double-check error strings for clarity and absence of lossy UTF-8 conversions (use `BString` formatting as needed).

## Open Questions / Follow-ups
- Confirm whether `readname_end_chars = ""` should be treated as “exact match” or whether we need an explicit boolean flag; adjust serde defaults accordingly once product expectations are clarified.
- Decide if we should ignore trailing whitespace in names or treat them as significant; default assumption is “names are byte-precise”.
