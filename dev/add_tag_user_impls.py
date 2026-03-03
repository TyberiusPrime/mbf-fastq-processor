#!/usr/bin/env python3
"""
Add empty TagUser implementations for all PartialTaggedVariant<PartialXxx> types
that are missing the TagUser trait impl. Places them before the 'impl Step' block.
"""

import subprocess
import re
import sys
from pathlib import Path

SRC_DIR = Path(__file__).parent.parent / "mbf-fastq-processor" / "src"

def get_errors():
    result = subprocess.run(
        ["cargo", "build"],
        capture_output=True, text=True,
        cwd=Path(__file__).parent.parent
    )
    output = result.stderr + result.stdout
    pattern = r'`PartialTaggedVariant<([^`>]+(?:>[^`]*)?)>: transformations::TagUser` is not satisfied'
    types = re.findall(pattern, output)
    return list(dict.fromkeys(types))  # deduplicate while preserving order

def partial_to_concrete(partial_type):
    """Convert Partial_Xxx -> _Xxx, PartialXxx -> Xxx, Box<Partial_Xxx> -> Box<_Xxx>"""
    if partial_type.startswith("Box<Partial"):
        # Box<Partial_ReportCount> -> Box<_ReportCount>
        # Box<PartialEvalExpression> -> Box<EvalExpression>
        inner = partial_type[len("Box<Partial"):]  # e.g. "_ReportCount>" or "EvalExpression>"
        inner = inner.rstrip(">")
        return f"Box<{inner}>"
    elif partial_type.startswith("Partial"):
        return partial_type[len("Partial"):]
    return partial_type

def find_file_for_type(concrete_type):
    """Find the .rs file that has 'impl Step for ConcreteType {'"""
    # Use word-boundary-like matching with { to avoid substring matches
    search = f"impl Step for {concrete_type} {{"
    result = subprocess.run(
        ["grep", "-rl", search, str(SRC_DIR)],
        capture_output=True, text=True
    )
    files = [f.strip() for f in result.stdout.strip().split("\n") if f.strip()]
    return files

def check_already_impl(filepath, partial_type):
    """Check if TagUser impl already exists for this type in this file"""
    content = Path(filepath).read_text()
    return f"impl TagUser for PartialTaggedVariant<{partial_type}>" in content

def remove_impl(filepath, partial_type):
    """Remove a TagUser impl from a file (for fixing misplaced ones)"""
    content = Path(filepath).read_text()
    pattern = f"impl TagUser for PartialTaggedVariant<{re.escape(partial_type)}> {{}}\\n\\n"
    new_content = re.sub(pattern, "", content)
    if new_content == content:
        # Try without trailing newline
        pattern = f"impl TagUser for PartialTaggedVariant<{re.escape(partial_type)}> {{}}\\n"
        new_content = re.sub(pattern, "", content)
    if new_content != content:
        Path(filepath).write_text(new_content)
        return True
    return False

def add_impl(filepath, partial_type, concrete_type):
    """Add empty TagUser impl before 'impl Step for ConcreteType {'"""
    content = Path(filepath).read_text()

    impl_step_pattern = f"impl Step for {re.escape(concrete_type)} {{"
    match = re.search(impl_step_pattern, content)
    if not match:
        print(f"  WARNING: Could not find 'impl Step for {concrete_type} {{' in {filepath}")
        return False

    pos = match.start()
    impl_text = f"impl TagUser for PartialTaggedVariant<{partial_type}> {{}}\n\n"
    new_content = content[:pos] + impl_text + content[pos:]
    Path(filepath).write_text(new_content)
    return True

def main():
    print("Running cargo build to find missing TagUser impls...")
    types = get_errors()

    if not types:
        print("No missing TagUser impls found!")
        return

    print(f"Found {len(types)} missing impls:")
    for t in types:
        print(f"  PartialTaggedVariant<{t}>")
    print()

    fixed = 0
    skipped = 0
    for partial_type in types:
        concrete_type = partial_to_concrete(partial_type)
        files = find_file_for_type(concrete_type)

        if not files:
            print(f"  ERROR: No file found for concrete type '{concrete_type}' (from '{partial_type}')")
            skipped += 1
            continue

        if len(files) > 1:
            print(f"  WARNING: Multiple files for '{concrete_type}': {files}")

        filepath = files[0]

        if check_already_impl(filepath, partial_type):
            print(f"  SKIP: {partial_type} already has TagUser impl in {Path(filepath).name}")
            skipped += 1
            continue

        success = add_impl(filepath, partial_type, concrete_type)
        if success:
            print(f"  ADDED: impl TagUser for PartialTaggedVariant<{partial_type}> in {Path(filepath).name}")
            fixed += 1
        else:
            skipped += 1

    print(f"\nDone: {fixed} impls added, {skipped} skipped/failed")

    if fixed > 0:
        print("\nRunning cargo fmt...")
        subprocess.run(["cargo", "fmt"], cwd=Path(__file__).parent.parent)
        print("Done formatting.")

if __name__ == "__main__":
    main()
