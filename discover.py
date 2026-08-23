#!/usr/bin/env python3
"""Discover the important pages behind a start URL and turn them into records.

Pipeline, fully automatic:
1. Fetch the start page and extract every same-origin link with its text.
2. Ask Brama which pages matter for a reference corpus (pricing, docs,
   sign-in, about…). If Brama is unreachable or unauthenticated, fall back to
   deterministic keyword classification — discovery never blocks on a model.
3. Download an overview screenshot per selected page and scaffold a numbered
   record through the same contract as `reference add` (partial, gaps named).

The model only proposes; validation rejects any URL outside the discovered
set, so a hallucinated link can never become a record.

Usage:
  discover.py <start-url> --catalog <slug> [--limit <n>] [--depth-links <n>]
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parent
UA = {"User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) spis-discovery/1.0"}
THUMB = "https://image.thum.io/get/width/1400/crop/1000/noanimate/"
FAMILIES = ["pricing", "docs", "signup", "about", "product", "blog", "other"]


def fail(message: str):
    print(f"discover: {message}", file=sys.stderr)
    raise SystemExit(1)


def fetch(url: str, timeout: int = 25) -> tuple[bytes, str]:
    request = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return response.read(), response.headers.get_content_type()


class LinkParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.links: dict[str, str] = {}
        self._href = None
        self._text: list[str] = []

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            self._href = dict(attrs).get("href")
            self._text = []

    def handle_data(self, data):
        if self._href is not None:
            self._text.append(data)

    def handle_endtag(self, tag):
        if tag == "a" and self._href:
            text = re.sub(r"\s+", " ", " ".join(self._text)).strip()
            self.links.setdefault(self._href, text)
            self._href = None


def same_origin(url: str, origin: str) -> bool:
    a, b = urllib.parse.urlparse(url), urllib.parse.urlparse(origin)
    return a.scheme in {"http", "https"} and a.netloc == b.netloc


def extract_links(start_url: str, html_bytes: bytes, limit: int) -> dict[str, str]:
    parser = LinkParser()
    parser.feed(html_bytes.decode("utf-8", errors="replace"))
    origin = start_url
    found: dict[str, str] = {}
    for href, text in parser.links.items():
        absolute = urllib.parse.urljoin(start_url, href.split("#")[0])
        if not same_origin(absolute, origin) or absolute.rstrip("/") == origin.rstrip("/"):
            continue
        if re.search(r"\.(pdf|zip|png|jpe?g|svg|webp|gif|mp4|css|js)$", absolute, re.I):
            continue
        found.setdefault(absolute, text or absolute)
        if len(found) >= limit:
            break
    return found


KEYWORDS = {
    "pricing": ["pricing", "plans", "plans-and-pricing"],
    "docs": ["docs", "documentation", "developers", "api", "guides"],
    "signup": ["sign-up", "signup", "register", "get-started", "start"],
    "about": ["about", "company", "customers", "careers"],
    "product": ["product", "features", "platform", "solutions"],
}


def heuristic_family(url: str, text: str) -> str:
    blob = f"{url} {text}".lower()
    for family, words in KEYWORDS.items():
        if any(word in blob for word in words):
            return family
    return "other"


def brama_rank(start_url: str, links: dict[str, str], limit: int) -> dict[str, str] | None:
    router = os.environ.get("MODEL_ROUTER_URL")
    if not router:
        return None
    endpoint = router.rstrip("/") + ("/chat/completions" if "/v1" in router else "/v1/chat/completions")
    listing = "\n".join(f"- {url} | {text}" for url, text in list(links.items())[:80])
    payload = json.dumps({
        "model": os.environ.get("MODEL_ROUTER_MODEL", "gpt-4o-mini"),
        "messages": [
            {"role": "system", "content":
                "You classify pages of one product's website for an interface reference corpus. "
                "Return STRICT JSON: {\"pages\": [{\"url\": string, \"family\": "
                f"one of {FAMILIES}]}} . Only use URLs from the list. Pick at most {limit}."},
            {"role": "user", "content": f"Start page: {start_url}\nDiscovered links:\n{listing}"},
        ],
    }).encode()
    request = urllib.request.Request(
        endpoint, data=payload,
        headers={"Content-Type": "application/json",
                 **({"Authorization": "Bearer " + os.environ["MODEL_ROUTER_TOKEN"]}
                    if os.environ.get("MODEL_ROUTER_TOKEN") else {})},
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            body = json.loads(response.read())
        content = body["choices"][0]["message"]["content"]
        parsed = json.loads(re.search(r"\{.*\}", content, re.S).group(0))
        result = {}
        for page in parsed.get("pages", []):
            url, family = page.get("url"), page.get("family")
            if url in links and family in FAMILIES:
                result[url] = family
        return result or None
    except Exception as error:  # noqa: BLE001 — any model failure falls back deterministically
        print(f"discover: Brama ranking unavailable ({error}); using keyword fallback", file=sys.stderr)
        return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("start_url")
    parser.add_argument("--catalog", required=True)
    parser.add_argument("--limit", type=int, default=6)
    parser.add_argument("--max-links", type=int, default=120)
    args = parser.parse_args()

    slug = args.catalog if args.catalog.endswith("-examples") else args.catalog + "-examples"
    directory = ROOT / slug

    html_bytes, _ = fetch(args.start_url)
    links = extract_links(args.start_url, html_bytes, args.max_links)
    print(f"discovered {len(links)} same-origin links on {args.start_url}")
    if not links:
        fail("no same-origin links found")

    ranked = brama_rank(args.start_url, links, args.limit)
    if ranked is None:
        ranked = {}
        for url, text in links.items():
            family = heuristic_family(url, text)
            if family != "other" and len([u for u, f in ranked.items() if f == family]) < max(1, args.limit // 3):
                ranked[url] = family
    selected = list(ranked.items())[: args.limit]
    if not selected:
        fail("nothing selected for this corpus")
    print(f"Brama/heuristics selected {len(selected)} page(s):")
    for url, family in selected:
        print(f"  [{family}] {url}")

    # ensure the catalog exists, then reuse the tested record scaffolder
    if not directory.is_dir():
        subprocess.run([
            sys.executable, str(ROOT / "catalog-type.py"), "add", args.catalog,
            "--title", f"{args.catalog.replace('-examples', '').capitalize()} examples",
        ], check=True)

    for url, family in selected:
        thumb_url = THUMB + url
        image_bytes, _ = fetch(thumb_url, timeout=40)
        tmp = Path("/tmp") / (re.sub(r"[^a-z0-9]+", "-", url.lower()).strip("-")[:60] + ".png")
        tmp.write_bytes(image_bytes)
        result = subprocess.run([
            sys.executable, str(ROOT / "reference-record.py"), "add", slug,
            "--name", family.capitalize() + " — " + urllib.parse.urlparse(url).path.strip("/").split("/")[-1][:40],
            "--source-url", url,
            "--category", family,
            "--selection-note", f"auto-discovered from {args.start_url}; family {family}",
            "--visual", str(tmp),
            "--owner", urllib.parse.urlparse(url).netloc,
        ], capture_output=True, text=True)
        print(result.stdout.strip() or result.stderr.strip())

    regenerate = subprocess.run([sys.executable, str(ROOT / "generate-example-catalogs.py")], capture_output=True, text=True)
    if regenerate.returncode != 0:
        fail(f"index regeneration refused:\n{regenerate.stdout}{regenerate.stderr}")
    print("done: catalog regenerated and consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
