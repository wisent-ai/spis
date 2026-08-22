#!/usr/bin/env python3
"""Render the example-catalog index and catalog pages from the measured records.

This generator is the gate. It refuses to render a catalog whose data contradicts
the files beside it, and it renders the measured numbers rather than an intention:
how many records are complete, how many are partial, and how the motion evidence was
actually obtained (a product we drove, a browser we drove, or media its owner
published).

The previous version asserted "50 complete references" per catalog and validated
state fields (`name`, `source_motion_path`) that no record has ever carried, so it
could not run at all — which is why the corpus drifted unnoticed. The contract now
lives in `reference_contract.py` and is shared with `verify-reference-evidence.py`.

    ./generate-example-catalogs.py             # validate and render
    ./generate-example-catalogs.py --check     # validate only, write nothing
"""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path
from urllib.parse import urlparse

from reference_contract import (
    CANONICAL_MOTION_KINDS,
    CATALOG_SCHEMA,
    EVIDENCE_STATUSES,
    INDEX_SCHEMA,
    INTERACTION_FIELDS,
    JOURNEY_FIELDS,
    JOURNEY_STEP_FIELDS,
    LOCAL_PROVENANCE,
    MIN_INTERACTIONS,
    MIN_JOURNEY_STEPS,
    MIN_MOTION_SECONDS,
    MIN_STATES,
    MOTION_ANALYSIS_FIELDS,
    MOTION_ANALYSIS_OPTIONAL,
    MOTION_SUFFIXES,
    PROVENANCE_CLASSES,
    RECORD_SCHEMA,
    STATE_SUFFIXES,
    TIMING_CLASSES,
)

ROOT = Path(__file__).resolve().parent

# Curated third-party families, in reading order, followed by catalogs of our own
# products. Any other directory that satisfies the contract is appended.
CATALOGS = (
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "wisent-product-examples",
)

PROVENANCE_LABEL = {
    "local-product-run": "product run here",
    "local-browser-recording": "browser driven here",
    "upstream-owner-media": "owner-published media",
    "unclassified": "unclassified",
}


class ContractError(ValueError):
    """A catalog contradicts the evidence contract."""


def discovered_catalogs() -> list[str]:
    known = list(CATALOGS)
    for path in sorted(ROOT.glob("*-examples")):
        if not (path / "sources.json").is_file() or not (path / "references").is_dir():
            continue
        if path.name not in known:
            known.append(path.name)
    return [slug for slug in known if (ROOT / slug / "references").is_dir()]


def require_nonempty(record: dict, fields: tuple[str, ...], context: str) -> None:
    missing = [field for field in fields if record.get(field) in (None, "", [])]
    if missing:
        raise ContractError(f"{context}: missing {missing}")


def resolve_evidence_path(base: Path, relative: str, context: str) -> Path:
    path = (base / relative).resolve()
    if not path.is_relative_to(base.resolve()) or not path.is_file():
        raise ContractError(f"{context}: unavailable local evidence {relative!r}")
    return path


def validate_file_metadata(path: Path, record: dict, context: str) -> None:
    payload = path.read_bytes()
    if len(payload) != record.get("bytes"):
        raise ContractError(f"{context}: byte count differs from the file")
    if hashlib.sha256(payload).hexdigest() != record.get("sha256"):
        raise ContractError(f"{context}: SHA-256 differs from the file")


