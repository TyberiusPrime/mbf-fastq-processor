# Task
Ensure `docs/content/docs/reference/toml/template.toml` mirrors the canonical `src/template.toml`, add a Nix-powered documentation build that refreshes the copy before running Hugo, and update the GitHub Pages workflow to use the new step.

## Plan
1. Materialize the canonical template under `docs/content/docs/reference/toml/template.toml` so documentation includes the reference file.
2. Extend `flake.nix` with a `documentation` package that copies the template before running Hugo, keeping the docs build self-contained.
3. Rework `.github/workflows/pages.yml` to build pages via `nix build .#documentation` and publish the resulting artifact.

## Notes
- Preserve existing workflow semantics (deployment permissions, branch filters).
- Ensure the documentation derivation emits Hugo output under `$out` for easy artifact upload.
- The copy step should overwrite any stale `docs/.../template.toml` prior to invoking Hugo.
