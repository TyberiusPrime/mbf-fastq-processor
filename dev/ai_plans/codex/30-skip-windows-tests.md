# Task
Skip specific integration tests on Windows (e.g., named pipes) where behavior cannot be supported even with WSL.

## Plan
1. Teach the test harness to respect a `skip_windows` marker file in a test case directory.
2. Drop marker files into the named pipe fixtures so they are ignored on Windows builds.
3. Run `cargo fmt` to keep formatting clean.

## Notes
- Emit a message when a test is skipped so the CI logs show intent.
- Leave non-Windows behavior untouched.
