#!/usr/bin/env python3
"""Measure the stored reference evidence and rewrite every record to what the files prove.

The reference corpus records motion, state, journey, interaction, and accessibility
evidence per product. Those records were written by hand and drifted from the media
beside them: media kinds used two vocabularies, static images were recorded as motion,
durations were missing, hashes were never re-checked, and every record claimed
`evidence_status: complete` regardless of what it contained.

This utility is the measurement. It reads every `references/*/reference.json`, probes
the real media with ffprobe (or the asciinema cast header), verifies bytes and SHA-256,
derives the media kind and the provenance class from observable facts, locates each
state frame inside its motion source by pixel comparison, and then recomputes
`evidence_status` from the measured floor in `full-reference-contract.md`.

Nothing here invents evidence. A field that cannot be measured is set to null and
named in `evidence_gaps`, and the record is not called complete.

    ./verify-reference-evidence.py            # report only
    ./verify-reference-evidence.py --apply    # rewrite reference.json + references.json
    ./verify-reference-evidence.py --catalog cli-examples --apply
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from reference_contract import (
    CONTAINER_KIND,
    INDEX_SCHEMA,
    INTERACTION_FIELDS,
    MIN_ACCESSIBILITY_OBSERVATIONS,
    MIN_INTERACTIONS,
    MIN_JOURNEY_STEPS,
    MIN_MOTION_FRAMES,
    MIN_MOTION_SECONDS,
    MIN_STATES,
    MOTION_ANALYSIS_ALIASES,
    MOTION_ANALYSIS_FIELDS,
    MOTION_ANALYSIS_OPTIONAL,
    RECORD_SCHEMA,
    STATE_MATCH_MAX_DIFF,
    STILL_KIND,
    canonical_motion_kind,
    canonical_timing_class,
    classify_provenance,
)

ROOT = Path(__file__).resolve().parent


def sh(args: list[str]) -> str:
    proc = subprocess.run(args, capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"{args[0]} failed: {proc.stderr.strip()[:300]}")
    return proc.stdout


@dataclass
class Probe:
    """What the file itself says, independent of the record."""

    exists: bool
    bytes: int | None = None
    sha256: str | None = None
    kind: str | None = None
    width: int | None = None
    height: int | None = None
    duration_seconds: float | None = None
    frame_count: int | None = None
    error: str | None = None


@dataclass
class Findings:
    changed: bool = False
    gaps: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)


def digest(path: Path) -> tuple[int, str]:
    h = hashlib.sha256()
    size = 0
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            size += len(chunk)
            h.update(chunk)
    return size, h.hexdigest()


def probe_cast(path: Path) -> Probe:
    """asciinema v2: JSON header line, then [time, kind, data] events."""
    size, sha = digest(path)
    header: dict[str, Any] = {}
    last_time = 0.0
    frames = 0
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        first = fh.readline()
        try:
            header = json.loads(first)
        except json.JSONDecodeError:
            return Probe(True, size, sha, "terminal-cast", error="unreadable cast header")
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(event, list) and event:
                frames += 1
                try:
                    last_time = max(last_time, float(event[0]))
                except (TypeError, ValueError):
                    pass
    return Probe(
        exists=True,
        bytes=size,
        sha256=sha,
        kind="terminal-cast",
        width=header.get("width"),
        height=header.get("height"),
        duration_seconds=round(last_time, 3) if last_time else None,
        frame_count=frames or None,
    )


def probe_media(path: Path) -> Probe:
    if not path.exists():
        return Probe(False, error="file missing")
    if path.suffix == ".cast":
        return probe_cast(path)

    size, sha = digest(path)
    try:
        raw = sh(
            [
                "ffprobe",
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                "-count_frames",
                str(path),
            ]
        )
    except RuntimeError as exc:
        return Probe(True, size, sha, error=str(exc))

    data = json.loads(raw)
    streams = [s for s in data.get("streams", []) if s.get("codec_type") == "video"]
    if not streams:
        return Probe(True, size, sha, error="no video stream")
    stream = streams[0]
    fmt = data.get("format", {})
    frames = stream.get("nb_read_frames") or stream.get("nb_frames")
    frames = int(frames) if frames not in (None, "N/A") else None

    duration = stream.get("duration") or fmt.get("duration")
    try:
        duration = round(float(duration), 3)
    except (TypeError, ValueError):
        duration = None
    if duration is None and frames and stream.get("avg_frame_rate", "0/0") != "0/0":
        num, _, den = stream["avg_frame_rate"].partition("/")
        try:
            rate = float(num) / float(den)
            duration = round(frames / rate, 3) if rate else None
        except (ValueError, ZeroDivisionError):
            duration = None

    kind = CONTAINER_KIND.get(fmt.get("format_name", ""), None)
    if kind in (None, STILL_KIND) and path.suffix.lower() == ".webp":
        kind = "animated-webp" if (frames or 1) > 1 else STILL_KIND
    if frames is not None and frames < MIN_MOTION_FRAMES:
        kind = STILL_KIND

    return Probe(
        exists=True,
        bytes=size,
        sha256=sha,
        kind=kind,
        width=stream.get("width"),
        height=stream.get("height"),
        duration_seconds=duration,
        frame_count=frames,
    )




def locate_state_in_motion(state: Path, motion: Path, duration: float | None) -> dict[str, Any] | None:
    """Find the timestamp in `motion` whose frame is closest to `state`.

    Deterministic: the motion is decoded once at two frames per second into 16x16
    grayscale signatures, the state image is reduced the same way, and the two are
    compared by mean absolute difference. The distance is returned with the match so
    a caller can refuse a frame that does not actually come from this motion.
    """
    if not state.exists() or not motion.exists() or motion.suffix == ".cast":
        return None
    target = still_signature(state)
    if target is None:
        return None
    frames = motion_signatures(motion)
    if not frames:
        return None
    best: tuple[float, float] | None = None
    for index, sig in enumerate(frames):
        dist = sum(abs(a - b) for a, b in zip(target, sig)) / len(sig)
        if best is None or dist < best[1]:
            best = (round(index / SAMPLE_FPS, 3), round(dist, 4))
    if best is None:
        return None
    return {
        "timestamp_seconds": best[0],
        "mean_abs_diff": best[1],
        "sampled_frames": len(frames),
    }


SAMPLE_FPS = 2.0
SIG_BYTES = 256
_MOTION_CACHE: dict[str, list[tuple[int, ...]]] = {}
_STILL_CACHE: dict[str, tuple[int, ...] | None] = {}


def _raw_signatures(args: list[str]) -> list[tuple[int, ...]]:
    proc = subprocess.run(args, capture_output=True)
    if proc.returncode != 0:
        return []
    data = proc.stdout
    return [
        tuple(data[offset : offset + SIG_BYTES])
        for offset in range(0, len(data) - SIG_BYTES + 1, SIG_BYTES)
    ]


def motion_signatures(path: Path) -> list[tuple[int, ...]]:
    key = str(path)
    if key not in _MOTION_CACHE:
        _MOTION_CACHE[key] = _raw_signatures(
            [
                "ffmpeg",
                "-v",
                "error",
                "-i",
                key,
                "-vf",
                f"fps={SAMPLE_FPS:g},scale=16:16,format=gray",
                "-f",
                "rawvideo",
                "-",
            ]
        )
    return _MOTION_CACHE[key]


def still_signature(path: Path) -> tuple[int, ...] | None:
    key = str(path)
    if key not in _STILL_CACHE:
        sigs = _raw_signatures(
            [
                "ffmpeg",
                "-v",
                "error",
                "-i",
                key,
                "-frames:v",
                "1",
                "-vf",
                "scale=16:16,format=gray",
                "-f",
                "rawvideo",
                "-",
            ]
        )
        _STILL_CACHE[key] = sigs[0] if sigs else None
    return _STILL_CACHE[key]


def measure_record(record_path: Path, locate_states: bool) -> Findings:
    base = record_path.parent
    data = json.loads(record_path.read_text())
    before = json.dumps(data, sort_keys=True)
    out = Findings()
    gaps: list[str] = []

    data["schema"] = RECORD_SCHEMA

    motion_paths: list[tuple[Path, Probe]] = []
    for entry in data.get("motion", []):
        local = base / str(entry.get("local_path", ""))
        probe = probe_media(local)
        motion_paths.append((local, probe))
        declared_kind = entry.get("media_kind")
        canonical = canonical_motion_kind(declared_kind)
        entry["declared_media_kind"] = declared_kind
        if not probe.exists:
            entry["media_kind"] = canonical or "missing"
            entry["measured"] = False
            gaps.append(f"motion file missing: {entry.get('local_path')}")
            continue
        entry["media_kind"] = probe.kind or canonical or "unknown"
        entry["measured"] = True
        if entry.get("sha256") and probe.sha256 != entry["sha256"]:
            gaps.append(f"motion sha256 mismatch: {entry.get('local_path')}")
        entry["sha256"] = probe.sha256
        entry["bytes"] = probe.bytes
        entry["width"] = probe.width
        entry["height"] = probe.height
        entry["duration_seconds"] = probe.duration_seconds
        entry["frame_count"] = probe.frame_count
        entry["measurement_method"] = (
            "asciinema-v2 header and event stream" if probe.kind == "terminal-cast" else "ffprobe -count_frames"
        )
        if probe.error:
            gaps.append(f"motion probe error ({entry.get('local_path')}): {probe.error}")
        if entry["media_kind"] == STILL_KIND:
            gaps.append(f"motion asset is a still image: {entry.get('local_path')}")
        if (probe.duration_seconds or 0) < MIN_MOTION_SECONDS and probe.kind != STILL_KIND:
            gaps.append(
                f"motion shorter than {MIN_MOTION_SECONDS:g}s: {entry.get('local_path')}"
                f" ({probe.duration_seconds}s)"
            )
        entry["provenance_class"] = classify_provenance(entry.get("capture_method"), entry["media_kind"])
        if entry["provenance_class"] == "unclassified":
            gaps.append(f"motion provenance unclassified: {entry.get('local_path')}")

    primary_motion = next(((p, pr) for p, pr in motion_paths if pr.exists and pr.kind != STILL_KIND), None)

    for entry in data.get("states", []):
        local = base / str(entry.get("local_path", ""))
        probe = probe_media(local)
        if not probe.exists:
            gaps.append(f"state file missing: {entry.get('local_path')}")
            continue
        if entry.get("sha256") and probe.sha256 != entry["sha256"]:
            gaps.append(f"state sha256 mismatch: {entry.get('local_path')}")
        entry["sha256"] = probe.sha256
        entry["bytes"] = probe.bytes
        entry["width"] = probe.width
        entry["height"] = probe.height
        # v1 records already used `name` and `source_motion_path`. A failed v2
        # migration introduced parallel `state_name` / `source_relationship`
        # fields and then called every original record unnamed. Clean cutover:
        # retain one vocabulary and consume the aliases if an observer wrote one.
        alias_name = entry.pop("state_name", None)
        if alias_name:
            entry["name"] = alias_name
        if not entry.get("name"):
            gaps.append(f"state unnamed: {entry.get('local_path')}")
        alias_relationship = entry.pop("source_relationship", None)
        if alias_relationship:
            entry["source_motion_path"] = alias_relationship
        if locate_states and primary_motion is not None:
            match = locate_state_in_motion(local, primary_motion[0], primary_motion[1].duration_seconds)
            if match:
                entry["source_match"] = {
                    "motion_path": str(primary_motion[0].relative_to(base)),
                    "method": "16x16 grayscale mean-absolute-difference frame search",
                    **match,
                }
                if not entry.get("source_motion_path") and match["mean_abs_diff"] <= STATE_MATCH_MAX_DIFF:
                    entry["source_motion_path"] = (
                        f"frame of {entry['source_match']['motion_path']} at "
                        f"{match['timestamp_seconds']:g}s (mean abs diff {match['mean_abs_diff']:g}/255)"
                    )
        if not entry.get("source_motion_path"):
            gaps.append(f"state source relationship unproven: {entry.get('local_path')}")

    if len(data.get("states", [])) < MIN_STATES:
        gaps.append(f"fewer than {MIN_STATES} states")

    journey = data.get("journey") or {}
    steps = journey.get("steps") or []
    if len(steps) < MIN_JOURNEY_STEPS:
        gaps.append(f"journey exposes fewer than {MIN_JOURNEY_STEPS} observed steps")
    for key in ("actor", "goal", "prerequisites", "failure_route", "recovery_route", "completion_evidence"):
        if not journey.get(key):
            gaps.append(f"journey missing {key}")

    interactions = data.get("interactions") or []
    if len(interactions) < MIN_INTERACTIONS:
        gaps.append(f"fewer than {MIN_INTERACTIONS} mapped interactions")
    for item in interactions:
        missing = [f for f in INTERACTION_FIELDS if not item.get(f)]
        if missing:
            gaps.append(f"interaction '{item.get('name') or '?'}' missing {', '.join(missing)}")
            break

    analysis = data.get("motion_analysis")
    if not analysis:
        gaps.append("motion analysis absent")
    else:
        entries = analysis if isinstance(analysis, list) else [analysis]
        for item in entries:
            for alias, canonical in MOTION_ANALYSIS_ALIASES.items():
                if alias in item:
                    item.setdefault(canonical, item.pop(alias))
                    item.pop(alias, None)
            declared_timing = item.get("timing_class")
            canonical_timing = canonical_timing_class(declared_timing)
            if declared_timing and canonical_timing and declared_timing != canonical_timing:
                item.setdefault("timing_description", declared_timing)
            item["timing_class"] = canonical_timing
            if declared_timing and canonical_timing is None:
                gaps.append(f"motion analysis timing class unrecognized: {declared_timing}")
            for key in MOTION_ANALYSIS_FIELDS:
                item.setdefault(key, None)
            unknown = [k for k in item if k not in (*MOTION_ANALYSIS_FIELDS, *MOTION_ANALYSIS_OPTIONAL)]
            if unknown:
                gaps.append(f"motion analysis carries unknown fields {', '.join(sorted(unknown))}")
            missing = [f for f in MOTION_ANALYSIS_FIELDS if not item.get(f)]
            if missing:
                gaps.append(f"motion analysis missing {', '.join(missing)}")
                break

    access = data.get("accessibility") or {}
    if len(access.get("observations") or []) < MIN_ACCESSIBILITY_OBSERVATIONS:
        gaps.append("fewer than three accessibility observations")
    if not access.get("measured"):
        gaps.append("accessibility never measured against the product")

    classes = {e.get("provenance_class") for e in data.get("motion", []) if e.get("measured")}
    if not classes:
        gaps.append("no measured motion evidence")
    data["motion_provenance"] = sorted(c for c in classes if c)

    data["evidence_gaps"] = gaps
    data["evidence_status"] = "complete" if not gaps else "partial"
    data["measured_at"] = TODAY

    if json.dumps(data, sort_keys=True) != before:
        out.changed = True
    out.gaps = gaps
    record_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    return out


TODAY = "2026-08-19"


def catalogs(selected: str | None) -> list[Path]:
    found = sorted(p for p in ROOT.glob("*-examples") if (p / "references").is_dir())
    if selected:
        found = [p for p in found if p.name == selected]
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--no-state-match", action="store_true", help="skip the frame search")
    parser.add_argument("--jobs", type=int, default=8)
    args = parser.parse_args()

    if not args.apply:
        print("dry run: records are measured and reported, nothing is written")

    total = 0
    complete = 0
    gap_counter: dict[str, int] = {}
    for catalog in catalogs(args.catalog):
        records = sorted(catalog.glob("references/*/reference.json"))
        results: list[tuple[Path, Findings]] = []

        def work(path: Path) -> tuple[Path, Findings]:
            if args.apply:
                return path, measure_record(path, not args.no_state_match)
            data = json.loads(path.read_text())
            copy = path.parent / ".reference.measured.json"
            copy.write_text(json.dumps(data, indent=2) + "\n")
            findings = measure_record(copy, not args.no_state_match)
            measured = json.loads(copy.read_text())
            copy.unlink()
            del measured
            return path, findings

        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            results = list(pool.map(work, records))

        cat_complete = sum(1 for _, f in results if not f.gaps)
        total += len(results)
        complete += cat_complete
        for _, f in results:
            for gap in f.gaps:
                key = re.sub(r"[:(].*", "", gap).strip()
                gap_counter[key] = gap_counter.get(key, 0) + 1
        print(f"{catalog.name}: {cat_complete}/{len(results)} complete")

        index = catalog / "references.json"
        if args.apply and index.exists():
            payload = json.loads(index.read_text())
            by_path = {}
            for path, findings in results:
                by_path[f"references/{path.parent.name}/reference.json"] = findings
            for ref in payload.get("references", []):
                findings = by_path.get(ref.get("path"))
                if findings is None:
                    continue
                ref["evidence_status"] = "complete" if not findings.gaps else "partial"
                ref["evidence_gap_count"] = len(findings.gaps)
            payload["schema"] = INDEX_SCHEMA
            payload["measured_at"] = TODAY
            payload["complete_count"] = cat_complete
            payload["partial_count"] = len(results) - cat_complete
            index.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")

    print(f"\nmeasured {total} records, {complete} complete, {total - complete} partial")
    for key, count in sorted(gap_counter.items(), key=lambda kv: -kv[1]):
        print(f"{count:5d}  {key}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
