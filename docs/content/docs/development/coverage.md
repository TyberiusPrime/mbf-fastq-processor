---
weight: 20
---
# Code Coverage

## Running coverage

```bash
python3 dev/coverage.py
```

This prints a line-coverage summary with exclusions applied (see below).

### Other formats

```bash
python3 dev/coverage.py --lcov    # writes coverage.lcov
python3 dev/coverage.py --html    # writes coverage-html/html/index.html
python3 dev/coverage.py --json    # writes coverage.json
python3 dev/coverage.py --all     # all of the above + summary
python3 dev/coverage.py --open    # --html and open in browser
```

The summary, LCOV, and HTML outputs all apply exclusions. HTML is rendered by
`genhtml` (from the `lcov` package, included in the nix dev shell) from the
post-processed lcov file; excluded lines appear as uninstrumented (white, not
red). JSON is generated directly by `cargo-llvm-cov` and does **not** reflect
exclusion comments.

To disable exclusion processing:

```bash
python3 dev/coverage.py --no-excl
```

## Excluding lines from coverage

Some lines are legitimately unreachable or only reachable in error conditions
that are impractical to trigger in tests (e.g. `panic!` arms, `Display` impls
for error types, exhaustive match arms over internal enums). Mark these with
coverage exclusion comments so they don't drag down the reported percentage.

### Exclude a single line

```rust
Err(e) => panic!("internal error: {e}"), // cov:excl-line
```

The comment can appear anywhere on the line.

### Exclude a block

```rust
// cov:excl-start
impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
// cov:excl-stop
```

The start and stop lines themselves are also excluded.

### When to use exclusions

Use exclusions **sparingly** and only for code that genuinely cannot be covered:

- `panic!` / `unreachable!` arms that guard against impossible states
- Trait impls required by the type system but never exercised at runtime
  (`Debug`, `Display`, `From` for internal error variants)
- Exhaustive match arms on internal enums where one variant only exists for
  forward-compatibility

Do **not** exclude code just because it is hard to test. If a branch is hard to
reach, that is a signal to either write a targeted test or reconsider the design.
