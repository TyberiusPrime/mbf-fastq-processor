outcome: Success. Well, 99%, I had to place it in a separate file as intended.
# ForgetAllTags step

## Plan
1. Review existing `ForgetTag` implementation, documentation, template entry, and tests to mirror the expected behavior and validation patterns.
2. Implement the `ForgetAllTags` transformation in Rust, ensuring configuration validation knows it removes every tag and wiring it into the transformation enum and tag module exports.
3. Document the new step (reference docs + template) and add regression tests (config validation + integration fixture) that cover successful use and misuse after clearing tags.
