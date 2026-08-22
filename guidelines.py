#!/usr/bin/env python3
"""Derive a DRAFT guidelines document from a catalog's measured records.

Every statement in the draft is an aggregate over records and carries its own
count (n/m). Nothing is invented: a pattern only appears if the records show
it, and a family with no measured records produces no guidelines. The output
is a draft for human review — it becomes guidelines only after a human edits,
confirms, and moves it into product-guidelines.

Usage:
  guidelines.py <catalog> [--out <file>]
"""

from __future__ import annotations

import argparse
import collections
import datetime
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def fail(message: str):
    print(f"guidelines: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_records(catalog: str) -> list[dict]:
    directory = ROOT / (catalog if catalog.endswith("-examples") else catalog + "-examples")
    sources_path = directory / "sources.json"
    if not sources_path.is_file():
        fail(f"{directory.name} is not a managed catalog")
    sources = json.loads(sources_path.read_text())
    records = []
    for entry in json.loads((directory / "references.json").read_text()).get("references", []):
        record_path = directory / entry["path"]
        if record_path.is_file():
            records.append(json.loads(record_path.read_text()))
    return sources, records


def counter_block(title: str, pairs: list[tuple[str, int]], total: int, lines: list[str]) -> None:
    if not pairs:
        return
    lines.append(f"### {title}")
    lines.append("")
    for name, count in sorted(pairs, key=lambda kv: -kv[1]):
        lines.append(f"- {name} — {count}/{total} records")
    lines.append("")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("catalog")
    parser.add_argument("--out")
    args = parser.parse_args()

    sources, records = load_records(args.catalog)
    total = len(records)
    if total == 0:
        fail(f"{args.catalog} has no measured records; guidelines require evidence")

    complete = sum(r.get("evidence_status") == "complete" for r in records)
    provenance = collections.Counter(
        cls for r in records for cls in r.get("motion_provenance", [])
    )
    interactions = collections.Counter(
        i.get("name", "unnamed") for r in records for i in r.get("interactions", [])
    )
    timing = collections.Counter(
        (m.get("timing_class") or "unspecified")
        for r in records
        for m in (r.get("motion_analysis") or [])
    )
    accessibility_measured = sum(1 for r in records if r.get("accessibility", {}).get("measured"))
    categories = collections.Counter(e.get("category", "uncategorized") for e in sources.get("examples", []))

    gap_counter = collections.Counter(g for r in records for g in r.get("evidence_gaps", []))

    lines = [
        f"# {sources.get('title', args.catalog)} — derived guidelines (DRAFT)",
        "",
        f"Machine-derived from `{args.catalog}` on "
        f"{datetime.date.today().isoformat()}. Every line cites its record count; "
        "a line without a count is not from this corpus.",
        "",
        "**This is a DRAFT.** It becomes guidelines only after a human reviews it, "
        "edits it, and moves the confirmed rules into product-guidelines. Counts "
        "below quote only what the records measure; the corpus does not score taste.",
        "",
        "## Coverage",
        "",
        f"- records: {total} ({complete} complete, {total - complete} partial)",
        f"- accessibility measured on the product: {accessibility_measured}/{total}",
        "",
    ]
    counter_block("Record categories", list(categories.items()), total, lines)
    counter_block("Motion provenance", list(provenance.items()), total, lines)
    counter_block("Observed interactions (how often each appeared)", list(interactions.items()), total, lines)
    counter_block("Motion timing classes", list(timing.items()), total, lines)
    counter_block("Named evidence gaps across records", list(gap_counter.items()), total, lines)
    lines += [
        "## Review checklist",
        "",
        "- [ ] every rule kept above still cites a count I accept",
        "- [ ] rules I reject are deleted here before promotion",
        "- [ ] promoted copy lands in product-guidelines with this file cited as source",
        "",
    ]

    out = Path(args.out) if args.out else ROOT / args.catalog / "guidelines-draft.md"
    out.write_text("\n".join(lines))
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
