#!/usr/bin/env python3
"""Shared utilities for documentation generation."""

from pathlib import Path
import re
import shutil
import subprocess

def update_cookbook_output(cookbook_dir: Path) -> None:
    print("building", cookbook_dir)
    subprocess.check_call(['cargo','run','--release', '--','process','input.toml', '.', '--allow-overwrite'], 
                          cwd=cookbook_dir)
    ref_output_dir  = cookbook_dir / "reference_output"
    if ref_output_dir.exists():
        shutil.rmtree(ref_output_dir)
    ref_output_dir.mkdir()
    for fn in cookbook_dir.glob("output_*"):
        shutil.move(str(fn), ref_output_dir / fn.name)


def generate_cookbook_docs(cookbooks_src: Path, docs_dir: Path) -> None:
    """
    Generate Hugo markdown pages for cookbooks and create tar.gz archives.

    Args:
        cookbooks_src: Path to the cookbooks source directory
        docs_dir: Path to the docs directory where content and static files should be placed
    """
    if not cookbooks_src.exists():
        print("No cookbooks directory found, skipping cookbook generation")
        return


    cookbooks_dest = docs_dir / "content" / "docs" / "how-to" / "cookbooks"
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

    for ii, cookbook_dir in enumerate(cookbooks):
        update_cookbook_output(cookbook_dir)
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

        name_without_number = cookbook_name.split("-",1)[1]

        # Generate Hugo markdown page
        page_content = []
        page_content.append("+++\n")
        #page_content.append(f'title = "{name_without_number}"\n')
        #page_content.append(f'weight = {ii}\n')
        page_content.append("+++\n\n")

        # Add README content
        readme_text = readme.read_text(encoding="utf-8")
        page_content.append(readme_text)
        page_content.append("\n\n")

        # Add download link
        page_content.append(f"## Download\n\n")
        page_content.append(f"[Download {cookbook_name}.tar.gz](../../../../../cookbooks/{archive_name}) for a complete, runnable example including expected output files.\n\n")

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


def copy_template_toml(src_dir: Path, docs_dir: Path) -> None:
    """
    Copy template.toml from src to docs/content.

    Args:
        src_dir: Path to the directory containing src/template.toml
        docs_dir: Path to the docs directory
    """
    template_src = src_dir / "src" / "template.toml"
    template_dst = docs_dir / "content" / "docs" / "reference" / "toml" / "template.toml"

    if not template_src.exists():
        print(f"Warning: {template_src} not found")
        return

    template_dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(template_src, template_dst)
    print(f"Copied template.toml to {template_dst}")
