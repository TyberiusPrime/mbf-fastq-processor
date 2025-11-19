#!/usr/bin/env python3
"""
Update documentation files from Rust source docstrings.

This script updates both template.toml and markdown documentation files
by replacing marked sections with fresh documentation from Rust sources.

Marker format for template.toml:
    # AUTO-GENERATED-START: StructName
    ... replaced content ...
    # AUTO-GENERATED-END

Marker format for markdown (.md):
    <!-- AUTO-GENERATED-START: StructName -->
    ... replaced content ...
    <!-- AUTO-GENERATED-END -->

Usage:
    python3 dev/update_docs_from_rust.py [--template] [--markdown] [--all]
"""

import argparse
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
    field_type: str
    has_default: bool = False
    aliases: List[str] = dataclass_field(default_factory=list)


@dataclass
class StructDoc:
    """Documentation for a transformation struct."""
    name: str
    struct_doc: str
    fields: Dict[str, FieldDoc]


def extract_doc_comment(lines: List[str], start_idx: int) -> str:
    """Extract doc comments (///) before a field/struct."""
    docs = []
    idx = start_idx

    # Go backwards collecting ONLY doc comments (///)
    # Stop at anything that isn't a doc comment, attribute, or blank line
    while idx >= 0:
        line = lines[idx].strip()
        if line.startswith('///'):
            doc_line = line[3:].strip()
            docs.insert(0, doc_line)
            idx -= 1
        elif line.startswith('#[') or line == '' or line.startswith('//'):
            # Skip attributes, blank lines, and regular comments
            # But only if we haven't collected docs yet or might find more docs
            idx -= 1
        else:
            # Hit actual code, stop
            break

    return ' '.join(docs)


def parse_struct_file(file_path: Path) -> Optional[StructDoc]:
    """Parse a Rust struct file and extract all documentation."""
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

    # Extract struct-level documentation
    struct_doc = extract_doc_comment(lines, struct_line_idx - 1)

    # Find all public fields
    field_docs = {}
    brace_count = 0

    for idx in range(struct_line_idx, len(lines)):
        line = lines[idx]
        brace_count += line.count('{') - line.count('}')

        if struct_line_idx < idx and brace_count <= 0:
            break

        # Look for public OR private fields (serde uses private fields)
        field_match = re.match(r'\s*(pub\s+)?(\w+):\s*(.+?)(?:,|$)', line)
        if field_match and field_match.group(2) not in ['skip', 'default']:  # Field name
            field_name = field_match.group(2)
            field_type = field_match.group(3).strip().rstrip(',')
            field_doc = extract_doc_comment(lines, idx - 1)

            if field_doc:
                # Check for skip, default, and aliases
                # Only look at attributes directly preceding this field (stop at blank line or other code)
                has_skip = False
                has_default = False
                aliases = []
                check_idx = idx - 1
                while check_idx >= 0:
                    attr_line = lines[check_idx].strip()
                    if attr_line.startswith('#['):
                        if '#[serde(skip)]' in attr_line or 'serde(skip' in attr_line:
                            has_skip = True
                        if '#[serde(default)]' in attr_line or 'serde(default' in attr_line:
                            has_default = True
                        alias_match = re.search(r'alias\s*=\s*"(\w+)"', attr_line)
                        if alias_match:
                            aliases.append(alias_match.group(1))
                        check_idx -= 1
                    elif attr_line.startswith('///') or attr_line == '' or attr_line.startswith('//'):
                        check_idx -= 1
                    else:
                        # Hit actual code (likely previous field), stop
                        break

                # Skip fields marked with #[serde(skip)]
                if not has_skip:
                    field_docs[field_name] = FieldDoc(
                        name=field_name,
                        doc=field_doc,
                        field_type=field_type,
                        has_default=has_default,
                        aliases=aliases
                    )

    if not field_docs:
        return None

    return StructDoc(name=struct_name, struct_doc=struct_doc, fields=field_docs)


