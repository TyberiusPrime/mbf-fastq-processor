#!/usr/bin/env python3
"""
Generate local documentation content for hugo serve.
This creates cookbook pages and template.toml in docs/content for local development.
"""

from pathlib import Path
import shutil
import re
import subprocess
import sys


def generate_cookbook_docs_local(repo_root: Path) -> None:
    """Generate Hugo markdown pages for cookbooks and create tar.gz archives for local dev."""
    cookbooks_src = repo_root / "cookbooks"
    docs_dir = repo_root / "docs"
    cookbooks_dest = docs_dir / "content" / "docs" / "cookbooks"
    cookbooks_static = docs_dir / "static" / "cookbooks"

    if not cookbooks_src.exists():
        print("No cookbooks directory found, skipping cookbook generation")
        return

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
        page_content.append(f'weight = {len(cookbooks) - list(cookbooks).index(cookbook_dir)}\n')
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

    print(f"Generated {len(cookbooks)} cookbook pages in {cookbooks_dest}")


def generate_template_toml(repo_root: Path) -> None:
    """Copy template.toml from src to docs/content for local development."""
    template_src = repo_root / "src" / "template.toml"
    template_dst = repo_root / "docs" / "content" / "docs" / "reference" / "toml" / "template.toml"

    if not template_src.exists():
        print(f"Warning: {template_src} not found")
        return

    template_dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(template_src, template_dst)
    print(f"Copied template.toml to {template_dst}")


def main() -> int:
    script_path = Path(__file__).resolve()
    repo_root = script_path.parent.parent

    print("Generating local documentation content...")

    generate_template_toml(repo_root)
    generate_cookbook_docs_local(repo_root)

    print("\nDone! You can now run 'hugo serve' in the docs/ directory.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
