#!/usr/bin/env python3
"""
Update template.toml field comments from Rust source docstrings.

This script updates inline field comments in template.toml by extracting
documentation from Rust source files. The structure and examples are preserved.

Usage:
    python3 dev/update_template_from_rust_docs.py
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, field as dataclass_field


@dataclass
class FieldDoc:
    """Documentation for a struct field."""
    name: str
    doc: str


@dataclass
class StructDocs:
    """All field docs for a struct."""
    name: str
    fields: Dict[str, str]  # field_name -> documentation


def extract_doc_comment(lines: List[str], start_idx: int) -> str:
    """Extract doc comments (///) before a field/struct."""
    docs = []
    idx = start_idx

    # Go backwards to find all doc comments
    while idx >= 0:
        line = lines[idx].strip()
        if line.startswith('///'):
            doc_line = line[3:].strip()
            docs.insert(0, doc_line)
            idx -= 1
        elif line.startswith('#[') or line == '' or line.startswith('//'):
            idx -= 1
        else:
            break

    return ' '.join(docs)


def parse_struct_file(file_path: Path) -> Optional[StructDocs]:
    """Parse a Rust struct file and extract field documentation."""
    content = file_path.read_text()
    lines = content.splitlines()

    # Find the public struct definition
    struct_pattern = re.compile(r'pub\s+struct\s+(\w+)\s*\{')
    struct_match = None
    struct_line_idx = -1

    for idx, line in enumerate(lines):
        match = struct_pattern.search(line)
        if match:
            struct_match = match
            struct_line_idx = idx
            break

    if not struct_match:
        return None

    struct_name = struct_match.group(1)
    field_docs = {}
    brace_count = 0

    # Find all public fields and their docs
    for idx in range(struct_line_idx, len(lines)):
        line = lines[idx]
        brace_count += line.count('{') - line.count('}')

        if struct_line_idx < idx and brace_count <= 0:
            break

        # Look for public fields
        field_match = re.match(r'\s*pub\s+(\w+):\s*', line)
        if field_match:
            field_name = field_match.group(1)
            field_doc = extract_doc_comment(lines, idx - 1)

            if field_doc:
                field_docs[field_name] = field_doc

    if not field_docs:
        return None

    return StructDocs(name=struct_name, fields=field_docs)


def find_all_struct_docs(src_dir: Path) -> Dict[str, StructDocs]:
    """Find and parse all transformation struct files."""
    transformations_dir = src_dir / 'transformations'
    all_docs = {}

    # Parse transformation module files
    for module_dir in transformations_dir.iterdir():
        if module_dir.is_dir():
            for rust_file in module_dir.glob('*.rs'):
                struct_docs = parse_struct_file(rust_file)
                if struct_docs:
                    all_docs[struct_docs.name] = struct_docs

    # Parse top-level transformation files
    for rust_file in transformations_dir.glob('*.rs'):
        if rust_file.stem not in ['mod', 'prelude']:
            struct_docs = parse_struct_file(rust_file)
            if struct_docs:
                all_docs[struct_docs.name] = struct_docs

    # Parse config files
    config_dir = src_dir / 'config'
    for rust_file in config_dir.glob('*.rs'):
        if rust_file.stem in ['input', 'output', 'options']:
            struct_docs = parse_struct_file(rust_file)
            if struct_docs:
                all_docs[struct_docs.name] = struct_docs

    return all_docs


def get_transformation_mapping(src_dir: Path) -> Dict[str, str]:
    """Map action names to struct names from transformations.rs enum."""
    transformations_file = src_dir / 'transformations.rs'
    content = transformations_file.read_text()

    enum_match = re.search(r'pub enum Transformation \{(.*?)\n\}', content, re.DOTALL)
    if not enum_match:
        return {}

    enum_content = enum_match.group(1)
    mapping = {}

    for line in enum_content.splitlines():
        line = line.strip()
        match = re.match(r'([A-Z]\w+)\(\w+::(\w+)\)', line)
        if match:
            action_name = match.group(1)
            struct_name = match.group(2)
            mapping[action_name] = struct_name

    return mapping


def update_template_toml(template_path: Path, struct_docs: Dict[str, StructDocs],
                         action_mapping: Dict[str, str]) -> str:
    """
    Update template.toml by replacing field comments with Rust docstrings.
    Returns the updated content.
    """
    lines = template_path.read_text().splitlines()
    output_lines = []
    current_action = None
    current_struct = None

    # Create reverse mapping: struct_name -> action_name
    struct_to_action = {v: k for k, v in action_mapping.items()}

    for line in lines:
        # Check if we're entering a new transformation section
        section_match = re.match(r'#\s*====\s*(\w+)\s*====', line)
        if section_match:
            current_action = section_match.group(1)
            current_struct = action_mapping.get(current_action)
            output_lines.append(line)
            continue

        # Check if this is a field assignment line with a comment
        # Pattern: #    field_name = value # comment
        field_match = re.match(r'(#\s+)(\w+)\s*=\s*([^#]+?)\s*(#.*)?$', line)

        if field_match and current_struct and current_struct in struct_docs:
            indent = field_match.group(1)
            field_name = field_match.group(2)
            value_part = field_match.group(3).strip()
            old_comment = field_match.group(4) or ''

            # Get doc from Rust
            docs = struct_docs[current_struct]
            if field_name in docs.fields:
                new_comment = f"# {docs.fields[field_name]}"
                # Reconstruct line with new comment
                new_line = f"{indent}{field_name} = {value_part} {new_comment}"
                output_lines.append(new_line)
                continue

        # Default: keep line as-is
        output_lines.append(line)

    return '\n'.join(output_lines)


def main():
    """Main entry point."""
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    src_dir = repo_root / 'mbf-fastq-processor' / 'src'
    template_path = src_dir / 'template.toml'

    if not template_path.exists():
        print(f"Error: template.toml not found at {template_path}", file=sys.stderr)
        sys.exit(1)

    print("Extracting documentation from Rust source files...")
    struct_docs = find_all_struct_docs(src_dir)
    print(f"Found documentation for {len(struct_docs)} structs")

    print("Mapping transformations to structs...")
    action_mapping = get_transformation_mapping(src_dir)
    print(f"Found {len(action_mapping)} transformations")

    print("\nUpdating template.toml...")
    original_content = template_path.read_text()
    updated_content = update_template_toml(template_path, struct_docs, action_mapping)

    # Count changes
    original_lines = original_content.splitlines()
    updated_lines = updated_content.splitlines()
    changes = sum(1 for i, (old, new) in enumerate(zip(original_lines, updated_lines)) if old != new)

    print(f"Updated {changes} lines")

    # Write updated content
    template_path.write_text(updated_content)
    print(f"\nTemplate updated at {template_path}")


if __name__ == '__main__':
    main()
