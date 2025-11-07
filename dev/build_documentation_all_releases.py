#!/usr/bin/env python3
"""Build Hugo documentation for main and all release tags."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional, Tuple

EXCLUDED_TAGS = {
    "v.0.5.1",
    "v0.1.0",
    "v0.2.0",
    "v0.3.0",
    "v0.3.1",
    "v0.3.2",
    "v0.4.0",
    "v0.4.2",
    "v0.4.3",
    "v0.5.0",
    "v0.5.1",
    "v0.5.2",
    "v0.6.0",
    "v0.6.0.win",
    "v0.6.1",
    "v0.7.0",
}


@dataclass
class VersionEntry:
    ref: str
    folder: str
    label: str
    numeric: Optional[Tuple[int, ...]]
    suffix: str
    is_main: bool = False
    base_url: str = ""


class BuildError(RuntimeError):
    pass


def run(cmd: List[str], *, cwd: Path, check: bool = True, env: Optional[dict] = None) -> subprocess.CompletedProcess:
    result = subprocess.run(cmd, cwd=str(cwd), check=False, text=True, capture_output=True, env=env)
    if check and result.returncode != 0:
        stdout = result.stdout.strip()
        stderr = result.stderr.strip()
        message = f"Command {' '.join(cmd)} failed with code {result.returncode}."
        if stdout:
            message += f"\nstdout:\n{stdout}"
        if stderr:
            message += f"\nstderr:\n{stderr}"
        raise BuildError(message)
    return result


def extract_numbers(tag: str) -> Tuple[int, ...]:
    return tuple(int(part) for part in re.findall(r"\d+", tag))


def gather_release_entries(repo_root: Path) -> List[VersionEntry]:
    result = run(["git", "tag", "--list", "v*"], cwd=repo_root)
    tags: List[VersionEntry] = []
    for tag in result.stdout.splitlines():
        tag = tag.strip()
        if not tag or not tag.startswith("v"):
            continue
        if tag in EXCLUDED_TAGS:
            continue
        numbers = extract_numbers(tag)
        tags.append(VersionEntry(ref=tag, folder=tag, label=tag, numeric=numbers or None, suffix=""))
    tags.sort(key=lambda entry: (entry.numeric or tuple(), entry.label))
    return tags


def compose_base_url(base: Optional[str], folder: str) -> str:
    cleaned = (base or "").rstrip("/")
    if cleaned:
        return f"{cleaned}/{folder}/"
    return f"/{folder}/"


def generate_versions_content(entries: Iterable[VersionEntry], current: VersionEntry) -> str:
    lines = ["+++\n", 'title = "Documentation Versions"\n', 'description = "Snapshots built from releases and main."\n', "+++\n\n"]
    lines.append("Available builds:\n\n")
    for entry in entries:
        if entry is current:
            lines.append(f"- **{entry.label}** (this build)\n")
        else:
            lines.append(f"- [{entry.label}]({entry.base_url})\n")
    lines.append("\n")
    return "".join(lines)


def ensure_clean_output(output_root: Path) -> None:
    if output_root.exists():
        shutil.rmtree(output_root)
    output_root.mkdir(parents=True)


def write_redirect(output_root: Path, target_folder: str) -> None:
    index_path = output_root / "index.html"
    target = target_folder.rstrip("/") + "/"
    html = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="0; url={target}">
  <title>Redirecting...</title>
  <link rel="canonical" href="{target}">
</head>
<body>
  <p>Redirecting to <a href="{target}">{target}</a>...</p>
</body>
</html>
"""
    index_path.write_text(html.strip() + "\n", encoding="utf-8")


def generate_cookbook_docs(worktree_dir: Path, docs_dir: Path) -> None:
    """Generate Hugo markdown pages for each cookbook and create tar.gz archives."""
    cookbooks_src = worktree_dir / "cookbooks"
    cookbooks_dest = docs_dir / "content" / "docs" / "cookbooks"
    cookbooks_static = docs_dir / "static" / "cookbooks"

    # Clean and create directories
    if cookbooks_dest.exists():
        shutil.rmtree(cookbooks_dest)
    cookbooks_dest.mkdir(parents=True, exist_ok=True)

    if cookbooks_static.exists():
        shutil.rmtree(cookbooks_static)
    cookbooks_static.mkdir(parents=True, exist_ok=True)

    # Create index page
    index_content = []
    index_content.append("+++\n")
    index_content.append('title = "Cookbooks"\n')
    index_content.append('description = "Practical examples for common bioinformatics workflows"\n')
    index_content.append("weight = 5\n")
    index_content.append("+++\n\n")
    index_content.append("# Cookbooks\n\n")
    index_content.append("Complete, runnable examples demonstrating common use cases.\n\n")

    # Find all cookbooks (directories with input.toml)
    cookbooks = sorted([d for d in cookbooks_src.iterdir()
                       if d.is_dir() and (d / "input.toml").exists()])

    for cookbook_dir in cookbooks:
        cookbook_name = cookbook_dir.name
        readme = cookbook_dir / "README.md"
        input_toml = cookbook_dir / "input.toml"

        if not readme.exists() or not input_toml.exists():
            continue

        # Create tar.gz archive of the cookbook
        archive_name = f"{cookbook_name}.tar.gz"
        archive_path = cookbooks_static / archive_name

        # Create archive with tar
        subprocess.run(
            ["tar", "czf", str(archive_path), "-C", str(cookbooks_src), cookbook_name],
            check=True
        )

        # Generate Hugo markdown page
        page_content = []
        page_content.append("+++\n")
        page_content.append(f'title = "{cookbook_name}"\n')
        page_content.append(f'weight = {len(page_content) + 10}\n')
        page_content.append("+++\n\n")

        # Add README content
        readme_text = readme.read_text(encoding="utf-8")
        page_content.append(readme_text)
        page_content.append("\n\n")

        # Add download link
        page_content.append(f"## Download\n\n")
        page_content.append(f"[Download {cookbook_name}.tar.gz](../../../cookbooks/{archive_name})\n\n")

        # Add the TOML configuration
        page_content.append("## Configuration File\n\n")
        page_content.append("```toml\n")
        page_content.append(input_toml.read_text(encoding="utf-8"))
        page_content.append("```\n")

        # Write the Hugo page
        page_file = cookbooks_dest / f"{cookbook_name}.md"
        page_file.write_text("".join(page_content), encoding="utf-8")

        # Add to index
        title_match = re.search(r'^#\s+(.+)$', readme_text, re.MULTILINE)
        title = title_match.group(1) if title_match else cookbook_name
        index_content.append(f"- [{cookbook_name}]({cookbook_name}) - {title}\n")

    # Write index page
    index_file = cookbooks_dest / "_index.md"
    index_file.write_text("".join(index_content), encoding="utf-8")