def validate_motion(record: dict, record_path: Path, reference_dir: Path) -> list[str]:
    motion = record.get("motion")
    if not isinstance(motion, list):
        raise ContractError(f"{record_path}: motion must be a list")
    if not motion:
        if record.get("evidence_status") == "partial":
            return []
        raise ContractError(f"{record_path}: complete evidence needs at least one motion asset")
    classes: list[str] = []
    for position, item in enumerate(motion, 1):
        context = f"{record_path}: motion {position}"
        require_nonempty(
            item,
            ("local_path", "source_url", "media_kind", "bytes", "sha256", "capture_method", "provenance_class"),
            context,
        )
        if item["media_kind"] not in CANONICAL_MOTION_KINDS:
            raise ContractError(
                f"{context}: media kind {item['media_kind']!r} is not in the canonical vocabulary "
                f"{sorted(CANONICAL_MOTION_KINDS)}"
            )
        if item["provenance_class"] not in PROVENANCE_CLASSES:
            raise ContractError(f"{context}: unknown provenance class {item['provenance_class']!r}")
        if item["provenance_class"] == "unclassified":
            raise ContractError(f"{context}: provenance was never classified")
        if not item.get("measured"):
            raise ContractError(f"{context}: asset was never measured; run verify-reference-evidence.py")
        parsed = urlparse(item["source_url"])
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            raise ContractError(f"{context}: invalid source URL")
        motion_path = resolve_evidence_path(reference_dir, item["local_path"], context)
        if motion_path.suffix.casefold() not in MOTION_SUFFIXES:
            raise ContractError(f"{context}: unsupported motion format")
        duration = item.get("duration_seconds")
        if duration is None or duration < MIN_MOTION_SECONDS:
            raise ContractError(f"{context}: measured duration {duration!r} is below the floor")
        if motion_path.suffix.casefold() != ".cast":
            require_nonempty(item, ("width", "height"), context)
        validate_file_metadata(motion_path, item, context)
        classes.append(item["provenance_class"])

    declared = record.get("motion_provenance")
    if sorted(set(classes)) != sorted(declared or []):
        raise ContractError(f"{record_path}: motion_provenance does not match the motion entries")
    return classes


def validate_states(record: dict, record_path: Path, reference_dir: Path) -> None:
    states = record.get("states")
    if not isinstance(states, list):
        raise ContractError(f"{record_path}: states must be a list")
    if len(states) < MIN_STATES and record.get("evidence_status") == "complete":
        raise ContractError(f"{record_path}: complete evidence needs at least {MIN_STATES} local states")
    for position, item in enumerate(states, 1):
        context = f"{record_path}: state {position}"
        require_nonempty(item, ("local_path", "width", "height", "bytes", "sha256"), context)
        state_path = resolve_evidence_path(reference_dir, item["local_path"], context)
        if state_path.suffix.casefold() not in STATE_SUFFIXES:
            raise ContractError(f"{context}: unsupported state-image format")
        validate_file_metadata(state_path, item, context)


def validate_behaviour(record: dict, record_path: Path) -> None:
    interactions = record.get("interactions")
    if not isinstance(interactions, list):
        raise ContractError(f"{record_path}: interactions must be a list")
    if len(interactions) < MIN_INTERACTIONS and record.get("evidence_status") == "complete":
        raise ContractError(f"{record_path}: complete evidence needs at least {MIN_INTERACTIONS} observed interactions")
    for position, item in enumerate(interactions, 1):
        require_nonempty(item, INTERACTION_FIELDS, f"{record_path}: interaction {position}")

    journey = record.get("journey") or {}
    if journey:
        require_nonempty(journey, JOURNEY_FIELDS, f"{record_path}: journey")
        steps = journey["steps"]
        if not isinstance(steps, list) or len(steps) < MIN_JOURNEY_STEPS:
            raise ContractError(f"{record_path}: journey needs at least {MIN_JOURNEY_STEPS} observed steps")
        for position, step in enumerate(steps, 1):
            require_nonempty(step, JOURNEY_STEP_FIELDS, f"{record_path}: journey step {position}")
            if step["index"] != position:
                raise ContractError(f"{record_path}: journey step order is invalid")
    elif record.get("evidence_status") == "complete":
        raise ContractError(f"{record_path}: complete evidence needs a journey")

    analysis = record.get("motion_analysis")
    if analysis is not None:
        entries = analysis if isinstance(analysis, list) else [analysis]
        for position, item in enumerate(entries, 1):
            unknown = [key for key in item if key not in (*MOTION_ANALYSIS_FIELDS, *MOTION_ANALYSIS_OPTIONAL)]
            if unknown:
                raise ContractError(f"{record_path}: motion analysis {position} has unknown fields {unknown}")
            missing = [key for key in MOTION_ANALYSIS_FIELDS if key not in item]
            if missing:
                raise ContractError(f"{record_path}: motion analysis {position} omits {missing}")
            timing = item.get("timing_class")
            if timing is not None and timing not in TIMING_CLASSES:
                raise ContractError(f"{record_path}: timing class {timing!r} is not one of {sorted(TIMING_CLASSES)}")
    elif record.get("evidence_status") == "complete":
        raise ContractError(f"{record_path}: complete evidence needs motion_analysis")

    accessibility = record.get("accessibility") or {}
    if not isinstance(accessibility.get("observations"), list) or not isinstance(
        accessibility.get("unknowns"), list
    ):
        raise ContractError(f"{record_path}: accessibility observations and unknowns are required")


