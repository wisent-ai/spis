#!/usr/bin/env python3
"""Add, edit, or remove a product-type catalog (e.g. landing-page, readme).

A product type is one `*-examples/` directory: a family of reference records
with its own sources, index, and evidence floor. This tool only scaffolds and
maintains the structure; it never fabricates records. The generated index is
refreshed through generate-example-catalogs.py after every mutation.

Usage:
  catalog-type.py add <slug> --title <title> [--description <text>]
  catalog-type.py edit <slug> [--title <title>] [--description <text>]
                              [--status <status>] [--rename <new-slug>]
  catalog-type.py remove <slug> [--force]
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
INDEX = ROOT / "example-catalogs.json"
SUFFIX = "-examples"


def fail(message: str) -> "NoReturn":  # type: ignore[name-defined]
    print(f"catalog-type: {message}", file=sys.stderr)
    raise SystemExit(1)


def catalog_dir(slug: str) -> Path:
    return ROOT / (slug if slug.endswith(SUFFIX) else slug + SUFFIX)


def normalize(slug: str) -> str:
    base = slug[: -len(SUFFIX)] if slug.endswith(SUFFIX) else slug
    if not base or not all(c.isalnum() or c == "-" for c in base) or base != base.lower():
        fail(f"slug must be lowercase kebab-case, got {slug!r}")
    return base + SUFFIX


def load_sources(directory: Path) -> dict:
    sources = directory / "sources.json"
    if not sources.is_file():
        fail(f"{directory.name}/sources.json is missing; not a managed catalog")
    return json.loads(sources.read_text())


def save_sources(directory: Path, sources: dict) -> None:
    (directory / "sources.json").write_text(json.dumps(sources, indent=2, ensure_ascii=False) + "\n")


def regenerate() -> None:
    result = subprocess.run(
        [sys.executable, str(ROOT / "generate-example-catalogs.py")],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        fail(f"index regeneration refused the change:\n{result.stdout}{result.stderr}")


def cmd_add(args: argparse.Namespace) -> None:
    slug = normalize(args.slug)
    directory = catalog_dir(slug)
    if directory.exists():
        fail(f"{directory.name} already exists")
    if not args.title:
        fail("--title is required for add")
    directory.mkdir()
    (directory / "references").mkdir()
    sources = {
        "schema": "wisent.example-catalog.v2",
        "catalog": slug,
        "slug": slug,
        "title": args.title,
        "description": args.description or "",
        "status": "scaffolded",
        "curated_at": __import__("datetime").date.today().isoformat(),
        "count": 0,
        "examples": [],
        "visual_count": 0,
        "structure_count": 0,
    }
    save_sources(directory, sources)
    (directory / "references.json").write_text(
        json.dumps({"schema": "wisent.full-product-reference.index.v1", "records": []}, indent=2) + "\n"
    )
    regenerate()
    print(f"added {directory.name} ({args.title}); scaffolded with zero records")


def cmd_edit(args: argparse.Namespace) -> None:
    slug = normalize(args.slug)
    directory = catalog_dir(slug)
    if not directory.is_dir():
        fail(f"{directory.name} does not exist")
    sources = load_sources(directory)
    changed = []
    if args.title:
        sources["title"] = args.title
        changed.append("title")
    if args.description is not None:
        sources["description"] = args.description
        changed.append("description")
    if args.status:
        sources["status"] = args.status
        changed.append("status")
    if args.rename:
        new_slug = normalize(args.rename)
        new_directory = catalog_dir(new_slug)
        if new_directory.exists():
            fail(f"{new_directory.name} already exists")
        sources["slug"] = new_slug
        sources["catalog"] = new_slug
        changed.append(f"slug -> {new_slug}")
    if not changed:
        fail("nothing to edit: pass --title, --description, --status, or --rename")
    save_sources(directory, sources)
    if args.rename:
        directory.rename(catalog_dir(new_slug))
    regenerate()
    print(f"edited {slug}: {', '.join(changed)}")


def cmd_remove(args: argparse.Namespace) -> None:
    slug = normalize(args.slug)
    directory = catalog_dir(slug)
    if not directory.is_dir():
        fail(f"{directory.name} does not exist")
    sources = load_sources(directory)
    record_count = len(sources.get("records", []))
    references_dir = directory / "references"
    stored = len(list(references_dir.iterdir())) if references_dir.is_dir() else 0
    if (record_count or stored) and not args.force:
        fail(
            f"{directory.name} holds {record_count} indexed record(s) and {stored} "
            "reference director(ies); passing --force deletes that evidence permanently"
        )
    shutil.rmtree(directory)
    regenerate()
    print(f"removed {directory.name}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    add = sub.add_parser("add")
    add.add_argument("slug")
    add.add_argument("--title", required=True)
    add.add_argument("--description")
    add.set_defaults(func=cmd_add)

    edit = sub.add_parser("edit")
    edit.add_argument("slug")
    edit.add_argument("--title")
    edit.add_argument("--description")
    edit.add_argument("--status")
    edit.add_argument("--rename")
    edit.set_defaults(func=cmd_edit)

    remove = sub.add_parser("remove")
    remove.add_argument("slug")
    remove.add_argument("--force", action="store_true")
    remove.set_defaults(func=cmd_remove)

    args = parser.parse_args()
    args.func(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
