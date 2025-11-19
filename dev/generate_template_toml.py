#!/usr/bin/env python3
"""
Generate template.toml from Rust source docstrings.

This script extracts field documentation from Rust source files and generates
a comprehensive template.toml file. The Rust docstrings are the source of truth.

Usage:
    python3 dev/generate_template_toml.py

The generated template.toml will be written to mbf-fastq-processor/src/template.toml
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

            # Check for default attribute and aliases
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

    return structs


def get_transformation_mapping(src_dir: Path) -> Dict[str, str]:
    """
    Map action names to struct names by parsing transformations.rs enum.
    Returns dict like {"ExtractIUPAC": "IUPAC", "ExtractRegion": "Region", ...}
    """
    transformations_file = src_dir / 'transformations.rs'
    content = transformations_file.read_text()

    # Find the Transformation enum
    enum_match = re.search(r'pub enum Transformation \{(.*?)\n\}', content, re.DOTALL)
    if not enum_match:
        return {}

    enum_content = enum_match.group(1)
    mapping = {}

    # Parse enum variants: ActionName(module::StructName),
    for line in enum_content.splitlines():
        line = line.strip()
        # Match patterns like: ExtractIUPAC(extract::IUPAC),
        match = re.match(r'([A-Z]\w+)\(\w+::(\w+)\)', line)
        if match:
            action_name = match.group(1)
            struct_name = match.group(2)
            mapping[action_name] = struct_name

    return mapping


def generate_template_header() -> str:
    """Generate the header section of template.toml."""
    return """# mbf-fastq-processor Configuration Template
# AUTO-GENERATED from Rust source docstrings
# DO NOT EDIT - Regenerate with: python3 dev/generate_template_toml.py

# This template includes all available transformation steps with explanations
# it is therefore very comprehensive and long.

# To get started, use mbf-fastq-processor cookbooks instead.

# == Input ==

[input]
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd'] # one is required
    #read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd'] # (optional)
    #index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd'] # (optional)
    #index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd'] # (optional)
    # interleaved = [...] # Activates interleaved reading, see below

## A mapping from segment names to files to read.
## Compression is auto-detected.
## File format is auto-detected (see options below).
## The number of files per segment must match.

[input.options]
    # fasta_fake_quality = 30      # required for FASTA inputs: synthetic Phred score (0-93)
    # bam_include_mapped = true    # required for BAM inputs: keep reads with alignments
    # bam_include_unmapped = true  # required for BAM inputs: keep reads without alignments
    # read_comment_char = ' '      # defaults to ' '. The character separating read name from the 'read comment'.

## Interleaved mode:
## specify exactly one key=[files] value, and set interleaved to ['read1','read2',...]
## Further steps use the segment names from interleaved

## By default, (paired) input reads are spot checked for their names matching.
## see options.spot_check_read_pairing at the end of this example.

# == Output ==

[output]
     prefix = "output" # files get named {prefix}_{segment}{suffix}, e.g. "output_read1.fq.gz"
     format = "Fastq" # (optional) output format, defaults to 'Fastq'
                      # Valid values are: Fastq, Fasta, BAM and None (for no sequence output, just reports)
     compression = "Gzip" # (optional), defaults to 'uncompressed'
                     # Valid values are uncompressed, Gzip, Zstd.
#     suffix = ".fq.gz" # optional, determined by the format if left off.
#     compression_level = 6 # optional compression level for gzip (0-9) or zstd (1-22)
                          # defaults: gzip=6, zstd=5

     report_json = true # (optional) write a json report file ($prefix.json)?
     report_html = true # (optional) write an interactive html report file ($prefix.html)?
     report_timing = true # (optional) write timing statistics to json file ($prefix_timing.json)?

#     stdout = false # write read1 to stdout, do not produce other fastq files.
#                    # sets interleave to true (if Read2 is in input),
#                    # format to Raw
#                    # You still need to set a prefix for
#                    # Reports/keep_index/Inspect/QuantifyRegion(s)
#                    # Incompatible with a Progress Transform that's logging to stdout
#
#     interleave = false # (optional) interleave fastq output, producing
#                        # only a single output file for read1/read2
#                        # (with infix _interleaved instead of '_1', e.g. 'output_interleaved.fq.gz')
#     keep_index = false # (optional) write index to files as well?
#                        # (independent of the interleave setting. )
#     output_hash_uncompressed = false # (optional) write a {prefix}_{1|2|i1|i2}.uncompressed.sha256
#                                    # with a hexdigest of the uncompressed data's sha256,
#                                    # similar to what sha256sum would do on the raw FASTQ
#     output_hash_compressed = false   # (optional) write a {prefix}_{1|2|i1|i2}.compressed.sha256
#                                    # with a hexdigest of the compressed output file's sha256,
#                                    # allowing verification with sha256sum on the actual output files
#     output = ["read1", "read2"] # (optional) which segments to write. Defaults to all segments defined in [input]. Set to empty list to suppress output. (Equivalent to `format="None`")
#     ix_separator = "_" # (optional, default '_') separator inserted between prefix, infix, and segment names
#     Chunksize = 1_000_000 # (optional) maximum number of molecules per output file. When set, chunk indexes are appended to filenames.
#


