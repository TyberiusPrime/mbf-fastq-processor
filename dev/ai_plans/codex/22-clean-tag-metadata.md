#outcome: failed. Prompting?
# Clean tag metadata duplication

1. Inspect existing tag metadata tracking and validation to understand current dependencies on `tag_provides_location`/`tag_requires_location`.
2. Promote shared tag validation helpers to the root transformations module and remove redundant trait hooks across steps and config validation.
3. Update transformation steps that require location tags to call the shared validators and adjust configuration bookkeeping.
4. Format and check the workspace to confirm the refactor compiles cleanly.