def load_full_references(slug: str, examples: list[dict]) -> dict:
    catalog_dir = (ROOT / slug).resolve()
    index_path = catalog_dir / "references.json"
    index = json.loads(index_path.read_text())
    require_nonempty(
        index,
        ("schema", "catalog", "reference_count", "references"),
        str(index_path),
    )
    if index["schema"] != INDEX_SCHEMA:
        raise ContractError(f"{index_path}: expected schema {INDEX_SCHEMA!r}, found {index['schema']!r}")
    if index["catalog"] != slug:
        raise ContractError(f"{index_path}: catalog must equal directory name")

    records = index["references"]
    if not isinstance(records, list) or len(records) != len(examples):
        raise ContractError(f"{index_path}: {len(records)} references for {len(examples)} curated examples")
    if index["reference_count"] != len(records):
        raise ContractError(f"{index_path}: reference_count does not match the reference list")

    provenance: Counter[str] = Counter()
    statuses: Counter[str] = Counter()
    gap_total = 0

    for position, (entry, example) in enumerate(zip(records, examples), 1):
        require_nonempty(entry, ("index", "name", "path", "evidence_status"), str(index_path))
        if entry["index"] != position or entry["name"] != example["name"]:
            raise ContractError(f"{index_path}: reference {position} does not match sources.json")
        if entry["evidence_status"] not in EVIDENCE_STATUSES:
            raise ContractError(f"{index_path}: reference {position} has status {entry['evidence_status']!r}")

        record_path = resolve_evidence_path(catalog_dir, entry["path"], str(index_path))
        record = json.loads(record_path.read_text())
        if record.get("schema") != RECORD_SCHEMA:
            raise ContractError(f"{record_path}: expected schema {RECORD_SCHEMA!r}")
        if record.get("name") != example["name"] or record.get("product_url") != example["source_url"]:
            raise ContractError(f"{record_path}: product identity differs from sources.json")

        gaps = record.get("evidence_gaps")
        if not isinstance(gaps, list):
            raise ContractError(f"{record_path}: evidence_gaps must be a list, empty when nothing is missing")
        expected = "complete" if not gaps else "partial"
        if record.get("evidence_status") != expected:
            raise ContractError(
                f"{record_path}: status {record.get('evidence_status')!r} contradicts {len(gaps)} recorded gaps"
            )
        if entry["evidence_status"] != expected or entry.get("evidence_gap_count") != len(gaps):
            raise ContractError(f"{index_path}: reference {position} disagrees with its record")

        reference_dir = record_path.parent.resolve()
        provenance.update(validate_motion(record, record_path, reference_dir))
        validate_states(record, record_path, reference_dir)
        validate_behaviour(record, record_path)

        statuses[expected] += 1
        gap_total += len(gaps)

    if index.get("complete_count") != statuses["complete"] or index.get("partial_count") != statuses["partial"]:
        raise ContractError(
            f"{index_path}: recorded counts ({index.get('complete_count')} complete, "
            f"{index.get('partial_count')} partial) differ from the measured "
            f"{statuses['complete']}/{statuses['partial']}"
        )

    index["measured_provenance"] = dict(sorted(provenance.items()))
    index["measured_gap_total"] = gap_total
    index["locally_driven_count"] = sum(count for name, count in provenance.items() if name in LOCAL_PROVENANCE)
    return index


