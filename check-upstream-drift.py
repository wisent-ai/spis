#!/usr/bin/env python3
"""Report where the stored reference evidence no longer matches its upstream.

Every snapshot in this repository is dated and hashed: README blobs carry
`readme_blob_sha`, catalog entries carry `source_url` and `source_image_url`,
reference records carry a `source_url` per motion asset. Nothing re-checked any of
it, so a source that moved, went private, or changed content stayed invisible.

This utility performs three read-only checks and writes one report:

1. **README drift** — the current blob SHA of each snapshotted README, through the
   GitHub API (`gh`), against the SHA recorded at capture time.
2. **Source reachability** — an HTTP HEAD (falling back to a ranged GET) for every
   catalog `source_url`, `source_image_url`, and motion `source_url`.
3. **Local integrity** — every recorded local media path resolves and matches its
   recorded SHA-256.

    ./check-upstream-drift.py                     # everything, report to stdout
    ./check-upstream-drift.py --skip-network      # local integrity only
    ./check-upstream-drift.py --write-report      # also writes upstream-drift.json
    ./check-upstream-drift.py --strict            # exit 1 when drift is found
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
REPORT = ROOT / "upstream-drift.json"
USER_AGENT = "WisentProductGuidelines/1.0 (+https://wisent.ai)"
TIMEOUT = 20
SCHEMA = "wisent.upstream-drift-report.v1"


@dataclass
class Drift:
    readme_changed: list[dict[str, Any]] = field(default_factory=list)
    readme_unreachable: list[dict[str, Any]] = field(default_factory=list)
    readme_unchanged: int = 0
    sources_gone: list[dict[str, Any]] = field(default_factory=list)
    sources_guarded: list[dict[str, Any]] = field(default_factory=list)
    sources_unresolved: list[dict[str, Any]] = field(default_factory=list)
    sources_ok: int = 0
    sources_skipped: int = 0
    media_missing: list[str] = field(default_factory=list)
    media_hash_mismatch: list[str] = field(default_factory=list)
    media_ok: int = 0

    def any_drift(self) -> bool:
        """Guarded sources are not drift: an authenticated product answering 401 is
        behaving as recorded. A gone or unresolvable source is drift."""
        return bool(
            self.readme_changed
            or self.readme_unreachable
            or self.sources_gone
            or self.sources_unresolved
            or self.media_missing
            or self.media_hash_mismatch
        )


def gh_json(path: str) -> dict[str, Any] | None:
    proc = subprocess.run(
        ["gh", "api", path, "--cache", "0"],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def check_readmes(drift: Drift) -> None:
    sources = json.loads((ROOT / "readme-examples" / "sources.json").read_text())
    entries = sources.get("repositories") or sources.get("examples") or sources.get("sources") or []

    def one(entry: dict[str, Any]) -> None:
        repo = entry.get("repository")
        readme_path = entry.get("readme_path")
        recorded = entry.get("readme_blob_sha")
        if not (repo and readme_path):
            return
        data = gh_json(f"repos/{repo}/contents/{readme_path}")
        if data is None or "sha" not in data:
            drift.readme_unreachable.append({"repository": repo, "readme_path": readme_path})
            return
        if data["sha"] != recorded:
            drift.readme_changed.append(
                {
                    "repository": repo,
                    "readme_path": readme_path,
                    "recorded_sha": recorded,
                    "current_sha": data["sha"],
                    "snapshot": entry.get("filename"),
                }
            )
        else:
            drift.readme_unchanged += 1

    with ThreadPoolExecutor(max_workers=8) as pool:
        list(pool.map(one, entries))


# A dead reference and a guarded one are different findings. An authenticated web
# app answering 401 at its dashboard URL is behaving exactly as recorded; a 404 means
# the page we cited is gone and the provenance is now wrong.
GONE_CODES = {400, 404, 410}
GUARDED_CODES = {401, 403, 405, 429, 451, 501, 503}


def url_state(url: str) -> tuple[str, int | str]:
    """Return (state, detail) where state is reachable | gone | guarded | unresolved."""
    request = urllib.request.Request(url, method="HEAD", headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
            return "reachable", response.status
    except urllib.error.HTTPError as exc:
        if exc.code in GUARDED_CODES:  # HEAD refused or rate limited; try one byte
            try:
                ranged = urllib.request.Request(
                    url, headers={"User-Agent": USER_AGENT, "Range": "bytes=0-0"}
                )
                with urllib.request.urlopen(ranged, timeout=TIMEOUT) as response:
                    return "reachable", response.status
            except urllib.error.HTTPError as inner:
                if inner.code in GONE_CODES:
                    return "gone", inner.code
                return "guarded", inner.code
            except Exception as inner:  # noqa: BLE001 - reported, not raised
                return "guarded", f"{exc.code} then {type(inner).__name__}"
        if exc.code in GONE_CODES:
            return "gone", exc.code
        return "guarded", exc.code
    except urllib.error.URLError as exc:
        return "unresolved", type(exc.reason).__name__ if exc.reason else "URLError"
    except Exception as exc:  # noqa: BLE001 - network failure is the finding
        return "unresolved", type(exc).__name__


def collect_urls() -> list[tuple[str, str, str | None]]:
    """(url, where, expected_state) for every recorded upstream reference."""
    pairs: list[tuple[str, str, str | None]] = []
    for sources in sorted(ROOT.glob("*-examples/sources.json")):
        data = json.loads(sources.read_text())
        for example in data.get("examples", []) or []:
            name = example.get("name", "?")
            if example.get("source_url"):
                pairs.append((
                    example["source_url"],
                    f"{sources.parent.name}/{name}/source_url",
                    example.get("source_url_state"),
                ))
            visual = example.get("visual") or {}
            for key in ("source_page_url", "source_image_url"):
                if visual.get(key):
                    pairs.append((
                        visual[key],
                        f"{sources.parent.name}/{name}/{key}",
                        visual.get(f"{key}_state"),
                    ))
    for record in sorted(ROOT.glob("*-examples/references/*/reference.json")):
        data = json.loads(record.read_text())
        for entry in data.get("motion", []) or []:
            if entry.get("source_url"):
                pairs.append((
                    entry["source_url"],
                    f"{record.parent.parent.parent.name}/{record.parent.name}/motion",
                    entry.get("source_url_state"),
                ))
    seen: set[str] = set()
    unique: list[tuple[str, str, str | None]] = []
    for url, where, expected in pairs:
        if url in seen:
            continue
        seen.add(url)
        unique.append((url, where, expected))
    return unique


def check_sources(drift: Drift) -> None:
    buckets = {
        "gone": drift.sources_gone,
        "guarded": drift.sources_guarded,
        "unresolved": drift.sources_unresolved,
    }

    def one(pair: tuple[str, str, str | None]) -> None:
        url, where, expected = pair
        if not url.startswith(("http://", "https://")):
            drift.sources_skipped += 1
            return
        state, detail = url_state(url)
        if state == "reachable":
            drift.sources_ok += 1
            return
        # A private repository or authenticated application may deliberately
        # answer anonymous HTTP with 400/404. Its recorded classification wins
        # over the transport code; otherwise the checker reports a live guarded
        # surface as deleted on every run.
        if expected == "guarded" and state == "gone":
            state = "guarded"
        buckets[state].append({"url": url, "where": where, "result": detail, "expected": expected})

    with ThreadPoolExecutor(max_workers=12) as pool:
        list(pool.map(one, collect_urls()))


def check_local_media(drift: Drift) -> None:
    for record in sorted(ROOT.glob("*-examples/references/*/reference.json")):
        data = json.loads(record.read_text())
        base = record.parent
        for key in ("motion", "states"):
            for entry in data.get(key, []) or []:
                local = base / str(entry.get("local_path", ""))
                rel = str(local.relative_to(ROOT))
                if not local.exists():
                    drift.media_missing.append(rel)
                    continue
                recorded = entry.get("sha256")
                if not recorded:
                    continue
                digest = hashlib.sha256()
                with local.open("rb") as fh:
                    for chunk in iter(lambda: fh.read(1 << 20), b""):
                        digest.update(chunk)
                if digest.hexdigest() != recorded:
                    drift.media_hash_mismatch.append(rel)
                else:
                    drift.media_ok += 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-network", action="store_true")
    parser.add_argument("--skip-readme", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    drift = Drift()
    check_local_media(drift)
    if not args.skip_network:
        if not args.skip_readme:
            check_readmes(drift)
        check_sources(drift)

    print(f"local media verified: {drift.media_ok}")
    print(f"local media missing: {len(drift.media_missing)}")
    print(f"local media hash mismatch: {len(drift.media_hash_mismatch)}")
    if not args.skip_network:
        print(f"README snapshots unchanged: {drift.readme_unchanged}")
        print(f"README snapshots changed upstream: {len(drift.readme_changed)}")
        print(f"README snapshots unreachable: {len(drift.readme_unreachable)}")
        print(f"upstream URLs reachable: {drift.sources_ok}")
        print(f"upstream URLs gone: {len(drift.sources_gone)}")
        print(f"upstream URLs guarded (auth, rate limit, bot wall): {len(drift.sources_guarded)}")
        print(f"upstream URLs unresolved (network): {len(drift.sources_unresolved)}")
    for item in drift.media_missing[:20]:
        print(f"  missing media: {item}")
    for item in drift.media_hash_mismatch[:20]:
        print(f"  hash mismatch: {item}")
    for item in drift.readme_changed[:20]:
        print(f"  README changed: {item['repository']} ({item['snapshot']})")
    for item in drift.sources_gone:
        print(f"  upstream gone: {item['where']} -> {item['result']} {item['url']}")
    for item in drift.sources_unresolved:
        print(f"  upstream unresolved: {item['where']} -> {item['result']} {item['url']}")

    if args.write_report:
        REPORT.write_text(
            json.dumps(
                {
                    "schema": SCHEMA,
                    "checked_at": subprocess.run(
                        ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], capture_output=True, text=True
                    ).stdout.strip(),
                    "network_checked": not args.skip_network,
                    "local_media_verified": drift.media_ok,
                    "local_media_missing": drift.media_missing,
                    "local_media_hash_mismatch": drift.media_hash_mismatch,
                    "readme_unchanged": drift.readme_unchanged,
                    "readme_changed": drift.readme_changed,
                    "readme_unreachable": drift.readme_unreachable,
                    "upstream_urls_reachable": drift.sources_ok,
                    "upstream_urls_gone": drift.sources_gone,
                    "upstream_urls_guarded": drift.sources_guarded,
                    "upstream_urls_unresolved": drift.sources_unresolved,
                    "upstream_urls_skipped": drift.sources_skipped,
                },
                indent=2,
            )
            + "\n"
        )
        print(f"\nreport written to {REPORT.name}")

    return 1 if args.strict and drift.any_drift() else 0


if __name__ == "__main__":
    sys.exit(main())
