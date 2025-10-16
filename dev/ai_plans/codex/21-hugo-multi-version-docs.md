## Goal
Automate documentation builds for main and tagged releases so GitHub Pages serves versioned Hugo sites.

## Plan
1. Design the python workflow: gather eligible tags, compute version metadata, and orchestrate per-version builds via git worktrees.
2. Implement `dev/build_documentation_all_releases.py` to generate `older_versions.md`, run Hugo into `docs/public/<version>/`, and write a redirecting root index.
3. Update `.github/workflows/pages.yml` to call the new script during the Pages build instead of the previous single-site invocation.