def find_all_struct_docs(src_dir: Path) -> Dict[str, StructDoc]:
    """Find and parse all transformation struct files."""
    transformations_dir = src_dir / 'transformations'
    all_docs = {}

    # Parse transformation module files
    for module_dir in transformations_dir.iterdir():
        if module_dir.is_dir():
            for rust_file in module_dir.glob('*.rs'):
                struct_doc = parse_struct_file(rust_file)
                if struct_doc:
                    all_docs[struct_doc.name] = struct_doc

    # Parse top-level transformation files
    for rust_file in transformations_dir.glob('*.rs'):
        if rust_file.stem not in ['mod', 'prelude']:
            struct_doc = parse_struct_file(rust_file)
            if struct_doc:
                all_docs[struct_doc.name] = struct_doc

    # Parse config files
    config_dir = src_dir / 'config'
    for rust_file in config_dir.glob('*.rs'):
        if rust_file.stem in ['input', 'output', 'options']:
            struct_doc = parse_struct_file(rust_file)
            if struct_doc:
                all_docs[struct_doc.name] = struct_doc

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


def get_example_value(field: FieldDoc) -> str:
    """Generate a reasonable example value for a field."""
    name_lower = field.name.lower()

    if 'label' in name_lower:
        return '"mytag"'
    if 'segment' in name_lower:
        return '"read1"'
    if 'search' in name_lower or 'query' in name_lower:
        return '"AGTC"'
    if 'pattern' in name_lower:
        return '"pattern"'
    if 'replacement' in name_lower:
        return '"$1"'
    if 'base' in name_lower and field.field_type == 'u8':
        return '"A"'
    if 'seed' in name_lower:
        return '42'
    if 'file' in name_lower:
        return '["file.fastq"]' if 'Vec' in field.field_type else '"file.fastq"'
    if 'regions' in name_lower and 'Vec' in field.field_type:
        return '[{segment="read1", start=0, length=8}]'
    if 'anchor' in name_lower:
        return "'Anywhere'"
    if name_lower == 'n':
        return '100'
    if name_lower == 'p':
        return '0.5'
    if 'mismatch' in name_lower:
        return '1'
    if 'length' in name_lower or 'count' in name_lower or 'size' in name_lower:
        return '10'

    # Type-based defaults
    if 'String' in field.field_type or 'BString' in field.field_type:
        return '"value"'
    elif 'usize' in field.field_type:
        return '10'
    elif field.field_type in ['f32', 'f64']:
        return '0.5'
    elif field.field_type == 'bool':
        return 'true'
    elif 'Vec<' in field.field_type:
        return '[]'
    else:
        return '"value"'


def generate_toml_fields(struct_doc: StructDoc, indent: str = "#    ") -> List[str]:
    """Generate TOML field lines with inline documentation."""
    lines = []

    for field_name, field in struct_doc.fields.items():
        if field_name in ['segment_index']:  # Skip internal fields
            continue

        example_value = get_example_value(field)
        optional_marker = " (optional)" if field.has_default else ""

        line = f"{indent}{field.name} = {example_value} # {field.doc}{optional_marker}"
        lines.append(line)

    return lines


def generate_markdown_fields(struct_doc: StructDoc) -> List[str]:
    """Generate markdown parameter documentation."""
    lines = []

    for field_name, field in struct_doc.fields.items():
        if field_name in ['segment_index']:  # Skip internal fields
            continue

        # Format: - **field_name**: Documentation text (optional)
        optional = " *(optional)*" if field.has_default else ""
        aliases_text = f" (alias: `{field.aliases[0]}`)" if field.aliases else ""
        lines.append(f"- **`{field.name}`**{aliases_text}: {field.doc}{optional}")

    return lines


def update_template_toml(
    template_path: Path,
    struct_docs: Dict[str, StructDoc]
) -> Tuple[str, int]:
    """Update template.toml by replacing marked sections."""
    lines = template_path.read_text().splitlines()
    output_lines = []
    i = 0
    updates = 0

    while i < len(lines):
        line = lines[i]
        # Match with optional # prefix and spaces
        start_match = re.match(r'^#?\s*AUTO-GENERATED-START:\s*(\w+)', line)

        if start_match:
            struct_name = start_match.group(1)
            output_lines.append(line)

            # Find end marker
            end_idx = i + 1
            while end_idx < len(lines) and not re.match(r'^#?\s*AUTO-GENERATED-END', lines[end_idx]):
                end_idx += 1

            # Generate new content
            if struct_name in struct_docs:
                field_lines = generate_toml_fields(struct_docs[struct_name])
                output_lines.extend(field_lines)
                updates += 1
            else:
                # Keep existing content
                for j in range(i + 1, end_idx):
                    output_lines.append(lines[j])

            if end_idx < len(lines):
                output_lines.append(lines[end_idx])
                i = end_idx + 1
            else:
                i += 1
        else:
            output_lines.append(line)
            i += 1

    return '\n'.join(output_lines), updates