def build_version(
    repo_root: Path,
    entry: VersionEntry,
    *,
    output_root: Path,
    version_entries: List[VersionEntry],
    base_env: dict,
    temp_base: Path,
) -> None:
    worktree_dir = temp_base / f"worktree-{entry.folder}"
    if worktree_dir.exists():
        shutil.rmtree(worktree_dir)
    worktree_dir.parent.mkdir(parents=True, exist_ok=True)
    run(["git", "worktree", "add", "--detach", str(worktree_dir), entry.ref], cwd=repo_root)
    try:
        docs_dir = worktree_dir / "docs"
        if not docs_dir.is_dir():
            raise BuildError(f"Missing docs directory in worktree for {entry.label} at {docs_dir}.")

        run(["git", "submodule", "update", "--init", "--recursive"], cwd=worktree_dir)

        template_src = worktree_dir / "src" / "template.toml"
        template_dst = docs_dir / "content" / "docs" / "reference" / "toml" / "template.toml"
        if template_src.exists():
            template_dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(template_src, template_dst)

        # Generate cookbook documentation and archives
        cookbooks_src = worktree_dir / "cookbooks"
        if cookbooks_src.exists():
            generate_cookbook_docs(worktree_dir, docs_dir)

        listing_path = docs_dir / "content" / "docs" / "older_versions.md"
        listing_path.parent.mkdir(parents=True, exist_ok=True)
        listing_path.write_text(generate_versions_content(version_entries, entry), encoding="utf-8")

        output_dir = output_root / entry.folder
        if output_dir.exists():
            shutil.rmtree(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        env = dict(base_env)
        env.setdefault("HUGO_ENVIRONMENT", "production")
        env.setdefault("HUGO_ENV", "production")
        env.setdefault("TZ", "America/Los_Angeles")

        cmd = [
            "hugo",
            "--gc",
            "--minify",
            "--baseURL",
            entry.base_url,
            "--destination",
            str(output_dir),
        ]
        run(cmd, cwd=docs_dir, env=env)
        print(f"Built documentation for {entry.label} -> {output_dir}")
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree_dir)], cwd=repo_root, check=False)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build docs for all releases and main using Hugo.")
    parser.add_argument(
        "--base-url",
        dest="base_url",
        default=None,
        help="Base URL for GitHub Pages (e.g., https://example.github.io/repo).",
    )
    parser.add_argument(
        "--main-ref",
        dest="main_ref",
        default="HEAD",
        help="Git ref to treat as the main (development) build.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    script_path = Path(__file__).resolve()
    repo_root = script_path.parent.parent
    output_root = repo_root / "docs" / "public"

    ensure_clean_output(output_root)

    releases = gather_release_entries(repo_root)
    main_entry = VersionEntry(
        ref=args.main_ref,
        folder="main",
        label="main",
        numeric=None,
        suffix="",
        is_main=True,
    )

    all_entries = [main_entry] + list(reversed(releases))
    for entry in all_entries:
        entry.base_url = compose_base_url(args.base_url, entry.folder)

    base_env = dict(os.environ)

    with tempfile.TemporaryDirectory(prefix="docs-build-") as temp_dir:
        temp_base = Path(temp_dir)
        for entry in all_entries:
            build_version(
                repo_root,
                entry,
                output_root=output_root,
                version_entries=all_entries,
                base_env=base_env,
                temp_base=temp_base,
            )

    redirect_target = main_entry.folder if main_entry in all_entries else (all_entries[0].folder if all_entries else None)
    if redirect_target:
        write_redirect(output_root, redirect_target)

    versions_list = ", ".join(entry.label for entry in all_entries)
    print(f"Built documentation for versions: {versions_list}")
    print(f"Output root: {output_root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
