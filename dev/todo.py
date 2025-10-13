#!/usr/bin/env python3
"""Utilities for managing todos stored in dev/issues."""

from __future__ import annotations

import argparse
import os
import re
import shlex
import shutil
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Iterable, List, Tuple

BASE_DIR = Path(__file__).resolve().parent
ISSUES_DIR = BASE_DIR / "issues"
FILENAME_RE = re.compile(r"^(?P<num>\d+)-(?P<slug>.+)\.md$")


def ensure_issues_dir() -> None:
    if not ISSUES_DIR.exists():
        raise SystemExit(f"issues directory missing: {ISSUES_DIR}")


def list_issue_files() -> List[Path]:
    ensure_issues_dir()
    return sorted(p for p in ISSUES_DIR.iterdir() if p.suffix == ".md")


def parse_number(path: Path) -> int:
    match = FILENAME_RE.match(path.name)
    if not match:
        raise ValueError(f"Could not parse issue number from {path.name}")
    return int(match.group("num"))


def slugify(title: str, used: Iterable[str] | None = None) -> str:
    text = title.lower()
    text = re.sub(r"[^a-z0-9]+", "-", text)
    text = re.sub(r"-+", "-", text)
    text = text.strip("-")
    if not text:
        text = "todo"
    if used is None:
        return text
    used_set = set(used)
    if text not in used_set:
        used_set.add(text)
        return text
    index = 2
    while f"{text}-{index}" in used_set:
        index += 1
    final = f"{text}-{index}"
    used_set.add(final)
    return final


def read_title(path: Path) -> str:
    for line in path.read_text().splitlines():
        if line.startswith("#"):
            return line.lstrip("#").strip()
    raise ValueError(f"No markdown heading found in {path}")


def command_add(args: argparse.Namespace) -> None:
    ensure_issues_dir()
    files = list_issue_files()
    next_num = max((parse_number(p) for p in files), default=0) + 1
    existing_slugs = [
        FILENAME_RE.match(p.name).group("slug")
        for p in files
        if FILENAME_RE.match(p.name)
    ]

    tmp_path = ISSUES_DIR / f"{next_num:03d}-draft.md"
    template = "status: open\n# Title\n\n"
    tmp_path.write_text(template)

    editor = os.environ.get("EDITOR", "vi")
    try:
        subprocess.run([editor, str(tmp_path)], check=True)
    except FileNotFoundError as exc:
        raise SystemExit(f"EDITOR not found: {editor}") from exc

    content = tmp_path.read_text()
    title = None
    for line in content.splitlines():
        if line.startswith("#"):
            title = line.lstrip("#").strip()
            break
    if not title:
        tmp_path.unlink(missing_ok=True)
        raise SystemExit("New todo requires a markdown heading (e.g. '# My TODO')")

    slug = slugify(title, existing_slugs)
    final_path = ISSUES_DIR / f"{next_num:04d}-{slug}.md"
    if final_path.exists():
        raise SystemExit(f"Target file already exists: {final_path}")
    tmp_path.rename(final_path)
    print(f"Created {final_path}")


def command_fix_names(args: argparse.Namespace) -> None:
    files = list_issue_files()
    renames: List[Tuple[Path, Path]] = []
    used_slugs: List[str] = []
    for index, path in enumerate(sorted(files, key=parse_number), start=1):
        title = read_title(path)
        slug = slugify(title, used_slugs)
        used_slugs.append(slug)
        target = path.with_name(f"{index:04d}-{slug}.md")
        if path == target:
            continue
        renames.append((path, target))

    if not renames:
        print("All todo file names already match their titles.")
        return

    temp_pairs: List[Tuple[Path, Path]] = []
    for source, target in renames:
        temp_name = source.with_name(f".tmp-{uuid.uuid4().hex}.md")
        source.rename(temp_name)
        temp_pairs.append((temp_name, target))

    for temp_path, target in temp_pairs:
        if target.exists():
            target.unlink()
        temp_path.rename(target)
        print(f"Renamed to {target.name}")


def command_search_status(args: argparse.Namespace) -> None:
    ensure_issues_dir()
    pattern = rf"^status:\s*{re.escape(args.status)}"
    rg = shutil.which("rg")
    if rg is None:
        raise SystemExit("ripgrep (rg) not found in PATH")
    result = subprocess.run(
        [rg, "--color", "never", pattern, "-l"],
        check=False,
        text=True,
        capture_output=True,
        cwd=ISSUES_DIR,
    )
    if result.returncode not in (0, 1):
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    if result.stdout:
        lines = result.stdout.strip().splitlines()
        lines = sorted(lines, key=lambda p: parse_number(Path(p)))
        # make them relative to the current directory
        lines = [os.path.relpath(ISSUES_DIR / x, start=Path(".")) for x in lines]
        lines = ["./" + x if "/" not in x else x for x in lines]
        fzf = shutil.which("fzf")
        if fzf is None:
            raise SystemExit("fzf not found in PATH")

        selection_input = "\n".join(lines) + "\n"
        selection = subprocess.run(
            [fzf],
            input=selection_input,
            text=True,
            capture_output=True,
            check=False,
        )
        if selection.returncode != 0:
            print("Selection cancelled.")
            return

        chosen = selection.stdout.strip()
        if not chosen:
            print("Selection cancelled.")
            return

        chosen_path = Path(chosen).expanduser()
        if not chosen_path.exists():
            raise SystemExit(f"Selected path does not exist: {chosen}")

        editor = os.environ.get("EDITOR", "vi")
        editor_cmd = shlex.split(editor)
        editor_cmd.append(str(chosen_path))

        try:
            subprocess.run(editor_cmd, check=True)
        except FileNotFoundError as exc:
            raise SystemExit(f"EDITOR not found: {editor_cmd[0]}") from exc
        except subprocess.CalledProcessError as exc:
            raise SystemExit(exc.returncode) from exc
    else:
        print(f"No todos with status '{args.status}'.")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    sub_add = sub.add_parser("add", help="Create a new todo by opening $EDITOR.")
    sub_add.set_defaults(func=command_add)

    sub_fix = sub.add_parser(
        "fix-names", help="Rename todo files to match numbering and titles."
    )
    sub_fix.set_defaults(func=command_fix_names)

    sub_status = sub.add_parser("search-status", help="Search todos matching a status.")
    sub_status.add_argument("status", help="Status value to match (e.g. open, closed).")
    sub_status.set_defaults(func=command_search_status)

    return parser


def main(argv: List[str] | None = None) -> None:
    parser = build_parser()
    if len(sys.argv) == 1 and argv is None:
        argv = ['search-status','open']
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
