#!/usr/bin/env python3
"""Measure recurring structural patterns in the curated README snapshots."""

from __future__ import annotations

import json
import re
import statistics
from itertools import pairwise
from pathlib import Path

ROOT = Path(__file__).resolve().parent
EXAMPLES = ROOT / "readme-examples"
OUTPUT = EXAMPLES / "analysis.json"

SECTION_PATTERNS = {
    "installation_or_quick_start": r"(?im)^.{0,8}(install|installation|setup|getting started|quick ?start|get started)",
    "usage_or_examples": r"(?im)^.{0,8}(usage|how to use|examples?|tutorial)",
    "features_or_capabilities": r"(?im)^.{0,8}(features?|capabilities|what .* does)",
    "documentation_links": r"(?im)^.{0,8}(documentation|docs|learn more)",
    "contribution_guidance": r"(?im)^.{0,8}(contribut|development)",
    "license_section": r"(?im)^.{0,8}(licen[cs]e)",
    "security_guidance": r"(?im)^.{0,8}(security|vulnerabilit)",
    "support_or_community": r"(?im)^.{0,8}(support|help|community|getting help)",
    "architecture_or_how_it_works": r"(?im)^.{0,8}(architecture|how it works|design|internals)",
    "status_or_roadmap": r"(?im)^.{0,8}(status|roadmap|maturity|stability)",
    "requirements_or_prerequisites": r"(?im)^.{0,8}(requirements?|prerequisites?|compatibility)",
    "alternatives_or_comparison": r"(?im)^.{0,8}(alternatives?|comparison|why )",
}

BADGE_PATTERN = re.compile(
    r"shields\.io|badge\.svg|actions/workflows|badge\.fury|badgen\.net|/badge/",
    re.IGNORECASE,
)
VISUAL_PATTERN = re.compile(
    r"<img\b[^>]*>|!\[[^]]*\]\([^)]+\)|\.\.\s+image::\s*\S+",
    re.IGNORECASE,
)
TABLE_PATTERN = re.compile(
    r"(?m)^\s*\|?(?:\s*:?-{3,}:?\s*\|)+\s*:?-{3,}:?\s*\|?\s*$"
)
LENGTH_BANDS = {
    "compact_75_lines_or_fewer": (0, 75),
    "standard_76_to_200_lines": (76, 200),
    "extended_201_to_400_lines": (201, 400),
    "manual_over_400_lines": (401, float("inf")),
}


def headings(text: str) -> list[str]:
    markdown = [
        match.group("title").strip()
        for match in re.finditer(r"(?m)^#{1,6}\s+(?P<title>.+?)\s*$", text)
    ]
    rst_lines = text.splitlines()
    rst = []
    for title, underline in pairwise(rst_lines):
        if title.strip() and re.fullmatch(r"[=\-~^`:#*+]{3,}", underline.strip()):
            rst.append(title.strip())
    return markdown + rst


def percentile(values: list[int], fraction: float) -> float:
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    return round(ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower), 2)


def inspect(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    found_headings = headings(text)
    has_visual = bool(VISUAL_PATTERN.search(text))
    return {
        "file": path.name,
        "lines": len(lines),
        "words": len(re.findall(r"\b\w+\b", text)),
        "headings": len(found_headings),
        "first_heading": next(iter(found_headings), None),
        "badges": bool(BADGE_PATTERN.search(text)),
        "visuals": has_visual,
        "visuals_first_30_lines": bool(VISUAL_PATTERN.search("\n".join(lines[:30]))),
        "animated_gif": bool(re.search(r"(?i)\.gif(?:[?#)\"'\s]|$)", text)),
        "video": bool(re.search(r"(?i)<video\b", text)),
        "code_examples": bool(re.search(r"```|\.\.\s+(code-block|sourcecode)::", text, re.IGNORECASE)),
        "code_block_count": len(re.findall(r"(?m)^```", text)) // 2,
        "mermaid": bool(re.search(r"(?im)^```mermaid", text)),
        "markdown_table": bool(TABLE_PATTERN.search(text)),
        "table_of_contents": bool(re.search(r"(?im)^.{0,8}(table of contents|contents)\s*$", text)),
        **{
            name: bool(re.search(pattern, text))
            for name, pattern in SECTION_PATTERNS.items()
        },
    }


def main() -> None:
    paths = sorted(EXAMPLES.glob("[0-9][0-9]-*"))
    records = [inspect(path) for path in paths if path.is_file()]
    total = len(records)
    boolean_fields = [
        "badges",
        "visuals",
        "visuals_first_30_lines",
        "animated_gif",
        "video",
        "code_examples",
        "mermaid",
        "markdown_table",
        "table_of_contents",
        *SECTION_PATTERNS,
    ]
    prevalence = {}
    for field in boolean_fields:
        count = sum(record[field] for record in records)
        prevalence[field] = {
            "count": count,
            "share": f"{count / total:.0%}",
        }

    line_values = [record["lines"] for record in records]
    word_values = [record["words"] for record in records]
    result = {
        "schema": "wisent.readme-example-analysis",
        "source_count": total,
        "length": {
            "median_lines": statistics.median(line_values),
            "p25_lines": percentile(line_values, 0.25),
            "p75_lines": percentile(line_values, 0.75),
            "p90_lines": percentile(line_values, 0.90),
            "median_words": statistics.median(word_values),
            "p25_words": percentile(word_values, 0.25),
            "p75_words": percentile(word_values, 0.75),
            "p90_words": percentile(word_values, 0.90),
            "shortest_lines": min(line_values),
            "longest_lines": max(line_values),
            "median_headings": statistics.median(record["headings"] for record in records),
            "median_code_blocks": statistics.median(record["code_block_count"] for record in records),
            "bands": {
                name: {
                    "count": sum(lower <= value <= upper for value in line_values),
                    "share": f"{sum(lower <= value <= upper for value in line_values) / total:.0%}",
                }
                for name, (lower, upper) in LENGTH_BANDS.items()
            },
        },
        "prevalence": prevalence,
        "files": records,
    }
    OUTPUT.write_text(
        json.dumps(result, ensure_ascii=False, indent=len("  ")) + "\n",
        encoding="utf-8",
    )
    print(json.dumps({key: result[key] for key in ("source_count", "length", "prevalence")}, indent=len("  ")))


if __name__ == "__main__":
    main()
