#!/usr/bin/env python3
"""
Generate template.toml from Rust docstrings.

This script extracts field documentation from Rust source files and generates
the template.toml file with up-to-date parameter descriptions.
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass


@dataclass
class FieldDoc:
    """Documentation for a struct field."""
    name: str
    doc: str
    field_type: str
    has_default: bool = False
    aliases: List[str] = None

    def __post_init__(self):
        if self.aliases is None:
            self.aliases = []


@dataclass
class StructDoc:
    """Documentation for a transformation struct."""
    name: str
    module: str
    doc: str
    fields: List[FieldDoc]


def extract_doc_comment(lines: List[str], start_idx: int) -> Tuple[str, int]:
    """
    Extract doc comments (///) before a field/struct.
    Returns (comment_text, line_index_after_comments).
    """
    docs = []
    idx = start_idx

    # Go backwards to find all doc comments
    while idx >= 0:
        line = lines[idx].strip()
        if line.startswith('///'):
            # Remove /// and leading/trailing whitespace
            doc_line = line[3:].strip()
            docs.insert(0, doc_line)
            idx -= 1
        elif line.startswith('#[') or line == '' or line.startswith('//'):
            # Skip attributes and regular comments
            idx -= 1
        else:
            break

    return ' '.join(docs), idx + 1


def parse_struct_file(file_path: Path) -> Optional[StructDoc]:
    """Parse a Rust struct file and extract documentation."""
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
    struct_doc, _ = extract_doc_comment(lines, struct_line_idx - 1)

    # Find all public fields
    fields = []
    in_struct = False
    brace_count = 0

    for idx in range(struct_line_idx, len(lines)):
        line = lines[idx]

        # Track braces
        brace_count += line.count('{') - line.count('}')
        if struct_line_idx < idx and brace_count <= 0:
            break

        # Look for public fields
        field_match = re.match(r'\s*pub\s+(\w+):\s*(.+?)(?:,|$)', line)
        if field_match:
            field_name = field_match.group(1)
            field_type = field_match.group(2).strip().rstrip(',')

            # Extract field documentation
            field_doc, _ = extract_doc_comment(lines, idx - 1)

            # Check for default attribute (looking backwards from field)
            has_default = False
            aliases = []
            for attr_idx in range(max(0, idx - 10), idx):
                attr_line = lines[attr_idx].strip()
                if '#[serde(default)]' in attr_line or 'serde(default' in attr_line:
                    has_default = True
                # Look for aliases
                alias_match = re.search(r'alias\s*=\s*"(\w+)"', attr_line)
                if alias_match:
                    aliases.append(alias_match.group(1))

            if field_doc:  # Only add fields that have documentation
                fields.append(FieldDoc(
                    name=field_name,
                    doc=field_doc,
                    field_type=field_type,
                    has_default=has_default,
                    aliases=aliases
                ))

    if not fields:
        return None

    # Determine module from file path
    module = file_path.parent.name if file_path.parent.name != 'transformations' else file_path.stem

    return StructDoc(
        name=struct_name,
        module=module,
        doc=struct_doc,
        fields=fields
    )


def find_all_transformation_structs(src_dir: Path) -> Dict[str, StructDoc]:
    """Find and parse all transformation struct files."""
    transformations_dir = src_dir / 'transformations'
    structs = {}

    # Parse module files
    for module_dir in transformations_dir.iterdir():
        if module_dir.is_dir():
            for rust_file in module_dir.glob('*.rs'):
                struct_doc = parse_struct_file(rust_file)
                if struct_doc:
                    structs[struct_doc.name] = struct_doc

    # Also parse top-level transformation files
    for rust_file in transformations_dir.glob('*.rs'):
        if rust_file.stem not in ['mod', 'prelude']:
            struct_doc = parse_struct_file(rust_file)
            if struct_doc:
                structs[struct_doc.name] = struct_doc

    # Parse config structs
    config_dir = src_dir / 'config'
    for rust_file in config_dir.glob('*.rs'):
        if rust_file.stem in ['input', 'output', 'options']:
            struct_doc = parse_struct_file(rust_file)
            if struct_doc:
                structs[struct_doc.name] = struct_doc

    return structs


def generate_field_comment(field: FieldDoc, indent: int = 4) -> str:
    """Generate a TOML comment for a field."""
    indent_str = ' ' * indent
    comment_lines = []

    # Split long comments into multiple lines
    words = field.doc.split()
    current_line = []
    current_length = 0
    max_length = 80 - len(indent_str) - 2  # Account for "# "

    for word in words:
        if current_length + len(word) + 1 > max_length and current_line:
            comment_lines.append(f"{indent_str}# {' '.join(current_line)}")
            current_line = [word]
            current_length = len(word)
        else:
            current_line.append(word)
            current_length += len(word) + 1

    if current_line:
        comment_lines.append(f"{indent_str}# {' '.join(current_line)}")

    return '\n'.join(comment_lines)


def generate_toml_section(struct_doc: StructDoc, action_name: Optional[str] = None) -> str:
    """Generate a TOML section with documentation for a transformation."""
    lines = []

    # Add section header
    section_name = action_name or struct_doc.name
    lines.append(f"# ==== {section_name} ====")

    # Add struct-level documentation if available
    if struct_doc.doc:
        lines.append(f"# {struct_doc.doc}")

    # Add step block
    lines.append("# [[step]]")
    if action_name:
        lines.append(f"#    action = \"{action_name}\"")

    # Add field documentation
    for field in struct_doc.fields:
        if field.name not in ['segment_index']:  # Skip internal fields
            field_comment = generate_field_comment(field)
            lines.append(field_comment)

            # Add example value
            example_value = get_example_value(field)
            optional_marker = " # (optional)" if field.has_default else ""
            alias_info = f" or {field.aliases[0]}" if field.aliases else ""
            lines.append(f"#    {field.name}{alias_info} = {example_value}{optional_marker}")

    lines.append("")
    return '\n'.join(lines)


def get_example_value(field: FieldDoc) -> str:
    """Get an example value for a field based on its type."""
    # Common patterns
    if 'String' in field.field_type or field.name.endswith('label'):
        return '"mytag"'
    elif field.field_type == 'usize' or 'usize' in field.field_type:
        return '100'
    elif field.field_type == 'f32' or field.field_type == 'f64':
        return '0.5'
    elif field.field_type == 'bool':
        return 'true'
    elif 'Vec<' in field.field_type:
        return '["item1", "item2"]'
    elif 'Segment' in field.field_type:
        return '"read1"'
    else:
        return '"value"'


def main():
    """Main entry point."""
    # Get source directory
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    src_dir = repo_root / 'mbf-fastq-processor' / 'src'
    template_path = src_dir / 'template.toml'

    if not src_dir.exists():
        print(f"Error: Source directory not found: {src_dir}", file=sys.stderr)
        sys.exit(1)

    print("Extracting documentation from Rust source files...")
    structs = find_all_transformation_structs(src_dir)
    print(f"Found {len(structs)} documented structs")

    # Read current template.toml to preserve structure
    if template_path.exists():
        current_template = template_path.read_text()
    else:
        current_template = ""

    # For now, just print what we extracted (we'll implement injection next)
    print("\nExtracted structures:")
    for name, struct in sorted(structs.items()):
        print(f"  - {name}: {len(struct.fields)} fields")
        if struct.fields:
            print(f"      Fields: {', '.join(f.name for f in struct.fields[:5])}")

    print(f"\nGenerated documentation can be inserted into {template_path}")
    print("Next step: Implement placeholder replacement in template.toml")


if __name__ == '__main__':
    main()