def load_catalog(slug: str) -> dict:
    source_path = ROOT / slug / "sources.json"
    catalog = json.loads(source_path.read_text())
    if catalog.get("schema") != CATALOG_SCHEMA:
        raise ContractError(f"{source_path}: expected schema {CATALOG_SCHEMA!r}")
    if catalog.get("catalog") != slug:
        raise ContractError(f"{source_path}: catalog must equal directory name")

    examples = catalog.get("examples")
    if not isinstance(examples, list):
        raise ContractError(f"{source_path}: examples must be a list")
    if not examples and catalog.get("status") == "scaffolded":
        # A scaffolded catalog is an intentional empty shell: the contract
        # requires records before they are indexed, so an empty type renders
        # with zero counts and its named evidence gaps.
        return {
            "catalog": slug,
            "slug": slug,
            "title": catalog.get("title", slug),
            "description": catalog.get("description", ""),
            "count": 0,
            "image_count": 0,
            "structure_count": 0,
            "complete_record_count": 0,
            "partial_record_count": 0,
            "visual_count": 0,
            "structure_count": 0,
            "curated_at": catalog.get("curated_at", "unknown"),
            "measured_provenance": {},
            "full_reference_catalog": {"measured_provenance": {}, "complete_count": 0, "partial_count": 0, "locally_driven_count": 0, "measured_gap_total": 0, "reference_count": 0},
            "source": f"{slug}/sources.json",
            "readme": f"{slug}/README.md",
            "full_reference": f"{slug}/full-reference.md",
            "full_reference_source": f"{slug}/references.json",
            "scaffolded": True,
            "examples": [],
        }
        raise ContractError(f"{source_path}: no examples")
    for key in ("count", "visual_count", "structure_count"):
        if catalog.get(key) != len(examples):
            raise ContractError(f"{source_path}: {key} does not match the {len(examples)} examples")

    names: set[str] = set()
    urls: set[str] = set()
    catalog_dir = source_path.parent.resolve()
    for index, example in enumerate(examples, 1):
        required = ("name", "source_url", "category", "selection_note", "visual", "interface_structure")
        missing = [key for key in required if not example.get(key)]
        if missing:
            raise ContractError(f"{source_path}: example {index} is missing {missing}")
        parsed = urlparse(example["source_url"])
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            raise ContractError(f"{source_path}: example {index} has an invalid URL")
        folded_name = example["name"].casefold()
        if folded_name in names:
            raise ContractError(f"{source_path}: duplicate example name {example['name']!r}")
        if example["source_url"] in urls:
            raise ContractError(f"{source_path}: duplicate source URL {example['source_url']!r}")
        names.add(folded_name)
        urls.add(example["source_url"])

        visual = example["visual"]
        visual_required = [
            "source_page_url",
            "local_path",
            "capture_kind",
            "captured_at",
            "format",
            "width",
            "height",
            "bytes",
            "sha256",
        ]
        capture_kind = visual.get("capture_kind")
        if capture_kind == "local-terminal-render":
            visual_required.append("source_recording_path")
        elif capture_kind != "local-browser-screenshot":
            visual_required.append("source_image_url")
        visual_missing = [key for key in visual_required if not visual.get(key)]
        if visual_missing:
            raise ContractError(f"{source_path}: example {index} visual is missing {visual_missing}")
        image_path = (catalog_dir / visual["local_path"]).resolve()
        if not image_path.is_relative_to(catalog_dir) or not image_path.is_file():
            raise ContractError(f"{source_path}: example {index} visual path is unavailable")
        payload = image_path.read_bytes()
        if len(payload) != visual["bytes"]:
            raise ContractError(f"{source_path}: example {index} visual byte count differs")
        if hashlib.sha256(payload).hexdigest() != visual["sha256"]:
            raise ContractError(f"{source_path}: example {index} visual digest differs")

        structure = example["interface_structure"]
        structure_required = (
            "analysis_kind",
            "image_sha256",
            "orientation",
            "layout_model",
            "panel_summary",
            "regions",
            "detected_separators",
            "visual_density",
            "confidence",
        )
        structure_missing = [key for key in structure_required if structure.get(key) in (None, "")]
        if structure_missing:
            raise ContractError(f"{source_path}: example {index} structure is missing {structure_missing}")
        if structure["image_sha256"] != visual["sha256"]:
            raise ContractError(f"{source_path}: example {index} structure describes another image")
        if not isinstance(structure["regions"], list) or not structure["regions"]:
            raise ContractError(f"{source_path}: example {index} has no structural regions")

    catalog["full_reference_catalog"] = load_full_references(slug, examples)
    return catalog


def escape_cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def provenance_sentence(index: dict) -> str:
    parts = [
        f"{count} {PROVENANCE_LABEL[name]}"
        for name, count in sorted(index["measured_provenance"].items(), key=lambda kv: -kv[1])
    ]
    return ", ".join(parts) if parts else "no measured motion"