def update_markdown_file(
    md_path: Path,
    struct_docs: Dict[str, StructDoc],
    action_mapping: Dict[str, str]
) -> Tuple[str, int]:
    """Update markdown file by replacing marked sections."""
    content = md_path.read_text()
    updates = 0

    # Find action name from filename
    action_name = md_path.stem
    struct_name = action_mapping.get(action_name)

    if not struct_name or struct_name not in struct_docs:
        return content, 0

    struct_doc = struct_docs[struct_name]

    # Replace marked sections
    def replace_marked_section(match):
        nonlocal updates
        marker_name = match.group(1)

        if marker_name == struct_name:
            updates += 1
            field_lines = generate_markdown_fields(struct_doc)
            return f"<!-- AUTO-GENERATED-START: {marker_name} -->\n" + '\n'.join(field_lines) + "\n<!-- AUTO-GENERATED-END -->"
        else:
            return match.group(0)

    pattern = r'<!--\s*AUTO-GENERATED-START:\s*(\w+)\s*-->.*?<!--\s*AUTO-GENERATED-END\s*-->'
    updated_content = re.sub(pattern, replace_marked_section, content, flags=re.DOTALL)

    return updated_content, updates


def update_all_markdown_files(
    docs_dir: Path,
    struct_docs: Dict[str, StructDoc],
    action_mapping: Dict[str, str]
) -> int:
    """Update all markdown files in documentation directory."""
    total_updates = 0

    if not docs_dir.exists():
        print(f"Warning: Documentation directory not found: {docs_dir}")
        return 0

    for md_file in docs_dir.rglob('*.md'):
        if md_file.name.startswith('_'):
            continue

        updated_content, updates = update_markdown_file(md_file, struct_docs, action_mapping)

        if updates > 0:
            md_file.write_text(updated_content)
            print(f"  Updated {md_file.relative_to(docs_dir.parent.parent)}: {updates} sections")
            total_updates += updates

    return total_updates


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Update documentation from Rust docstrings')
    parser.add_argument('--template', action='store_true', help='Update template.toml')
    parser.add_argument('--markdown', action='store_true', help='Update markdown files')
    parser.add_argument('--all', action='store_true', help='Update everything')
    args = parser.parse_args()

    # Default to --all if nothing specified
    if not (args.template or args.markdown or args.all):
        args.all = True

    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    src_dir = repo_root / 'mbf-fastq-processor' / 'src'
    template_path = src_dir / 'template.toml'
    docs_dir = repo_root / 'docs' / 'content' / 'docs' / 'reference'

    print("Extracting documentation from Rust source files...")
    struct_docs = find_all_struct_docs(src_dir)
    print(f"Found documentation for {len(struct_docs)} structs\n")

    action_mapping = get_transformation_mapping(src_dir)

    if args.all or args.template:
        if template_path.exists():
            print("Updating template.toml...")
            updated_content, updates = update_template_toml(template_path, struct_docs)

            if updates > 0:
                template_path.write_text(updated_content)
                print(f"✓ Updated {updates} sections in template.toml\n")
            else:
                print("✓ No marked sections found in template.toml\n")
        else:
            print(f"Warning: template.toml not found at {template_path}\n")

    if args.all or args.markdown:
        print("Updating markdown documentation...")
        total_updates = update_all_markdown_files(docs_dir, struct_docs, action_mapping)
        if total_updates > 0:
            print(f"\n✓ Updated {total_updates} sections across markdown files")
        else:
            print("✓ No marked sections found in markdown files")


if __name__ == '__main__':
    main()
