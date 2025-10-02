# outcome: planned
# fresh plan; execution not yet started.
# Error On Unused Extract Tags (Codex)

## Context
- `Extract*` transformations declare a tag via `declares_tag_type()` and the validation pass in `src/config/mod.rs:522` currently records the tag name and whether it provides location data.
- Downstream consumers advertise dependencies through `uses_tags()` or `removes_tag()`, but the validation never checks that a produced tag is observed by any later step.
- The goal is to surface a configuration error when a tag is set and flows through the pipeline without being consumed before the transformations finish.

## Numbered Implementation Plan
1. **Inventory tag metadata interfaces**
   - Audit the `Transformation` trait and concrete implementations to confirm that every tag-producing step funnels through `declares_tag_type()` and consumers through `uses_tags()/removes_tag()`.
   - Verify that `tag_requires_location()` still captures the only special-case consumer requirement so the new bookkeeping can piggyback on the existing map in `check_transformations`.

2. **Extend tag tracking state in `config::Config::check_transformations`**
   - Replace the current `HashMap<String, bool>` with a small struct (tag provides location, whether it has been used, step index of declaration) to support richer diagnostics.
   - Update insert, duplicate-label, and location-check branches to use the new struct while preserving current error messages verbatim.

3. **Mark tag usage during the validation walk**
   - When iterating `t.uses_tags()`, set the `used` flag on each matching tag entry before performing location checks so later logic can tell the tag has a consumer.
   - Ensure `removes_tag()` also marks the tag as used (the removal itself counts as “tag was acted on”), and gracefully handles tags that were already flagged missing.

4. **Emit errors for lingering unused tags**
   - After the main loop finishes, scan the tracking map for entries where `used == false` and push an error like `Extract label 'sample' declared at step 2 is never used downstream.`
   - Include available context: declaration step number, transformation description, and the list of steps examined, reusing the formatting style of existing errors.

5. **Add regression coverage**
   - Craft a focused unit test around `Config::check_transformations` (if existing module tests cover it) that builds a minimal config with an `Extract*` step lacking consumers and asserts the new error message.
   - Extend an integration fixture under `test_cases/` (or add a new one) to capture end-to-end behavior, then regenerate harness files via `python3 dev/update_tests.py` and run `cargo test` (include `-- --ignored` if required by touched fixtures).

6. **Documentation & release notes**
   - Update the relevant configuration documentation (likely `docs/content/docs/reference/Transforms/` or similar) to mention that unused tags now produce a hard error.
   - Add a short changelog or troubleshooting note if the project maintains one so users learn why a previously silent misconfiguration now fails.

