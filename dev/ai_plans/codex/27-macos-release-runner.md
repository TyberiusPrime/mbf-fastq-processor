# Task
Add macOS release runner that builds and tests, and ensure the Windows release runner runs tests in release mode.

## Plan
1. Review `.github/workflows/release.yml` to understand the existing build and Windows jobs.
2. Insert a release-mode `cargo test` step for the Windows matrix entry ahead of the build.
3. Add a macOS matrix entry that runs tests and builds the release binary, mirroring the upload logic.
4. Check the workflow for syntax issues or structural regressions.

## Notes
- macOS runner should target `aarch64-apple-darwin` to match the hosted architecture.
- Keep artifact upload paths aligned with the selected targets.
