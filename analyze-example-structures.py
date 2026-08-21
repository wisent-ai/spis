#!/usr/bin/env python3
"""Measure panel geometry and record interface anatomy for example images."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
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
    "landing-page-examples",
    "wisent-product-examples",
)


def moving_average(values: np.ndarray, radius: int) -> np.ndarray:
    width = radius * 2 + 1
    if len(values) < width:
        return values
    kernel = np.ones(width, dtype=np.float32) / width
    return np.convolve(values, kernel, mode="same")


def separator_positions(gray: np.ndarray, axis: str) -> list[dict]:
    if axis == "vertical":
        differences = np.abs(np.diff(gray, axis=1)).mean(axis=0)
        extent = gray.shape[1]
    else:
        differences = np.abs(np.diff(gray, axis=0)).mean(axis=1)
        extent = gray.shape[0]
    smoothed = moving_average(differences, max(1, extent // 350))
    low = int(extent * 0.06)
    high = int(extent * 0.94)
    segment = smoothed[low:high]
    if not len(segment):
        return []
    threshold = max(float(np.percentile(segment, 91)), float(segment.mean() + segment.std() * 1.15))
    candidate_indices = [index + low for index, score in enumerate(segment) if score >= threshold]
    groups: list[list[int]] = []
    for index in candidate_indices:
        if not groups or index - groups[-1][-1] > max(2, extent // 180):
            groups.append([index])
        else:
            groups[-1].append(index)
    peaks: list[tuple[int, float]] = []
    for group in groups:
        index = max(group, key=lambda value: smoothed[value])
        peaks.append((index, float(smoothed[index])))
    selected: list[tuple[int, float]] = []
    for index, score in sorted(peaks, key=lambda item: item[1], reverse=True):
        if all(abs(index - chosen) >= extent * 0.075 for chosen, _ in selected):
            selected.append((index, score))
        if len(selected) == 4:
            break
    maximum = max((score for _, score in selected), default=1.0)
    if maximum <= 0:
        return []
    return [
        {"position": round(index / extent, 3), "strength": round(score / maximum, 3)}
        for index, score in sorted(selected)
    ]


def contains_any(text: str, words: tuple[str, ...]) -> bool:
    return any(word in text for word in words)


def semantic_hints(example: dict, catalog: str) -> dict:
    text = " ".join(
        str(example.get(key, "")) for key in ("name", "category", "selection_note")
    ).casefold()
    leading = contains_any(text, (
        "sidebar", "navigation", "navigator", "channel", "workspace", "repository",
        "server", "folder", "object", "inbox", "project panel", "service switcher",
    ))
    trailing = contains_any(text, (
        "inspector", "detail panel", "side panel", "member list", "properties",
        "customer context", "context panel", "request-response", "detail view",
    ))
    table = contains_any(text, (
        "table", "grid", "result", "list", "stream", "inventory", "queue", "timeline",
    ))
    canvas = contains_any(text, (
        "canvas", "editor", "document", "map", "diagram", "chart", "dashboard", "workspace",
    ))
    command = catalog in {"tui-examples", "cli-examples"}
    mobile = catalog in {"ios-app-examples", "android-app-examples", "app-store-listing-examples"}
    request_response = contains_any(text, (
        "request-response", "request details", "response preview", "request and response",
    ))
    return {
        "leading": leading,
        "trailing": trailing,
        "table": table,
        "canvas": canvas,
        "command": command,
        "mobile": mobile,
        "request_response": request_response,
    }


def choose_boundary(separators: list[dict], lower: float, upper: float, fallback: float | None) -> float | None:
    candidates = [item for item in separators if lower <= item["position"] <= upper]
    if candidates:
        return max(candidates, key=lambda item: item["strength"])["position"]
    return fallback


def region(role: str, position: str, bounds: tuple[float, float, float, float], evidence: str) -> dict:
    return {
        "role": role,
        "position": position,
        "bounds": {
            "x": round(bounds[0], 3),
            "y": round(bounds[1], 3),
            "width": round(bounds[2], 3),
            "height": round(bounds[3], 3),
        },
        "evidence": evidence,
    }


def classify_layout(example: dict, catalog: str, vertical: list[dict], horizontal: list[dict]) -> tuple[str, str, list[dict], str]:
    hints = semantic_hints(example, catalog)
    top = choose_boundary(horizontal, 0.06, 0.22, 0.12 if not hints["command"] else None)
    bottom = choose_boundary(horizontal, 0.76, 0.94, None)
    content_top = top or 0.0
    content_bottom = bottom or 1.0
    height = content_bottom - content_top

    leading = choose_boundary(vertical, 0.14, 0.42, 0.26 if hints["leading"] else None)
    trailing = choose_boundary(vertical, 0.58, 0.88, 0.76 if hints["trailing"] else None)
    regions: list[dict] = []
    if top is not None:
        regions.append(region("toolbar/header", "top", (0.0, 0.0, 1.0, top), "measured horizontal separator"))
    if bottom is not None:
        regions.append(region("status/action bar", "bottom", (0.0, bottom, 1.0, 1.0 - bottom), "measured horizontal separator"))

    if hints["command"]:
        split = choose_boundary(vertical, 0.25, 0.75, None)
        horizontal_split = choose_boundary(horizontal, 0.28, 0.75, None)
        if split is not None:
            regions.extend((
                region("primary terminal pane", "leading", (0.0, content_top, split, height), "measured separator in terminal image"),
                region("secondary terminal pane", "trailing", (split, content_top, 1.0 - split, height), "measured separator in terminal image"),
            ))
            return "split-terminal", "Two side-by-side terminal work areas with shared command context.", regions, "medium"
        if horizontal_split is not None:
            regions.extend((
                region("primary terminal pane", "top", (0.0, content_top, 1.0, horizontal_split - content_top), "measured separator in terminal image"),
                region("secondary terminal pane", "bottom", (0.0, horizontal_split, 1.0, content_bottom - horizontal_split), "measured separator in terminal image"),
            ))
            return "stacked-terminal", "Two vertically stacked terminal work areas.", regions, "medium"
        regions.append(region("command and output flow", "center", (0.0, content_top, 1.0, height), "terminal-family catalog"))
        return "command-output", "Single command-and-output flow without a stable secondary panel.", regions, "medium"

    if hints["request_response"]:
        navigation_end = leading or 0.0
        detail_top = choose_boundary(horizontal, max(0.28, content_top + 0.12), min(0.78, content_bottom - 0.12), 0.48)
        detail_split = choose_boundary(vertical, max(0.42, navigation_end + 0.18), 0.82, 0.62)
        if leading is not None:
            regions.append(region("traffic source navigation", "leading", (0.0, content_top, leading, height), "semantic cue plus measured boundary"))
        regions.extend((
            region("traffic request table", "upper center", (navigation_end, content_top, 1.0 - navigation_end, detail_top - content_top), "measured horizontal separator"),
            region("request inspector", "lower center", (navigation_end, detail_top, detail_split - navigation_end, content_bottom - detail_top), "request-response semantic cue plus measured separators"),
            region("response inspector", "lower trailing", (detail_split, detail_top, 1.0 - detail_split, content_bottom - detail_top), "request-response semantic cue plus measured separators"),
        ))
        return "sidebar-table-request-response", "Traffic-source navigation beside a request table, with paired request and response inspectors below.", regions, "high"

    if hints["mobile"]:
        role = "scrolling content or app screen"
        regions.append(region(role, "center", (0.0, content_top, 1.0, height), "mobile-family catalog"))
        if bottom is not None:
            navigation = next((item for item in regions if item["role"] == "status/action bar"), None)
            if navigation:
                navigation["role"] = "bottom navigation/action area"
        return "mobile-single-column", "Single-column mobile hierarchy with top context and a vertically scrolling primary surface.", regions, "medium"

    start = 0.0
    end = 1.0
    if leading is not None:
        regions.append(region("navigation/list sidebar", "leading", (0.0, content_top, leading, height), "semantic cue plus measured or conventional boundary"))
        start = leading
    if trailing is not None and trailing > start + 0.18:
        regions.append(region("detail/inspector", "trailing", (trailing, content_top, 1.0 - trailing, height), "semantic cue plus measured or conventional boundary"))
        end = trailing
    primary_role = "data table/list" if hints["table"] else "primary canvas/content"
    regions.append(region(primary_role, "center", (start, content_top, end - start, height), "dominant remaining region"))

    if leading is not None and trailing is not None:
        return "sidebar-content-inspector", "Leading navigation, central work area, and trailing detail inspector.", regions, "high" if hints["leading"] and hints["trailing"] else "medium"
    if leading is not None:
        return "sidebar-content", "Leading navigation or collection list beside a dominant content area.", regions, "high" if hints["leading"] else "medium"
    if trailing is not None:
        return "content-inspector", "Dominant content area with a trailing detail or property inspector.", regions, "high" if hints["trailing"] else "medium"
    if hints["table"]:
        return "table-or-list", "Single dominant table or list with controls around its perimeter.", regions, "medium"
    if hints["canvas"]:
        return "canvas", "Single dominant canvas or editor with peripheral controls.", regions, "medium"
    return "single-surface", "One dominant content surface without a stable secondary panel detected.", regions, "low"


def analyze(example: dict, catalog: str, image_path: Path) -> dict:
    payload = image_path.read_bytes()
    image = Image.open(image_path).convert("RGB")
    working = image.copy()
    working.thumbnail((900, 900), Image.Resampling.LANCZOS)
    array = np.asarray(working, dtype=np.float32)
    gray = array.mean(axis=2)
    vertical = separator_positions(gray, "vertical")
    horizontal = separator_positions(gray, "horizontal")
    layout, summary, regions, confidence = classify_layout(example, catalog, vertical, horizontal)
    gradients = np.abs(np.diff(gray, axis=1))
    edge_density = float((gradients > 24).mean()) if gradients.size else 0.0
    density = "high" if edge_density >= 0.14 else "medium" if edge_density >= 0.075 else "low"
    return {
        "analysis_kind": "deterministic-image-layout-v1",
        "image_sha256": hashlib.sha256(payload).hexdigest(),
        "orientation": "landscape" if image.width > image.height * 1.1 else "portrait" if image.height > image.width * 1.1 else "square",
        "layout_model": layout,
        "panel_summary": summary,
        "regions": regions,
        "detected_separators": {"vertical": vertical, "horizontal": horizontal},
        "visual_density": density,
        "confidence": confidence,
    }


def analyze_catalog(slug: str) -> tuple[int, list[dict]]:
    source_path = ROOT / slug / "sources.json"
    catalog = json.loads(source_path.read_text())
    failures: list[dict] = []
    count = 0
    for index, example in enumerate(catalog["examples"], 1):
        visual = example.get("visual")
        if not visual or not visual.get("local_path"):
            failures.append({"index": index, "name": example["name"], "error": "visual missing"})
            continue
        image_path = source_path.parent / visual["local_path"]
        try:
            example["interface_structure"] = analyze(example, slug, image_path)
            count += 1
        except Exception as error:
            failures.append({"index": index, "name": example["name"], "error": str(error)})
    if not failures and count == len(catalog["examples"]):
        catalog["schema"] = "wisent.example-catalog.v2"
        catalog["visual_count"] = count
        catalog["structure_count"] = count
    source_path.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n")
    failure_path = source_path.parent / "structure-analysis-failures.json"
    if failures:
        failure_path.write_text(json.dumps(failures, indent=2, ensure_ascii=False) + "\n")
    else:
        failure_path.unlink(missing_ok=True)
    print(f"{slug}: analyzed={count} failures={len(failures)}")
    return count, failures


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("catalogs", nargs="*")
    args = parser.parse_args()
    unknown = sorted(set(args.catalogs) - set(CATALOGS))
    if unknown:
        parser.error(f"unknown catalog(s): {', '.join(unknown)}")
    selected = args.catalogs or list(CATALOGS)
    failures = 0
    for slug in selected:
        _count, catalog_failures = analyze_catalog(slug)
        failures += len(catalog_failures)
    if failures:
        raise SystemExit(f"structure analysis left {failures} unresolved entries")


if __name__ == "__main__":
    main()
