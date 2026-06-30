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
for error types, exhaustive match arms over internal enums). `panic!` /
`unreachable!` arms are ignored automatically (see [Auto-ignored lines](#auto-ignored-panicunreachable-lines));
mark everything else with coverage exclusion comments so it doesn't drag down
the reported percentage.

### Exclude a single line

```rust
Err(e) => panic!("internal error: {e}"), // cov:excl-line
```

The comment can appear anywhere on the line. (Lines containing `panic!(` or
`unreachable!(` are already ignored automatically, so the comment is optional
here — it is shown only to illustrate the syntax.)

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

### Auto-ignored panic/unreachable lines

Lines containing `panic!(` or `unreachable!(` are recognized automatically
and removed from coverage statistics — no comment required. They are kept in a
separate category from explicit exclusions: because such a line may or may not
be hit depending on which tests run, it is neither counted as covered nor as
uncovered, and it is never reported as a wrong exclusion. An explicit
exclusion that *is* nonetheless executed by a test is reported as a wrong
exclusion (and highlighted in HTML reports) so it can be fixed.

### When to use exclusions

Use exclusions **sparingly** and only for code that genuinely cannot be covered:

- `panic!` / `unreachable!` arms that guard against impossible states (handled
  automatically, no comment needed)
- Trait impls required by the type system but never exercised at runtime
  (`Debug`, `Display`, `From` for internal error variants)
- Exhaustive match arms on internal enums where one variant only exists for
  forward-compatibility

Do **not** exclude code just because it is hard to test. If a branch is hard to
reach, that is a signal to either write a targeted test or reconsider the design.
