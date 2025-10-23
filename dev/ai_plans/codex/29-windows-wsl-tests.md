# Task
Ensure Windows CI runners can execute `prep.sh` scripts by provisioning WSL and adjusting the test harness accordingly.

## Plan
1. Modify the integration test harness to execute `prep.sh` via WSL on Windows, translating paths as needed.
2. Provision WSL in the Windows GitHub Actions jobs so the required shell environment is available.
3. Run `cargo fmt` to keep formatting consistent and verify no syntax issues remain.

## Notes
- WSL path conversion must handle `\\?\`-prefixed canonical paths and drive-letter casing.
- WSL is only required for the Windows matrix jobs; gate the setup step with a matrix check.
