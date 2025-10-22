# Task
Add a CI job that builds platform binaries for pull requests without publishing releases, making artifacts available for download.

## Plan
1. Examine existing workflows to decide whether to extend `test.yml` or create a new workflow for PR builds.
2. Implement a matrix job that builds (and possibly tests) the binaries, then uploads them as workflow artifacts.
3. Ensure the workflow triggers on pull requests and verify its YAML structure.

## Notes
- Reuse the platform matrix from the release workflow if possible.
- Consider running `cargo test --release` before building to keep parity with release builds.
