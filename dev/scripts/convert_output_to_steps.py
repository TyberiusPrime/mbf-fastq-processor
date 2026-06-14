#!/usr/bin/env python3
"""Convert a legacy ``[output]`` section into explicit Output* pipeline steps.

The legacy configuration drives all file/report output from a single
``[output]`` section. This script rewrites a fastqrab TOML so that the work is
expressed as steps appended at the end of the pipeline:

  * ``format = "fastq" | "fasta" | "bam"``  -> ``OutputFASTQ`` / ``OutputFASTA`` / ``OutputBAM``
  * ``report_json`` / ``report_html``       -> ``OutputReport``

The slimmed ``[output]`` section keeps only ``prefix`` (still required),
``ix_separator`` and ``report_timing`` (which remain global / orthogonal).

The rest of the document (``[input]``, existing ``[[step]]`` blocks,
``[barcodes.*]``, ``[options]`` ...) is preserved verbatim; only the
``[output]`` section is replaced and the new steps are appended.

Usage::

    convert_output_to_steps.py input.toml            # prints to stdout
    convert_output_to_steps.py input.toml -o out.toml
    convert_output_to_steps.py input.toml --in-place
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# Fields that stay in the (slimmed) [output] section.
KEPT_OUTPUT_FIELDS = ("prefix", "ix_separator", "report_timing")

# Mapping of [output] keys -> record-output step keys, per format.
TEXT_FIELDS = (
    "output",
    "interleave",
    "stdout",
    "suffix",
    "compression",
    "compression_level",
    "compression_threads",
    "chunksize",
    "output_hash_uncompressed",
    "output_hash_compressed",
)
BAM_FIELDS = (
    "output",
    "interleave",
    "suffix",
    "compression_level",
    "compression_threads",
    "chunksize",
    "output_hash_compressed",
    "bam",
)
FORMAT_TO_ACTION = {
    "fastq": ("OutputFASTQ", TEXT_FIELDS),
    "fasta": ("OutputFASTA", TEXT_FIELDS),
    "bam": ("OutputBAM", BAM_FIELDS),
}


def toml_value(value) -> str:
    """Serialize a Python value as a TOML value (scalars, arrays, inline tables)."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, int) or isinstance(value, float):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(v) for v in value) + "]"
    if isinstance(value, dict):
        inner = ", ".join(f"{k} = {toml_value(v)}" for k, v in value.items())
        return "{ " + inner + " }"
    raise TypeError(f"Cannot serialize value of type {type(value)!r}: {value!r}")


def render_step(action: str, fields: dict) -> str:
    lines = ["[[step]]", f'    action = "{action}"']
    for key, value in fields.items():
        lines.append(f"    {key} = {toml_value(value)}")
    return "\n".join(lines)


def build_steps(output: dict) -> list[str]:
    """Build the rendered Output* step blocks from a parsed [output] table."""
    steps: list[str] = []

    fmt = str(output.get("format", "fastq")).lower()
    if fmt not in ("none",):
        if fmt not in FORMAT_TO_ACTION:
            raise SystemExit(f"Unknown output.format {fmt!r}")
        action, allowed = FORMAT_TO_ACTION[fmt]
        fields = {k: output[k] for k in allowed if k in output}
        steps.append(render_step(action, fields))

    report_json = bool(output.get("report_json", False))
    report_html = bool(output.get("report_html", False))
    if report_json or report_html:
        steps.append(
            render_step("OutputReport", {"json": report_json, "html": report_html})
        )

    return steps


def render_slim_output(output: dict) -> str:
    lines = ["[output]"]
    for key in KEPT_OUTPUT_FIELDS:
        if key in output:
            lines.append(f"    {key} = {toml_value(output[key])}")
    return "\n".join(lines)


# A top-level table or array-of-tables header, e.g. "[output]" / "[[step]]" /
# "[output.bam]". Captures the first dotted component of the table path.
_HEADER_RE = re.compile(r"^\s*\[\[?\s*([A-Za-z0-9_-]+)")


def replace_output_section(raw: str, slim_output: str) -> str:
    """Replace the whole ``[output]`` section (incl. ``[output.*]`` subtables)
    in ``raw`` with ``slim_output``. Everything else is left untouched."""
    lines = raw.splitlines()
    start = None
    for i, line in enumerate(lines):
        m = _HEADER_RE.match(line)
        if m and m.group(1) == "output" and line.strip().rstrip().startswith("[output"):
            # Only the top-level [output] header starts the section.
            if re.match(r"^\s*\[output\]", line):
                start = i
                break
    if start is None:
        raise SystemExit("No [output] section found.")

    end = len(lines)
    for j in range(start + 1, len(lines)):
        m = _HEADER_RE.match(lines[j])
        if m and m.group(1) != "output":
            end = j
            break

    new_lines = lines[:start] + slim_output.splitlines() + lines[end:]
    return "\n".join(new_lines) + ("\n" if raw.endswith("\n") else "")


def convert(raw: str) -> str:
    data = tomllib.loads(raw)
    output = data.get("output")
    if output is None:
        raise SystemExit("Config has no [output] section; nothing to convert.")

    steps = build_steps(output)
    result = replace_output_section(raw, render_slim_output(output))

    if steps:
        if not result.endswith("\n"):
            result += "\n"
        result += "\n# --- Output steps (migrated from [output]) ---\n"
        result += "\n\n".join(steps) + "\n"
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="Input TOML file")
    parser.add_argument("-o", "--output", type=Path, help="Write result to this file")
    parser.add_argument(
        "--in-place", action="store_true", help="Rewrite the input file in place"
    )
    args = parser.parse_args(argv)

    raw = args.input.read_text()
    try:
        converted = convert(raw)
    except:
        print(args)
        raise

    if args.in_place:
        args.input.write_text(converted)
    elif args.output:
        args.output.write_text(converted)
    else:
        sys.stdout.write(converted)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
