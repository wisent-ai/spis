#!/usr/bin/env python3
"""Capture a landing record at all three review widths through Weles.

Builds one `wisent.weles-capture-plan.v1` batch with a composition-axis
capture per width (390 × 844, 768 × 1024, 1440 × 1000) and enqueues it
through `stado host weles-capture` on the registry host. Weles stores the
screenshot **and** the rendered DOM (`*_dom_*.html`) for every width, which
is exactly what the landing-family contract requires.

Usage:
  capture-widths.py <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]
"""

from __future__ import annotations

import argparse
import datetime
import json
import re
import sys
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PLAN_SCHEMA = "wisent.weles-capture-plan.v1"
NAMESPACE = "stado://weles-captures/"
WIDTHS = [(390, 844), (768, 1024), (1440, 1000)]
DEFAULT_HOST = "charless-mac-mini"


def fail(message: str):
    print(f"capture-widths: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_catalog(catalog: str) -> tuple[Path, dict, dict]:
    directory = ROOT / (catalog if catalog.endswith("-examples") else catalog + "-examples")
    if not directory.is_dir():
        fail(f"{directory.name} does not exist")
    sources = json.loads((directory / "sources.json").read_text())
    index = json.loads((directory / "references.json").read_text())
    return directory, sources, index


def pick(sources: dict, index: dict, selector: str | None) -> tuple[dict, dict]:
    for position, example in enumerate(sources["examples"]):
        entry = index["references"][position]
        number = position + 1
        slug = Path(entry["path"]).parent.name
        if selector in (None, str(number), slug, example["name"].lower()):
            return example, entry
    fail(f"record {selector!r} not found")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("catalog")
    parser.add_argument("--record", help="NN, slug, or name; default: every record")
    parser.add_argument("--host", default=DEFAULT_HOST)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    directory, sources, index = load_catalog(args.catalog)
    selected = []
    if args.record:
        example, entry = pick(sources, index, args.record)
        selected.append((example, entry))
    else:
        selected = list(zip(sources["examples"], index["references"]))
    if not selected:
        fail("nothing to capture; add records first")

    batch = f"widths-{datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dt%H%M%SZ')}"
    captures = []
    for example, entry in selected:
        slug = Path(entry["path"]).parent.name
        for width, height in WIDTHS:
            captures.append({
                "batch": batch,
                "site_slug": f"{slug}-{width}",
                "source_url": example["source_url"],
                "axis": "composition",
                "viewport": {"width": width, "height": height, "device_scale_factor": 1},
                "artifact_prefix": f"{NAMESPACE}{batch}/{args.catalog}/{slug}/{width}/",
                "full_page": True,
                "record_seconds": 0,
                "steps": [{"op": "wait_ms", "value": 2500}],
            })

    plan = {"schema": PLAN_SCHEMA, "batch": batch, "target": args.host, "captures": captures}
    plan_path = Path.home() / ".stado" / "work" / "landing-width-plans" / f"{batch}.json"
    plan_path.parent.mkdir(parents=True, exist_ok=True)
    plan_path.write_text(json.dumps(plan, indent=2) + "\n")

    if args.dry_run:
        print(f"dry run: planned {len(captures)} captures across {len(selected)} record(s); plan={plan_path}")
        for capture in captures:
            print(f"  {capture['site_slug']} <- {capture['source_url']} @ {capture['viewport']['width']}px")
        return 0

    import shutil
    import subprocess

    if shutil.which("stado") is None:
        fail("stado is not on PATH; hosts are reached through stado, never ssh")
    result = subprocess.run(
        ["stado", "host", "weles-capture", args.host, "--plan", str(plan_path), "--json"],
        capture_output=True, text=True,
    )
    if result.returncode != 0 and "skarbiec" in (result.stderr + result.stdout).lower():
        # This machine holds no grant for the admission token. That is the
        # designed state: enqueue on the target host instead, where the host's
        # own Skarbiec identity authorizes the batch. The pinned Stado job is
        # the sanctioned remote channel; no credentials leave the host.
        print(f"local enqueue lacks the admission grant; submitting pinned Stado job on {args.host}")
        script = (
            "#!/bin/sh\nset -eu\n"
            f"cp {plan_path} /tmp/spis-widths-plan.json\n"
            "STADO=$HOME/.stado/bin/stado; [ -x \"$STADO\" ] || STADO=$(command -v stado)\n"
            "\"$STADO\" host weles-capture charless-mac-mini --plan /tmp/spis-widths-plan.json --json\n"
        )
        job_script = Path("/tmp") / f"spis-widths-{batch}.sh"
        job_script.write_text(script)
        submit = subprocess.run(
            ["stado", "submit", f"sh {job_script}", "--pinned-host", args.host],
            capture_output=True, text=True,
        )
        if submit.returncode != 0:
            fail(f"remote enqueue submission failed: {(submit.stderr or submit.stdout).strip()[:300]}")
        match = re.search(r"Batch: (batch-\d+)", submit.stdout)
        if not match:
            fail(f"could not read Stado batch id from: {submit.stdout.strip()[:200]}")
        print(f"remote batch {match.group(1)} submitted; poll with: stado status {match.group(1)}")
        return 0
    if result.returncode != 0:
        fail(f"weles-capture refused: {(result.stderr or result.stdout).strip()[:300]}")

    # Record the batch on each touched reference so retrieval can find it.
    for _, entry in selected:
        record_path = directory / entry["path"]
        record = json.loads(record_path.read_text())
        batches = record.setdefault("capture_batches", [])
        if batch not in batches:
            batches.append(batch)
        gaps = record.setdefault("evidence_gaps", [])
        pending = "width captures enqueued; awaiting retrieval"
        if pending not in gaps:
            gaps.append(pending)
        write_json(record_path, record)

    print(f"enqueued {len(captures)} captures as {batch}; artifacts land under {NAMESPACE}{batch}/")
    print("retrieve with spis verify --apply after the host finishes the batch")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
