#outcome: success
# Plan: Configurable ix separator

1. Extend `[output]` configuration with an `ix_separator` setting (default `_`) and validate against path separators.
2. Introduce a shared helper for joining filename components and apply it to demultiplexed outputs plus Progress/Inspect/QuantifyTag steps.
3. Refresh templates and documentation to explain the new separator behaviour.
4. Add an integration test using a non-default separator that exercises Demultiplex, Progress, Inspect, QuantifyTag, and StoreTagsInTable; regenerate generated tests and run the targeted `cargo test`.
