# Task
Skip the `multi_stage_head` test when running in GitHub Actions, mirroring the `skip_windows` behaviour used for platform-specific failures.

## Plan
1. Extend the test harness to detect a `skip_github` marker and respect it only when `GITHUB_ACTIONS=true`.
2. Add the marker file under `test_cases/head_early_termination/multi_stage_head/`.
3. Run `cargo fmt` to keep formatting consistent.
