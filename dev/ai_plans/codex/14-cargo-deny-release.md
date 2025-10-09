# Goal
Run `cargo deny` during the GitHub release workflow so new advisories or policy violations block the release.

# Plan
1. Review the release GitHub Actions workflow to find the right place to add a `cargo deny` step.
2. Confirm how `cargo deny` is invoked in this repo (via flake/devshell) and decide on the command we can run in CI.
3. Update the workflow to install prerequisites and execute `cargo deny`, ensuring it fails the job on new findings.