def render_readme(catalog: dict) -> str:
    index = catalog["full_reference_catalog"]
    rows = [
        f"# {catalog['title']}",
        "",
        catalog["description"],
        "",
        "Each entry pairs an attributed overview image and a measured panel anatomy with a "
        "per-product record: motion evidence, named states, an observed first-success journey, "
        "interaction and recovery behavior, accessibility observations, and provenance. Every "
        "number below is measured by `verify-reference-evidence.py`, and a record that is missing "
        "evidence says so in its own `evidence_gaps`.",
        "",
        f"**Examples:** {catalog['count']}  ",
        f"**Images:** {catalog['visual_count']}  ",
        f"**Structural analyses:** {catalog['structure_count']}  ",
        f"**Records with no remaining gap:** {index['complete_count']}  ",
        f"**Records with named gaps:** {index['partial_count']} ({index['measured_gap_total']} gaps)  ",
        f"**Motion provenance:** {provenance_sentence(index)}  ",
        f"**Curated:** {catalog['curated_at']}  ",
        "**Visual source data:** [`sources.json`](sources.json)  ",
        "**Record index:** [`references.json`](references.json)  ",
        "**Cross-example synthesis:** [`full-reference.md`](full-reference.md)",
        "",
        "| # | Reference image | Record | Motion evidence | Category | Interface structure | What to study |",
        "|---:|---|---|---|---|---|---|",
    ]
    for position, example in enumerate(catalog["examples"], 1):
        name = escape_cell(example["name"])
        url = example["source_url"]
        category = escape_cell(example["category"])
        note = escape_cell(example["selection_note"])
        visual = example["visual"]
        entry = index["references"][position - 1]
        reference_readme = str(Path(entry["path"]).with_name("README.md"))
        record = json.loads((ROOT / catalog["catalog"] / entry["path"]).read_text())
        provenance = ", ".join(PROVENANCE_LABEL[name] for name in record["motion_provenance"])
        status = (
            "no remaining gap"
            if entry["evidence_status"] == "complete"
            else f"{entry['evidence_gap_count']} named gaps"
        )
        structure = example["interface_structure"]
        image = f'<a href="{url}"><img src="{visual["local_path"]}" alt="{name} interface reference" width="220"></a>'
        region_text = "; ".join(f"{item['role']} ({item['position']})" for item in structure["regions"])
        anatomy = escape_cell(
            f"{structure['layout_model']}: {structure['panel_summary']} "
            f"Panels: {region_text}. Density: {structure['visual_density']}; "
            f"confidence: {structure['confidence']}."
        )
        rows.append(
            f"| {position} | {image} | [{name}]({reference_readme}) · [official product]({url}) "
            f"| {provenance}, {status} | {category} | {anatomy} | {note} |"
        )
    rows.extend(
        (
            "",
            "Normalized panel bounds, detected separators, image dimensions, source-image URLs, hashes, "
            "and analysis confidence are recorded in [`sources.json`](sources.json). Media kinds, measured "
            "durations, provenance classes, and per-record gaps are in [`references.json`](references.json).",
            "",
            "Attribution and product ownership remain with the linked source.",
            "",
        )
    )
    return "\n".join(rows)


