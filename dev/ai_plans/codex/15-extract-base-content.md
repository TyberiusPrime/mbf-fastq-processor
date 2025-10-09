# outcome: worked pretty well.
# Goal
Introduce an `ExtractBaseContent` step that generalizes GC content extraction so downstream code can count arbitrary bases, while keeping configs using `ExtractGCContent` working via automatic expansion.

# Plan
1. Inspect existing `ExtractGCContent` implementation and shared extract helpers to determine what needs to be abstracted.
2. Add the new transformation module, enum variant, and documentation describing the configurable base sets.
3. Update `Transformation::expand` to translate legacy `ExtractGCContent` steps into the new generalized step and trim the old implementation accordingly.
4. Create focused fixtures/tests covering `bases_to_count`/`bases_to_ignore` behavior and ensure existing GC scenarios still pass.
