#!/usr/bin/env python3
"""Measure reference accessibility with axe-core through Weles on a Stado host."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Callable, Iterable, Sequence
from urllib.parse import urlsplit, urlunsplit

ROOT = Path(__file__).resolve().parent
PLAN_DIR = Path.home() / ".stado" / "work" / "accessibility-audit-plans"
STAGING_ROOT = Path.home() / ".stado" / "work" / "accessibility-audits"
INDEX = ROOT / "accessibility-audit-index.json"
VERIFIER = ROOT / "verify-reference-evidence.py"

PLAN_SCHEMA = "wisent.weles-capture-plan.v1"
INDEX_SCHEMA = "wisent.accessibility-audit-index.v1"
ACTION = "generic_accessibility_audit"
NAMESPACE = "stado://weles-captures/"
DEFAULT_TARGET = "charless-mac-mini"
DEFAULT_CATALOGS = [
    "web-app-examples",
    "dashboard-console-examples",
    "documentation-site-examples",
    "design-system-examples",
    "onboarding-auth-examples",
]
ACTION_KEYS = {"batch", "site_slug", "source_url", "viewport", "artifact_prefix"}
PLAN_KEYS = {"schema", "batch", "target", "captures"}
SUMMARY_FIELDS = [
    "source_url",
    "viewport",
    "captured_at",
    "renderer",
    "weles_version",
    "axe_version",
    "violation_count",
    "violations",
    "passes_count",
    "incomplete_count",
    "bytes",
    "sha256",
]
VIEWPORT = {"width": 1440, "height": 1000, "device_scale_factor": 1}


class AuditError(RuntimeError):
    pass


@dataclass(frozen=True)
class Reference:
    catalog: str
    index: int
    name: str
    slug: str
    path: Path
    source_url: str

    @property
    def id(self) -> str:
        return f"{self.catalog}/{self.slug}"

    def action(self, batch: str) -> dict:
        return {
            "batch": batch,
            "site_slug": self.slug,
            "source_url": self.source_url,
            "viewport": dict(VIEWPORT),
            "artifact_prefix": f"{NAMESPACE}{batch}/{self.catalog}/{self.slug}/accessibility/",
        }


# Match capture-landing-pages.py: preserve the first useful Stado refusal rather
# than replacing it with the usage banner printed after it.
def stado(*args: str, parse_json: bool = False) -> object:
    if shutil.which("stado") is None:
        raise AuditError("stado is not on PATH; hosts are reached through stado, never ssh")
    proc = subprocess.run(["stado", *args], capture_output=True, text=True)
    if proc.returncode != 0:
        lines = [line.strip() for line in (proc.stderr or proc.stdout).splitlines() if line.strip()]
        said = [line for line in lines if line.lower().startswith(("error", "warning"))]
        detail = said or lines
        tail = " | ".join(detail[:4]) if detail else f"exit {proc.returncode}"
        raise AuditError(f"stado {' '.join(args)}: {tail}")
    if not parse_json:
        return proc.stdout
    return load_json(proc.stdout, " ".join(("stado", *args)))


def load_json(text: str, what: str) -> object:
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        start, end = text.find("{"), text.rfind("}")
        if start >= 0 and end > start:
            try:
                return json.loads(text[start:end + 1])
            except json.JSONDecodeError:
                pass
        raise AuditError(f"{what}: expected JSON on stdout, got {text.strip()[:200]!r}")


def strict_json(text: str, what: str) -> object:
    def object_pairs(pairs: list[tuple[str, object]]) -> dict:
        result: dict = {}
        for key, value in pairs:
            if key in result:
                raise AuditError(f"{what}: duplicate key {key!r}")
            result[key] = value
        return result

    try:
        return json.loads(text, object_pairs_hook=object_pairs)
    except json.JSONDecodeError as exc:
        raise AuditError(f"{what}: not readable JSON: {exc}") from exc


def pick(row: dict, *names: str, default: object = None) -> object:
    for name in names:
        if isinstance(row, dict) and row.get(name) is not None:
            return row[name]
    return default


def action_rows(payload: object, what: str) -> list[dict]:
    if isinstance(payload, list):
        return [row for row in payload if isinstance(row, dict)]
    if isinstance(payload, dict):
        for key in ("actions", "jobs", "captures", "items", "results"):
            value = payload.get(key)
            if isinstance(value, list):
                return [row for row in value if isinstance(row, dict)]
    raise AuditError(f"{what}: no per-action list in the response")


def action_id(row: dict) -> str | None:
    value = pick(row, "action_id", "actionId", "id", "job_id", "jobId")
    return str(value) if value is not None else None


def canonical_url(value: object, record_id: str, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AuditError(f"{record_id}: {field}: expected a non-empty http(s) URL")
    try:
        parts = urlsplit(value)
    except ValueError as exc:
        raise AuditError(f"{record_id}: {field}: expected an http(s) URL: {exc}") from exc
    if parts.scheme not in ("http", "https") or not parts.netloc:
        raise AuditError(f"{record_id}: {field}: expected an http(s) URL, got {value!r}")
    path = parts.path or "/"
    return urlunsplit((parts.scheme, parts.netloc, path, parts.query, parts.fragment))


def parse_record_selection(raw: str | None) -> set[int] | None:
    if raw is None:
        return None
    selected: set[int] = set()
    for token in raw.split(","):
        token = token.strip()
        if not token:
            raise AuditError("--records: empty item in the comma-separated selection")
        match = re.fullmatch(r"([1-9]\d*)(?:-([1-9]\d*))?", token)
        if not match:
            raise AuditError(f"--records: {token!r} is not a positive record number or range")
        first = int(match.group(1))
        last = int(match.group(2) or first)
        if last < first:
            raise AuditError(f"--records: descending range {token!r} is not allowed")
        selected.update(range(first, last + 1))
    return selected


def normalize_catalog(value: str) -> str:
    catalog = value.strip()
    if not catalog:
        raise AuditError("--catalog: catalog name must not be empty")
    if not catalog.endswith("-examples"):
        catalog += "-examples"
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]*-examples", catalog):
        raise AuditError(f"--catalog: invalid catalog name {value!r}")
    return catalog


def load_references(catalogs: Sequence[str], selection: set[int] | None) -> list[Reference]:
    references: list[Reference] = []
    for catalog in catalogs:
        catalog_dir = ROOT / catalog
        catalog_path = catalog_dir / "references.json"
        if not catalog_path.is_file():
            raise AuditError(f"{catalog}: references.json: catalog does not exist")
        payload = strict_json(catalog_path.read_text(), f"{catalog}: references.json")
        if not isinstance(payload, dict) or not isinstance(payload.get("references"), list):
            raise AuditError(f"{catalog}: references.json: references must be an array")
        known: set[int] = set()
        for position, pointer in enumerate(payload["references"], 1):
            pointer_id = f"{catalog}/record-{position}"
            if not isinstance(pointer, dict):
                raise AuditError(f"{pointer_id}: catalog entry: expected an object")
            index = pointer.get("index")
            if not isinstance(index, int) or isinstance(index, bool) or index < 1:
                raise AuditError(f"{pointer_id}: index: expected a positive integer")
            if index in known:
                raise AuditError(f"{catalog}/{index}: index: duplicate catalog record")
            known.add(index)
            if selection is not None and index not in selection:
                continue
            relative = pointer.get("path")
            record_id = f"{catalog}/{index:02d}"
            if not isinstance(relative, str) or not relative:
                raise AuditError(f"{record_id}: path: expected a reference path")
            path = (catalog_dir / relative).resolve()
            if not path.is_relative_to(catalog_dir.resolve()) or path.name != "reference.json":
                raise AuditError(f"{record_id}: path: {relative!r} is outside the catalog reference layout")
            if not path.is_file():
                raise AuditError(f"{record_id}: path: {relative!r} does not exist")
            document = strict_json(path.read_text(), f"{record_id}: reference.json")
            if not isinstance(document, dict):
                raise AuditError(f"{record_id}: reference.json: expected an object")
            slug = path.parent.name
            if not re.fullmatch(r"[a-z0-9][a-z0-9._-]{0,80}", slug):
                raise AuditError(f"{record_id}: site_slug: directory name {slug!r} is not a Weles slug")
            name = document.get("name")
            if not isinstance(name, str) or not name:
                raise AuditError(f"{record_id}: name: expected a non-empty string")
            source_field = "product_url" if document.get("product_url") is not None else "source_url"
            source_url = canonical_url(document.get(source_field), record_id, source_field)
            references.append(Reference(catalog, index, name, slug, path, source_url))
        if selection is not None:
            missing = sorted(selection - known)
            if missing:
                joined = ", ".join(str(value) for value in missing)
                raise AuditError(f"{catalog}: --records: record(s) {joined} do not exist")
    return references


def validate_plan(document: object, expected_target: str, expected: Sequence[Reference]) -> dict:
    if not isinstance(document, dict):
        raise AuditError("plan: document must be a JSON object")
    keys = set(document)
    if keys != PLAN_KEYS:
        raise AuditError(f"plan: keys must be exactly {', '.join(sorted(PLAN_KEYS))}; got {', '.join(sorted(keys))}")
    if document.get("schema") != PLAN_SCHEMA:
        raise AuditError(f"plan: schema must be {PLAN_SCHEMA}")
    batch = document.get("batch")
    if not isinstance(batch, str) or not re.fullmatch(r"[a-z0-9][a-z0-9._-]{0,80}", batch):
        raise AuditError("plan: batch must be a lowercase Weles slug")
    if document.get("target") != expected_target:
        raise AuditError(f"plan: target must be {expected_target!r}")
    captures = document.get("captures")
    if not isinstance(captures, list) or not captures:
        raise AuditError("plan: captures must be a non-empty array")
    if len(captures) != len(expected):
        raise AuditError(f"plan: expected {len(expected)} actions, got {len(captures)}")
    prefixes: set[str] = set()
    for position, (action, reference) in enumerate(zip(captures, expected), 1):
        label = f"{reference.id}: action {position}"
        if not isinstance(action, dict):
            raise AuditError(f"{label}: expected an object")
        if set(action) != ACTION_KEYS:
            raise AuditError(f"{label}: keys must be exactly {', '.join(sorted(ACTION_KEYS))}")
        expected_action = reference.action(batch)
        for field in ACTION_KEYS:
            if action.get(field) != expected_action[field]:
                raise AuditError(f"{label}: {field}: expected {expected_action[field]!r}, got {action.get(field)!r}")
        prefix = action["artifact_prefix"]
        if prefix in prefixes:
            raise AuditError(f"{label}: artifact_prefix: duplicate prefix {prefix!r}")
        prefixes.add(prefix)
    return document


def atomic_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.part")
    temporary.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
    temporary.replace(path)


def plan_path_for(args: argparse.Namespace, batch: str) -> Path:
    path = args.plan or PLAN_DIR / f"{batch}.json"
    path = path.expanduser().resolve()
    work = (Path.home() / ".stado" / "work").resolve()
    if not path.is_relative_to(work):
        raise AuditError(f"--plan: {path} is outside {work}; plans belong under ~/.stado/work")
    return path


def enqueue(target: str, path: Path, plan: dict) -> tuple[str, list[str]]:
    payload = stado("host", "weles-capture", target, "--plan", str(path), "--json", parse_json=True)
    rows = action_rows(payload, "weles-capture")
    captures = plan["captures"]
    if len(rows) != len(captures):
        raise AuditError(
            f"weles-capture enqueued {len(rows)} actions for a plan of {len(captures)}; "
            "refusing to attribute artifacts to records on a mismatched list"
        )
    if isinstance(payload, dict):
        returned_action = payload.get("action")
        if returned_action is not None and returned_action != ACTION:
            raise AuditError(f"weles-capture: action: expected {ACTION}, got {returned_action!r}")
        returned_batch = str(pick(payload, "batch", "batch_id", "id", default=plan["batch"]))
    else:
        returned_batch = plan["batch"]
    if returned_batch != plan["batch"]:
        raise AuditError(f"weles-capture: batch: expected {plan['batch']!r}, got {returned_batch!r}")
    ids: list[str] = []
    for position, (row, capture) in enumerate(zip(rows, captures), 1):
        identifier = action_id(row)
        if identifier is None:
            raise AuditError(f"weles-capture action {position}: action_id: missing")
        if row.get("site_slug") is not None and row["site_slug"] != capture["site_slug"]:
            raise AuditError(
                f"weles-capture action {position}: site_slug: expected {capture['site_slug']!r}, "
                f"got {row['site_slug']!r}"
            )
        if row.get("artifact_prefix") is not None and row["artifact_prefix"] != capture["artifact_prefix"]:
            raise AuditError(
                f"weles-capture action {position}: artifact_prefix: expected {capture['artifact_prefix']!r}, "
                f"got {row['artifact_prefix']!r}"
            )
        ids.append(identifier)
    if len(set(ids)) != len(ids):
        raise AuditError("weles-capture: action_id: duplicate ids prevent record attribution")
    return returned_batch, ids


def poll(target: str, batch: str, expected_ids: set[str], interval: int, timeout: int,
         log: Callable[[str], None]) -> dict[str, dict]:
    terminal = {"done", "failed", "error", "cancelled", "canceled", "skipped"}
    deadline = time.monotonic() + timeout
    latest: dict[str, dict] = {}
    while True:
        payload = stado("host", "weles-capture-status", target, "--batch", batch, "--json", parse_json=True)
        if isinstance(payload, dict):
            returned_action = payload.get("action")
            if returned_action is not None and returned_action != ACTION:
                raise AuditError(f"weles-capture-status: action: expected {ACTION}, got {returned_action!r}")
        rows = action_rows(payload, "weles-capture-status")
        latest = {}
        for row in rows:
            identifier = action_id(row)
            if identifier in expected_ids:
                latest[identifier] = row
        counts: dict[str, int] = {}
        for row in latest.values():
            state = str(pick(row, "state", "status", default="unknown")).lower()
            counts[state] = counts.get(state, 0) + 1
        log("  " + ", ".join(f"{key}={value}" for key, value in sorted(counts.items())) +
            f" ({len(latest)}/{len(expected_ids)})")
        if len(latest) == len(expected_ids) and all(
            str(pick(row, "state", "status", default="unknown")).lower() in terminal
            for row in latest.values()
        ):
            return latest
        if time.monotonic() > deadline:
            log(f"  timed out after {timeout}s; unresolved actions remain pending")
            return latest
        time.sleep(interval)


def artifact_keys(row: dict) -> list[str]:
    raw = pick(row, "artifacts", "artefacts", "objects", "keys", default=[])
    if isinstance(raw, str):
        raw = [raw]
    result: list[str] = []
    for item in raw or []:
        if isinstance(item, str):
            result.append(item)
        elif isinstance(item, dict):
            value = pick(item, "key", "uri", "url", "artifact", "artefact", "object", "path")
            if value is not None:
                result.append(str(value))
    return result


def named_artifact(keys: Iterable[str], name: str, reference: Reference) -> str:
    matches = [key for key in keys if PurePosixPath(key).name == name]
    if len(matches) != 1:
        raise AuditError(f"{reference.id}: artifacts.{name}: expected exactly one storage object, got {len(matches)}")
    return matches[0]


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fetch(key: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.unlink(missing_ok=True)
    stado("storage", "get", key, str(destination))
    if not destination.is_file():
        raise AuditError(f"stado storage get {key}: nothing was written to {destination}")


def require_summary_field(summary: dict, field: str, reference: Reference) -> object:
    if field not in summary:
        raise AuditError(f"{reference.id}: axe-summary.json.{field}: field is missing")
    return summary[field]


def nonempty_text(value: object, field: str, reference: Reference) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AuditError(f"{reference.id}: axe-summary.json.{field}: expected a non-empty string")
    return value


def count_field(summary: dict, field: str, reference: Reference) -> int:
    value = require_summary_field(summary, field, reference)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise AuditError(f"{reference.id}: axe-summary.json.{field}: expected a non-negative integer")
    return value


def validate_artifacts(reference: Reference, action: dict, raw_path: Path, summary_path: Path) -> dict:
    try:
        summary = strict_json(summary_path.read_text(), f"{reference.id}: axe-summary.json")
    except UnicodeDecodeError as exc:
        raise AuditError(f"{reference.id}: axe-summary.json: not UTF-8: {exc}") from exc
    if not isinstance(summary, dict):
        raise AuditError(f"{reference.id}: axe-summary.json: expected an object")
    for field in SUMMARY_FIELDS:
        require_summary_field(summary, field, reference)
    if summary["source_url"] != action["source_url"]:
        raise AuditError(
            f"{reference.id}: axe-summary.json.source_url: expected {action['source_url']!r}, "
            f"got {summary['source_url']!r}"
        )
    if summary["viewport"] != action["viewport"]:
        raise AuditError(
            f"{reference.id}: axe-summary.json.viewport: expected {action['viewport']!r}, "
            f"got {summary['viewport']!r}"
        )
    for field in ("captured_at", "renderer", "weles_version", "axe_version"):
        nonempty_text(summary[field], field, reference)
    raw_size = raw_path.stat().st_size
    raw_hash = sha256_of(raw_path)
    expected_size = summary["bytes"]
    if not isinstance(expected_size, int) or isinstance(expected_size, bool) or expected_size < 0:
        raise AuditError(f"{reference.id}: axe-summary.json.bytes: expected a non-negative integer")
    if raw_size != expected_size:
        raise AuditError(
            f"{reference.id}: axe-summary.json.bytes: downloaded axe.json has {raw_size} bytes, "
            f"summary records {expected_size}"
        )
    expected_hash = summary["sha256"]
    if not isinstance(expected_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", expected_hash):
        raise AuditError(f"{reference.id}: axe-summary.json.sha256: expected 64 lowercase hex characters")
    if raw_hash != expected_hash:
        raise AuditError(
            f"{reference.id}: axe-summary.json.sha256: downloaded axe.json hashes to {raw_hash}, "
            f"summary records {expected_hash}"
        )
    violation_count = count_field(summary, "violation_count", reference)
    passes_count = count_field(summary, "passes_count", reference)
    incomplete_count = count_field(summary, "incomplete_count", reference)
    violations = summary["violations"]
    if not isinstance(violations, list):
        raise AuditError(f"{reference.id}: axe-summary.json.violations: expected an array")
    if len(violations) != violation_count:
        raise AuditError(
            f"{reference.id}: axe-summary.json.violation_count: records {violation_count}, "
            f"but violations has {len(violations)} entries"
        )
    for position, violation in enumerate(violations):
        field = f"violations[{position}]"
        if not isinstance(violation, dict):
            raise AuditError(f"{reference.id}: axe-summary.json.{field}: expected an object")
        if not isinstance(violation.get("id"), str) or not violation["id"]:
            raise AuditError(f"{reference.id}: axe-summary.json.{field}.id: expected a non-empty string")
        impact = violation.get("impact")
        if impact is not None and not isinstance(impact, str):
            raise AuditError(f"{reference.id}: axe-summary.json.{field}.impact: expected a string or null")
        if not isinstance(violation.get("help"), str):
            raise AuditError(f"{reference.id}: axe-summary.json.{field}.help: expected a string")
        nodes = violation.get("node_count")
        if not isinstance(nodes, int) or isinstance(nodes, bool) or nodes < 0:
            raise AuditError(
                f"{reference.id}: axe-summary.json.{field}.node_count: expected a non-negative integer"
            )
    try:
        raw = strict_json(raw_path.read_text(), f"{reference.id}: axe.json")
    except UnicodeDecodeError as exc:
        raise AuditError(f"{reference.id}: axe.json: not UTF-8: {exc}") from exc
    if not isinstance(raw, dict):
        raise AuditError(f"{reference.id}: axe.json: expected an object")
    for field, count in (("violations", violation_count), ("passes", passes_count), ("incomplete", incomplete_count)):
        value = raw.get(field)
        if not isinstance(value, list):
            raise AuditError(f"{reference.id}: axe.json.{field}: expected an array")
        if len(value) != count:
            raise AuditError(
                f"{reference.id}: axe.json.{field}: has {len(value)} entries, summary records {count}"
            )
    summary["bytes"] = raw_size
    summary["sha256"] = raw_hash
    return summary


def install_artifacts(reference: Reference, raw_stage: Path, summary_stage: Path) -> tuple[Path, Path]:
    directory = reference.path.parent / "media" / "accessibility"
    directory.mkdir(parents=True, exist_ok=True)
    raw_path = directory / "axe.json"
    summary_path = directory / "axe-summary.json"
    raw_part = directory / f".axe.json.{os.getpid()}.part"
    summary_part = directory / f".axe-summary.json.{os.getpid()}.part"
    shutil.copyfile(raw_stage, raw_part)
    shutil.copyfile(summary_stage, summary_part)
    raw_part.replace(raw_path)
    summary_part.replace(summary_path)
    return raw_path, summary_path


def axe_observations(summary: dict) -> list[str]:
    version = summary["axe_version"]
    viewport = summary["viewport"]
    observations = [
        f"[axe-core] axe-core {version} reported {summary['violation_count']} violation rules, "
        f"{summary['passes_count']} passing rules, and {summary['incomplete_count']} incomplete rules "
        f"against the live product at {viewport['width']}x{viewport['height']}@"
        f"{viewport['device_scale_factor']} on {summary['captured_at']}."
    ]
    for violation in summary["violations"]:
        if not isinstance(violation, dict):
            continue
        rule = str(violation.get("id", "")) or "unnamed-rule"
        impact = violation.get("impact")
        impact_text = str(impact) if impact is not None else "impact not reported"
        nodes = violation.get("node_count", 0)
        help_text = str(violation.get("help", "")).strip()
        suffix = f": {help_text}" if help_text else ""
        observations.append(
            f"[axe-core] Rule {rule} ({impact_text}) affected {nodes} nodes{suffix}."
        )
    return observations


def update_record(reference: Reference, summary: dict) -> tuple[str, str]:
    document = strict_json(reference.path.read_text(), f"{reference.id}: reference.json")
    if not isinstance(document, dict):
        raise AuditError(f"{reference.id}: reference.json: expected an object")
    current_field = "product_url" if document.get("product_url") is not None else "source_url"
    current_url = canonical_url(document.get(current_field), reference.id, current_field)
    if current_url != reference.source_url:
        raise AuditError(
            f"{reference.id}: {current_field}: changed from {reference.source_url!r} to {current_url!r} while audit ran"
        )
    accessibility = document.get("accessibility")
    if accessibility is None:
        accessibility = {}
    if not isinstance(accessibility, dict):
        raise AuditError(f"{reference.id}: accessibility: expected an object")
    observations = accessibility.get("observations", [])
    if not isinstance(observations, list) or any(not isinstance(item, str) for item in observations):
        raise AuditError(f"{reference.id}: accessibility.observations: expected an array of strings")
    unknowns = accessibility.get("unknowns", [])
    if not isinstance(unknowns, list) or any(not isinstance(item, str) for item in unknowns):
        raise AuditError(f"{reference.id}: accessibility.unknowns: expected an array of strings")
    observations = [item for item in observations if not item.startswith("[axe-core]")]
    observations.extend(axe_observations(summary))
    accessibility["observations"] = observations
    accessibility["unknowns"] = unknowns
    accessibility["measured"] = True
    accessibility["measurement"] = {
        "tool": "axe-core",
        "version": summary["axe_version"],
        "captured_at": summary["captured_at"],
        "renderer": summary["renderer"],
        "weles_version": summary["weles_version"],
        "source_url": summary["source_url"],
        "viewport": summary["viewport"],
        "raw_path": "media/accessibility/axe.json",
        "summary_path": "media/accessibility/axe-summary.json",
        "raw_bytes": summary["bytes"],
        "raw_sha256": summary["sha256"],
        "violation_count": summary["violation_count"],
        "passes_count": summary["passes_count"],
        "incomplete_count": summary["incomplete_count"],
    }
    document["accessibility"] = accessibility
    atomic_json(reference.path, document)
    return accessibility["measurement"]["raw_path"], accessibility["measurement"]["summary_path"]


def retrieve(reference: Reference, action: dict, row: dict, batch: str) -> dict:
    if row.get("site_slug") is not None and row["site_slug"] != reference.slug:
        raise AuditError(
            f"{reference.id}: status.site_slug: expected {reference.slug!r}, got {row['site_slug']!r}"
        )
    if row.get("artifact_prefix") is not None and row["artifact_prefix"] != action["artifact_prefix"]:
        raise AuditError(
            f"{reference.id}: status.artifact_prefix: expected {action['artifact_prefix']!r}, "
            f"got {row['artifact_prefix']!r}"
        )
    keys = artifact_keys(row)
    raw_key = named_artifact(keys, "axe.json", reference)
    summary_key = named_artifact(keys, "axe-summary.json", reference)
    stage = STAGING_ROOT / batch / reference.catalog / reference.slug
    raw_stage = stage / "axe.json"
    summary_stage = stage / "axe-summary.json"
    try:
        fetch(summary_key, summary_stage)
    except (AuditError, OSError) as exc:
        raise AuditError(f"{reference.id}: staging.axe-summary.json: {exc}") from exc
    try:
        fetch(raw_key, raw_stage)
    except (AuditError, OSError) as exc:
        raise AuditError(f"{reference.id}: staging.axe.json: {exc}") from exc
    try:
        summary = validate_artifacts(reference, action, raw_stage, summary_stage)
    except OSError as exc:
        raise AuditError(f"{reference.id}: staged artifacts: {exc}") from exc
    try:
        raw_path, summary_path = install_artifacts(reference, raw_stage, summary_stage)
    except OSError as exc:
        raise AuditError(f"{reference.id}: media/accessibility: {exc}") from exc
    try:
        raw_relative, summary_relative = update_record(reference, summary)
    except OSError as exc:
        raise AuditError(f"{reference.id}: reference.json: {exc}") from exc
    return {
        "id": reference.id,
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "complete",
        "reason": None,
        "raw_path": str(raw_path.relative_to(ROOT)),
        "summary_path": str(summary_path.relative_to(ROOT)),
        "record_raw_path": raw_relative,
        "record_summary_path": summary_relative,
        "raw_storage_key": raw_key,
        "summary_storage_key": summary_key,
    }


def initial_row(reference: Reference, action: dict) -> dict:
    return {
        "id": reference.id,
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "pending",
        "artifact_prefix": action["artifact_prefix"],
        "raw_path": None,
        "summary_path": None,
        "reason": "not dispatched",
    }


def write_index(rows: Sequence[dict], batch: str, target: str, plan_path: Path,
                verifier_errors: Sequence[str]) -> dict:
    totals = {
        "planned": len(rows),
        "complete": sum(row.get("status") == "complete" for row in rows),
        "failed": sum(row.get("status") == "failed" for row in rows),
        "pending": sum(row.get("status") == "pending" for row in rows),
    }
    payload = {
        "schema": INDEX_SCHEMA,
        "generated_at": now(),
        "batch": batch,
        "target": target,
        "plan": str(plan_path),
        "totals": totals,
        "records": list(rows),
        "verifier_errors": list(verifier_errors),
    }
    atomic_json(INDEX, payload)
    return payload


def run_verifier(catalog: str) -> None:
    proc = subprocess.run(
        [sys.executable, str(VERIFIER), "--catalog", catalog, "--apply", "--no-state-match"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        lines = [line.strip() for line in (proc.stderr or proc.stdout).splitlines() if line.strip()]
        detail = " | ".join(lines[:4]) if lines else f"exit {proc.returncode}"
        raise AuditError(f"verify-reference-evidence.py --catalog {catalog}: {detail}")


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def default_batch() -> str:
    return datetime.now(timezone.utc).strftime("accessibility-%Y%m%dt%H%M%Sz")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--catalog",
        action="append",
        help="catalog family, repeatable (default: browser-reachable catalog families)",
    )
    parser.add_argument("--records", help="comma-separated record numbers and inclusive ranges")
    parser.add_argument("--batch", help="batch id (default: accessibility-<UTC timestamp>)")
    parser.add_argument("--target", default=DEFAULT_TARGET, help=f"Stado host (default: {DEFAULT_TARGET})")
    parser.add_argument("--plan", type=Path, help=f"plan path under {PLAN_DIR}")
    parser.add_argument("--dry-run", action="store_true", help="write and print the plan; contact no host")
    parser.add_argument("--poll-seconds", type=int, default=15)
    parser.add_argument("--timeout-minutes", type=int, default=120)
    args = parser.parse_args()

    try:
        if args.poll_seconds < 1:
            raise AuditError("--poll-seconds: must be at least 1")
        if args.timeout_minutes < 1:
            raise AuditError("--timeout-minutes: must be at least 1")
        catalogs = [normalize_catalog(value) for value in (args.catalog or DEFAULT_CATALOGS)]
        catalogs = list(dict.fromkeys(catalogs))
        selection = parse_record_selection(args.records)
        references = load_references(catalogs, selection)
        if not references:
            raise AuditError("selection: no records selected")
        batch = args.batch or default_batch()
        plan = {
            "schema": PLAN_SCHEMA,
            "batch": batch,
            "target": args.target,
            "captures": [reference.action(batch) for reference in references],
        }
        plan = validate_plan(plan, args.target, references)
        plan_path = plan_path_for(args, batch)
        atomic_json(plan_path, plan)
        parsed = strict_json(plan_path.read_text(), "plan")
        plan = validate_plan(parsed, args.target, references)
    except AuditError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.dry_run:
        json.dump(plan, sys.stdout, indent=2, ensure_ascii=False)
        sys.stdout.write("\n")
        print(
            f"dry run: planned={len(references)} complete=0 failed=0 pending={len(references)}; "
            f"no host was contacted; plan={plan_path}",
            file=sys.stderr,
        )
        return 0

    log = lambda line: print(line, file=sys.stderr, flush=True)
    rows = [initial_row(reference, action) for reference, action in zip(references, plan["captures"])]
    verifier_errors: list[str] = []
    try:
        batch, ids = enqueue(args.target, plan_path, plan)
        log(f"batch {batch}: {len(ids)} {ACTION} actions enqueued")
        states = poll(args.target, batch, set(ids), args.poll_seconds, args.timeout_minutes * 60, log)
    except AuditError as exc:
        reason = str(exc)
        for row in rows:
            row["reason"] = reason
        payload = write_index(rows, batch, args.target, plan_path, verifier_errors)
        log(f"error: {reason}")
        log(f"{INDEX.name}: {payload['totals']}")
        return 2

    completed_catalogs: set[str] = set()
    for position, (reference, action, identifier) in enumerate(zip(references, plan["captures"], ids)):
        state_row = states.get(identifier)
        if state_row is None:
            rows[position]["reason"] = f"action {identifier} had not reported a state when polling stopped"
            continue
        state = str(pick(state_row, "state", "status", default="unknown")).lower()
        if state != "done":
            error = pick(state_row, "error", "message", "reason")
            exact_error = str(error) if error is not None and str(error) else f"action ended in state {state}"
            rows[position]["status"] = "failed"
            rows[position]["reason"] = exact_error
            log(f"FAILED {reference.id}: action.error: {exact_error}")
            continue
        try:
            rows[position] = retrieve(reference, action, state_row, batch)
            completed_catalogs.add(reference.catalog)
            log(f"COMPLETE {reference.id}: {rows[position]['raw_path']}")
        except (AuditError, OSError) as exc:
            reason = str(exc) if isinstance(exc, AuditError) else f"{reference.id}: filesystem: {exc}"
            rows[position]["status"] = "failed"
            rows[position]["reason"] = reason
            log(f"REFUSED {reason}")

    for catalog in sorted(completed_catalogs):
        try:
            run_verifier(catalog)
            log(f"verified {catalog} with --apply --no-state-match")
        except AuditError as exc:
            verifier_errors.append(str(exc))
            log(f"error: {exc}")

    payload = write_index(rows, batch, args.target, plan_path, verifier_errors)
    totals = payload["totals"]
    log(
        f"{INDEX.name}: planned={totals['planned']} complete={totals['complete']} "
        f"failed={totals['failed']} pending={totals['pending']}"
    )
    return 0 if totals["failed"] == 0 and totals["pending"] == 0 and not verifier_errors else 3


if __name__ == "__main__":
    raise SystemExit(main())