def render_index(catalogs: list[dict]) -> str:
    totals: Counter[str] = Counter()
    for catalog in catalogs:
        totals.update(catalog["full_reference_catalog"]["measured_provenance"])
    complete = sum(c["full_reference_catalog"]["complete_count"] for c in catalogs)
    partial = sum(c["full_reference_catalog"]["partial_count"] for c in catalogs)
    local = sum(c["full_reference_catalog"]["locally_driven_count"] for c in catalogs)
    motion_total = sum(totals.values())

    rows = [
        "# Product interface example catalogs",
        "",
        "A shared evidence library for choosing complete interaction models before product "
        "implementation. Each family pairs a curated visual field and measured panel anatomy with "
        "per-product records of motion, states, journey, interactions, recovery, accessibility, and "
        "provenance.",
        "",
        "Motion evidence is labelled by how it was obtained, because the difference matters: "
        f"{local} of {motion_total} assets are a product driven on our own machines, the rest are "
        "recordings the product's owner published. A record still missing evidence is `partial` and "
        "carries the missing items in its own `evidence_gaps`; nothing is called complete on the "
        "strength of a marketing clip.",
        "",
        f"**Catalogs:** {len(catalogs)}  ",
        f"**Screenshots:** {sum(catalog['visual_count'] for catalog in catalogs)}  ",
        f"**Structural analyses:** {sum(catalog['structure_count'] for catalog in catalogs)}  ",
        f"**Records:** {complete + partial} ({complete} with no remaining gap, {partial} partial)  ",
        f"**Motion assets:** {motion_total} ({provenance_sentence({'measured_provenance': dict(totals)})})",
        "",
        "| Reference family | Representative screen | Scope | Records | Motion provenance |",
        "|---|---|---|---|---|",
    ]
    for catalog in catalogs:
        index = catalog["full_reference_catalog"]
        if catalog.get("scaffolded") or not catalog["examples"]:
            rows.append(
                f'| [{escape_cell(catalog["title"])}]({catalog["catalog"]}/full-reference.md) | — '
                f'| {escape_cell(catalog["description"])} '
                f'| [{index["complete_count"]} complete / {index["partial_count"]} partial]'
                f'({catalog["catalog"]}/references.json) | scaffolded, no records yet |'
            )
            continue
        example = catalog["examples"][0]
        image = (
            f'<a href="{catalog["catalog"]}/README.md"><img src="{catalog["catalog"]}/'
            f'{example["visual"]["local_path"]}" alt="{escape_cell(catalog["title"])} '
            'representative interface reference" width="260"></a>'
        )
        rows.append(
            f'| [{escape_cell(catalog["title"])}]({catalog["catalog"]}/full-reference.md) | {image} '
            f'| {escape_cell(catalog["description"])} '
            f'| [{index["complete_count"]} complete / {index["partial_count"]} partial]'
            f'({catalog["catalog"]}/references.json) | {provenance_sentence(index)} |'
        )
    rows.extend(
        (
            "",
            "Open a numbered per-product record for its motion evidence, named states, observed "
            "first-success journey, interactions, recovery, accessibility, and provenance. Read the "
            "family synthesis only after the underlying records.",
            "",
            "Attribution and product ownership remain with the linked source.",
            "",
        )
    )
    return "\n".join(rows)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate without writing")
    args = parser.parse_args()

    slugs = discovered_catalogs()
    catalogs = [load_catalog(slug) for slug in slugs]

    if args.check:
        for catalog in catalogs:
            index = catalog["full_reference_catalog"]
            print(
                f"{catalog['catalog']}: {index['complete_count']} complete, "
                f"{index['partial_count']} partial, {provenance_sentence(index)}"
            )
        return 0

    for catalog in catalogs:
        (ROOT / catalog["catalog"] / "README.md").write_text(render_readme(catalog))

    index = {
        "schema": CATALOG_SCHEMA,
        "generated_at": max(catalog["curated_at"] for catalog in catalogs),
        "catalog_count": len(catalogs),
        "example_count": sum(catalog["count"] for catalog in catalogs),
        "image_count": sum(catalog["visual_count"] for catalog in catalogs),
        "structure_count": sum(catalog["structure_count"] for catalog in catalogs),
        "record_count": sum(
            catalog["full_reference_catalog"]["reference_count"] for catalog in catalogs
        ),
        "complete_record_count": sum(
            catalog["full_reference_catalog"]["complete_count"] for catalog in catalogs
        ),
        "partial_record_count": sum(
            catalog["full_reference_catalog"]["partial_count"] for catalog in catalogs
        ),
        "locally_driven_motion_count": sum(
            catalog["full_reference_catalog"]["locally_driven_count"] for catalog in catalogs
        ),
        "catalogs": [
            {
                "slug": catalog["catalog"],
                "title": catalog["title"],
                "description": catalog["description"],
                "count": catalog["count"],
                "image_count": catalog["visual_count"],
                "structure_count": catalog["structure_count"],
                "complete_record_count": catalog["full_reference_catalog"]["complete_count"],
                "partial_record_count": catalog["full_reference_catalog"]["partial_count"],
                "measured_provenance": catalog["full_reference_catalog"]["measured_provenance"],
                "source": f"{catalog['catalog']}/sources.json",
                "readme": f"{catalog['catalog']}/README.md",
                "full_reference": f"{catalog['catalog']}/full-reference.md",
                "full_reference_source": f"{catalog['catalog']}/references.json",
            }
            for catalog in catalogs
        ],
    }
    (ROOT / "example-catalogs.json").write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n")
    (ROOT / "example-catalogs.md").write_text(render_index(catalogs))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