# == Tagging ==

# Extract data from / about sequences.
# Tags get temporarily stored in memory under a 'label'
# and can then be used in other steps.

# There are three kinds of tags,
#  - location based string tags (think search query results)
#  - numeric tags (e.g. length, GC content).
#  - boolean tags (e.g. is this a duplicate?)
# All tags can be stored within the fastq or separately (see below)
# Filtering is available as FilterByTag, FilterByNumericTag, FilterByBoolTag

# === String tags ===

"""


def generate_transformation_section(
    action_name: str,
    struct_doc: StructDoc,
    example_values: Dict[str, str]
) -> str:
    """Generate a TOML section for a transformation with field docs from Rust."""
    lines = []

    # Section header
    lines.append(f"# ==== {action_name} ====")

    # Struct-level doc if available
    if struct_doc.doc:
        # Wrap struct doc as comment
        for line_text in struct_doc.doc.split('. '):
            if line_text.strip():
                lines.append(f"# {line_text.strip()}.")

    # Start step block
    lines.append("# [[step]]")
    lines.append(f"#    action = \"{action_name}\"")

    # Add fields with their documentation
    for field in struct_doc.fields:
        if field.name in ['segment_index']:  # Skip internal fields
            continue

        # Add field documentation as comment
        comment = f"#    # {field.doc}"
        lines.append(comment)

        # Add example value
        example = example_values.get(field.name, get_default_example(field))
        optional_marker = " # (optional)" if field.has_default else ""
        lines.append(f"#    {field.name} = {example}{optional_marker}")

    lines.append("")
    return '\n'.join(lines)


def get_default_example(field: FieldDoc) -> str:
    """Generate a reasonable example value for a field based on its type and name."""
    name_lower = field.name.lower()

    # Name-based heuristics
    if 'label' in name_lower:
        return '"mytag"'
    if 'segment' in name_lower:
        return '"read1"'
    if 'search' in name_lower or 'query' in name_lower or 'pattern' in name_lower:
        return '"AGTC"'
    if 'base' in name_lower and field.field_type == 'u8':
        return '"A"'
    if 'seed' in name_lower:
        return '42'
    if 'file' in name_lower:
        return '["file.fastq"]' if 'Vec' in field.field_type else '"file.fastq"'

    # Type-based defaults
    if 'String' in field.field_type:
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


def load_custom_examples() -> Dict[str, Dict[str, str]]:
    """
    Return custom example values for specific transformations.
    This preserves the carefully crafted examples from the original template.
    """
    return {
        "ExtractIUPAC": {
            "search": "'CTN'",
            "max_mismatches": "1",
            "anchor": "'Anywhere'",
            "segment": '"read1"',
            "out_label": '"mytag"',
        },
        "Head": {
            "n": "1000",
        },
        "Skip": {
            "n": "100",
        },
        # Add more as needed...
    }


def main():
    """Main entry point."""
    # Get paths
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

    # Get action name -> struct name mapping
    action_to_struct = get_transformation_mapping(src_dir)
    print(f"Found {len(action_to_struct)} transformations in enum")

    # Load custom examples
    custom_examples = load_custom_examples()

    # Generate template
    print("\nGenerating template.toml...")
    output = [generate_template_header()]

    # Generate sections for each transformation
    for action_name, struct_name in sorted(action_to_struct.items()):
        if struct_name in structs:
            struct_doc = structs[struct_name]
            examples = custom_examples.get(action_name, {})
            section = generate_transformation_section(action_name, struct_doc, examples)
            output.append(section)
        else:
            print(f"Warning: No struct documentation found for {action_name} ({struct_name})")

    # Add options section at the end
    output.append("""# == Options ==
# [options]
#   spot_check_read_pairing = true # Whether to spot check read pair names. See SpotCheckReadPairing step
#   thread_count = -1 # only for the steps supporting multi-core.
#   block_size = 10000 # how many reads per block?
#   buffer_size = 102400 # how many bytes of buffer. Will extend if we can't get block_size reads in there.
#   accept_duplicate_files = false # for testing purposes.
""")

    # Write output
    template_content = '\n'.join(output)
    template_path.write_text(template_content)
    print(f"\nTemplate written to {template_path}")
    print(f"Total size: {len(template_content)} bytes")


if __name__ == '__main__':
    main()
