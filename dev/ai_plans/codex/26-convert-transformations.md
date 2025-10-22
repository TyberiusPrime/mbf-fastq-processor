# Convert transformations revamp

## Task
Rename the CalcRate step into a ConvertToRate conversion under a new convert category, add ConvertRegionsToLength and ExtractLongestPolyX transformations, and update docs/tests accordingly.

## Plan
1. Inspect existing CalcRate wiring, docs, and fixtures to understand dependencies.
2. Move the rate logic into src/transformations/convert/, rename to ConvertToRate, and update enum/template/tests/docs references.
3. Implement ConvertRegionsToLength to turn region tags into numeric length tags, with new unit/integration coverage and documentation.
4. Implement ExtractLongestPolyX mirroring ExtractPolyTail but scanning the full read, including fixtures and reference docs.
5. Refresh verification harnesses, templates, and run targeted cargo tests to validate the new steps.
