#!/usr/bin/env python3
"""Add, get, or remove a single reference record inside a product-type catalog.

A record is one numbered product reference: an overview image plus
`references/<NN-slug>/reference.json`. Adding scaffolds the record honestly —
motion, states, journey, and accessibility start empty and are named in
`evidence_gaps`, so the record is `partial` until the pipeline measures it.
The generated index is refreshed through generate-example-catalogs.py after
every mutation.

Usage:
  reference-record.py add <catalog> --name <Name> --source-url <url>
                          --category <text> --selection-note <text>
                          --visual <image> [--owner <owner>]
  reference-record.py get <catalog> <NN|slug>
  reference-record.py remove <catalog> <NN|slug> [--force]
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def fail(message: str):
    print(f"reference: {message}", file=sys.stderr)
    raise SystemExit(1)


def catalog_dir(slug: str) -> Path:
    directory = ROOT / (slug if slug.endswith("-examples") else slug + "-examples")
    if not directory.is_dir():
        fail(f"{directory.name} does not exist")
    return directory


def read_json(path: Path) -> dict:
    return json.loads(path.read_text())


def write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 16), b""):
            digest.update(chunk)
    return digest.hexdigest()


def image_dimensions(path: Path) -> tuple[int, int]:
    result = subprocess.run(
        ["sips", "-g", "pixelWidth", "-g", "pixelHeight", str(path)],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        fail(f"cannot read image dimensions: {path}")
    width = height = None
    for line in result.stdout.splitlines():
        if "pixelWidth" in line:
            width = int(line.rsplit(":", 1)[1].strip())
        if "pixelHeight" in line:
            height = int(line.rsplit(":", 1)[1].strip())
    return width, height


def kebab(name: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")
    if not slug:
        fail(f"--name {name!r} produces an empty slug")
    return slug


def load_catalog(directory: Path) -> tuple[dict, dict, dict]:
    sources = read_json(directory / "sources.json")
    index = read_json(directory / "references.json")
    return sources, index, sources


def save_all(directory: Path, sources: dict, index: dict) -> None:
    sources["count"] = len(sources["examples"])
    sources["visual_count"] = len(sources["examples"])
    sources["structure_count"] = len(sources["examples"])
    write_json(directory / "sources.json", sources)
    index["reference_count"] = len(index["references"])
    index["complete_count"] = sum(r["evidence_status"] == "complete" for r in index["references"])
    index["partial_count"] = sum(r["evidence_status"] == "partial" for r in index["references"])
    index["generated_at"] = index["measured_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    write_json(directory / "references.json", index)
    regenerate()


def regenerate() -> None:
    result = subprocess.run(
        [sys.executable, str(ROOT / "generate-example-catalogs.py")],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        fail(f"index regeneration refused the change:\n{result.stdout}{result.stderr}")


def find_record(directory: Path, sources: dict, index: dict, identifier: str):
    number = int(identifier) if identifier.isdigit() else None
    for position, example in enumerate(sources["examples"]):
        entry = index["references"][position]
        if number == position + 1 or entry["name"].lower().replace("_", "-").replace(" ", "-") == identifier.lower():
            return position, example, entry
    fail(f"record {identifier!r} not found in {directory.name}")


def cmd_add(args) -> None:
    directory = catalog_dir(args.catalog)
    sources, index, _ = read_json(directory / "sources.json"), read_json(directory / "references.json"), None
    visual_source = Path(args.visual)
    if not visual_source.is_file():
        fail(f"--visual {visual_source} is not a file")
    if any(example["name"].lower() == args.name.lower() for example in sources["examples"]):
        fail(f"a record named {args.name!r} already exists")

    slug = f"{len(sources['examples']) + 1:02d}-{kebab(args.name)}"
    suffix = visual_source.suffix.lower().lstrip(".") or "png"
    images_dir = directory / "images"
    images_dir.mkdir(exist_ok=True)
    visual_path = images_dir / f"{slug}.{suffix}"
    shutil.copyfile(visual_source, visual_path)

    width, height = image_dimensions(visual_path)
    digest = sha256_file(visual_path)
    today = datetime.date.today().isoformat()

    orientation = "landscape" if width >= height else "portrait"
    structure = {
        "analysis_kind": "deterministic-image-layout-v1",
        "image_sha256": digest,
        "orientation": orientation,
        "layout_model": "unanalyzed-scaffold",
        "panel_summary": "Scaffold region covering the full overview image; run analyze-structures to replace it.",
        "detected_separators": {"vertical": [], "horizontal": []},
        "visual_density": "unknown",
        "confidence": "low",
        "regions": [
            {
                "role": "full frame",
                "position": "center",
                "bounds": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
                "evidence": "placeholder bounds over the whole scaffolded image",
            }
        ],
    }


    example = {
        "name": args.name,
        "source_url": args.source_url,
        "category": args.category,
        "selection_note": args.selection_note,
        "visual": {
            "source_page_url": args.source_url,
            "source_image_url": args.owner or args.source_url,
            "local_path": f"images/{visual_path.name}",
            "capture_kind": "provided-file",
            "captured_at": today,
            "format": suffix,
            "width": width,
            "height": height,
            "original_width": width,
            "original_height": height,
            "bytes": visual_path.stat().st_size,
            "sha256": digest,
        },
        "interface_structure": structure,
    }
    sources["examples"].append(example)

    gaps = [
        "motion evidence absent",
        "first-success sequence not recorded",
        "state visuals below the three-state floor",
        "interaction map absent",
        "user journey not recorded",
        "motion analysis absent",
        "accessibility never measured against the product",
    ]
    now = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    record_dir = directory / "references" / slug
    record_dir.mkdir(parents=True, exist_ok=True)
    (record_dir / "media").mkdir(exist_ok=True)
    record = {
        "schema": "wisent.full-product-reference.v2",
        "name": args.name,
        "product_url": args.source_url,
        "evidence_status": "partial",
        "upstream_owner": args.owner or args.source_url,
        "captured_at": today,
        "motion": [],
        "states": [],
        "interactions": [],
        "journey": {},
        "accessibility": {"measured": False, "observations": [], "unknowns": ["everything; no audit exists yet"]},
        "motion_provenance": [],
        "evidence_gaps": gaps,
        "measured_at": now,
    }
    write_json(record_dir / "reference.json", record)
    index["references"].append(
        {
            "index": len(index["references"]) + 1,
            "name": args.name,
            "path": f"references/{slug}/reference.json",
            "evidence_status": "partial",
            "evidence_gap_count": len(gaps),
        }
    )

    save_all(directory, sources, index)
    print(f"added {directory.name}/{slug}: {args.name} ({len(gaps)} named gaps, status partial)")


def cmd_get(args) -> None:
    directory = catalog_dir(args.catalog)
    sources = read_json(directory / "sources.json")
    index = read_json(directory / "references.json")
    position, example, entry = find_record(directory, sources, index, args.identifier)
    record = read_json(directory / entry["path"].lstrip("/")) if False else read_json(directory / entry["path"])
    print(json.dumps({"example": example, "entry": entry, "record": record}, indent=2, ensure_ascii=False))


def cmd_remove(args) -> None:
    directory = catalog_dir(args.catalog)
    sources = read_json(directory / "sources.json")
    index = read_json(directory / "references.json")
    position, example, entry = find_record(directory, sources, index, args.identifier)
    record = read_json(directory / entry["path"])
    measured_motion = bool(record.get("motion"))
    if (measured_motion or record.get("journey")) and not args.force:
        fail("the record carries motion or journey evidence; pass --force to delete it permanently")

    record_dir = directory / Path(entry["path"]).parent
    shutil.rmtree(record_dir)
    visual_local = example.get("visual", {}).get("local_path")
    if visual_local:
        visual_path = directory / visual_local
        if visual_path.is_file():
            visual_path.unlink()
    del sources["examples"][position]
    del index["references"][position]
    for new_index, entry_after in enumerate(index["references"], 1):
        entry_after["index"] = new_index

    save_all(directory, sources, index)
    print(f"removed {directory.name} record {args.identifier}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    add = sub.add_parser("add")
    add.add_argument("catalog")
    add.add_argument("--name", required=True)
    add.add_argument("--source-url", required=True)
    add.add_argument("--category", required=True)
    add.add_argument("--selection-note", required=True)
    add.add_argument("--visual", required=True)
    add.add_argument("--owner")
    add.set_defaults(func=cmd_add)

    get = sub.add_parser("get")
    get.add_argument("catalog")
    get.add_argument("identifier")
    get.set_defaults(func=cmd_get)

    remove = sub.add_parser("remove")
    remove.add_argument("catalog")
    remove.add_argument("identifier")
    remove.add_argument("--force", action="store_true")
    remove.set_defaults(func=cmd_remove)

    args = parser.parse_args()
    args.func(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
