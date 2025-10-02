# outcome: failure
# the incidentials were ok, but the actual implementation required rewriting
# new plan authored by Codex agent

## Context
- The `Rename` step currently applies only a regex replacement, making it impossible to inject sequential numbering into read names without pre-encoded metadata.
- Adding a reserved token that expands to the running read index lets operators renumber records deterministically during renaming.
- Any change must preserve existing behaviour, update user-facing docs, and expand test coverage to prove the new interpolation works alongside regex captures.

## Numbered Implementation Plan
1. **Review rename flow and data structures**
   - Confirm how `Rename::apply` iterates through read segments and whether indices are tracked per block or globally to choose the appropriate counter strategy.

2. **Inject read-index placeholder handling**
   - After the regex replacement, detect a magic token such as `{{READ_INDEX}}`, compute the 1-based running index for the processed read, and substitute it without disrupting binary-safe names.
   - Ensure the counter persists across segments within a block so paired reads share the same index suffix.

3. **Add configuration and docs**
   - Update `docs/content/docs/reference/Modification steps/Rename.md` (and any inline config examples) to describe the new placeholder, escaping requirements, and ordering relative to regex capture expansions.

4. **Exercise feature with tests**
   - Create an integration fixture under `test_cases/` that uses the placeholder to renumber reads and checks outputs for the expected sequence.
   - Regenerate the harness via `dev/update_tests.py` and assert behaviour in the generated integration test file.

5. **Validate and lint**
   - Run `cargo fmt`, `cargo clippy --all-targets -- -D clippy::pedantic`, and relevant `cargo test` targets to confirm the implementation is correct and style-compliant.

