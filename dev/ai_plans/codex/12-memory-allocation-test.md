# Goal
Add an allocator measurement toggle to the CLI and cover it with a regression test that compares allocations for single vs duplicated inputs.

# Plan
1. Wire `allocation-counter` into `main` behind `RUST_MEASURE_ALLOC=1` and print a concise summary.
2. Teach the integration test runner to honor an optional `test.sh` script for custom invocations.
3. Add a new `test_cases/memory/duplicate_input_allocation` scenario with configs, symlinked FASTQ, and shell assertions that the duplicate configuration does not allocate significantly more memory.
