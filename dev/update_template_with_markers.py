#!/usr/bin/env python3
"""
Update template.toml by replacing marked sections with Rust docstrings.

This script looks for AUTO-GENERATED markers in template.toml and replaces
the content between them with fresh documentation from Rust source files.

Marker format:
    # AUTO-GENERATED-START: StructName
    ... content will be replaced ...
    # AUTO-GENERATED-END

Usage:
    python3 dev/update_template_with_markers.py
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
    field_type: str
    has_default: bool = False
    aliases: List[str] = dataclass_field(default_factory=list)


@dataclass
class StructDoc:
    """Documentation for a transformation struct."""
    name: str
    fields: Dict[str, FieldDoc]


def extract_doc_comment(lines: List[str], start_idx: int) -> str:
    """Extract doc comments (///) before a field/struct."""
    docs = []
    idx = start_idx

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


def parse_struct_file(file_path: Path) -> Optional[StructDoc]:
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

    # Find all public fields
    for idx in range(struct_line_idx, len(lines)):
        line = lines[idx]
        brace_count += line.count('{') - line.count('}')

        if struct_line_idx < idx and brace_count <= 0:
            break

        # Look for public fields
        field_match = re.match(r'\s*pub\s+(\w+):\s*(.+?)(?:,|$)', line)
        if field_match:
            field_name = field_match.group(1)
            field_type = field_match.group(2).strip().rstrip(',')
            field_doc = extract_doc_comment(lines, idx - 1)

            if field_doc:
                # Check for default and aliases
                has_default = False
                aliases = []
                for attr_idx in range(max(0, idx - 10), idx):
                    attr_line = lines[attr_idx].strip()
                    if '#[serde(default)]' in attr_line or 'serde(default' in attr_line:
                        has_default = True
                    alias_match = re.search(r'alias\s*=\s*"(\w+)"', attr_line)
                    if alias_match:
                        aliases.append(alias_match.group(1))

                field_docs[field_name] = FieldDoc(
                    name=field_name,
                    doc=field_doc,
                    field_type=field_type,
                    has_default=has_default,
                    aliases=aliases
                )

    if not field_docs:
        return None

    return StructDoc(name=struct_name, fields=field_docs)


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


def generate_field_lines(field: FieldDoc, indent: str = "#    ") -> List[str]:
    """Generate TOML lines for a field with its documentation."""
    lines = []

    # Add the field with inline comment
    example_value = get_example_value(field)
    optional_marker = " (optional)" if field.has_default else ""

    # Format: field = value # documentation (optional)
    line = f"{indent}{field.name} = {example_value} # {field.doc}{optional_marker}"
    lines.append(line)

    return lines


def get_example_value(field: FieldDoc) -> str:
    """Generate a reasonable example value for a field."""
    name_lower = field.name.lower()

    # Name-based heuristics
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
    if 'direction' in name_lower:
        return '"Start"'
    if 'encoding' in name_lower:
        return '"Illumina1.8"'
    if 'format' in name_lower:
        return '"Fastq"'
    if 'compression' in name_lower and 'level' not in name_lower:
        return '"Gzip"'
    if 'separator' in name_lower:
        return '"_"'
    if 'char' in name_lower:
        return "' '"
    if name_lower == 'n':
        return '100'
    if name_lower == 'p':
        return '0.5'
    if 'prefix' in name_lower or 'suffix' in name_lower or 'infix' in name_lower:
        return '"output"'

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
        return '["item1", "item2"]'
    else:
        return '"value"'


def update_template_with_markers(
    template_path: Path,
    struct_docs: Dict[str, StructDoc],
    action_mapping: Dict[str, str]
) -> str:
    """
    Update template.toml by replacing content between AUTO-GENERATED markers.

    Markers format:
        # AUTO-GENERATED-START: StructName
        ... replaced content ...
        # AUTO-GENERATED-END
    """
    lines = template_path.read_text().splitlines()
    output_lines = []
    i = 0

    while i < len(lines):
        line = lines[i]

        # Check for auto-generated marker
        start_match = re.match(r'#\s*AUTO-GENERATED-START:\s*(\w+)', line)

        if start_match:
            struct_name = start_match.group(1)
            output_lines.append(line)  # Keep the marker

            # Find the end marker
            end_idx = i + 1
            while end_idx < len(lines):
                if re.match(r'#\s*AUTO-GENERATED-END', lines[end_idx]):
                    break
                end_idx += 1

            # Generate new content
            if struct_name in struct_docs:
                struct_doc = struct_docs[struct_name]

                # Generate field lines
                for field_name, field in struct_doc.fields.items():
                    if field_name not in ['segment_index']:  # Skip internal fields
                        field_lines = generate_field_lines(field)
                        output_lines.extend(field_lines)
            else:
                print(f"Warning: No documentation found for struct {struct_name}")
                # Keep existing content if struct not found
                for j in range(i + 1, end_idx):
                    output_lines.append(lines[j])

            # Add the end marker
            if end_idx < len(lines):
                output_lines.append(lines[end_idx])
                i = end_idx + 1
            else:
                print(f"Warning: No end marker found for {struct_name}")
                i += 1
        else:
            output_lines.append(line)
            i += 1

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

    print("\nUpdating template.toml with marker-based replacement...")
    original_content = template_path.read_text()

    # Count existing markers
    marker_count = original_content.count('AUTO-GENERATED-START:')
    print(f"Found {marker_count} auto-generated sections to update")

    updated_content = update_template_with_markers(template_path, struct_docs, action_mapping)

    # Count changes
    if updated_content != original_content:
        template_path.write_text(updated_content)
        print(f"\n✓ Template updated at {template_path}")
    else:
        print("\n✓ No changes needed")


if __name__ == '__main__':
    main()
