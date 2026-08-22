#!/usr/bin/env python3
"""Collect one attributable official interface image for each catalog entry.

The collector performs static HTTP reads only. It prefers large images whose URL,
alt text, or surrounding metadata identifies a screenshot or product interface,
then stores a bounded WebP derivative while retaining the original image URL.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import io
import json
import math
import re
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path

from PIL import Image, ImageFile

ImageFile.LOAD_TRUNCATED_IMAGES = True

ROOT = Path(__file__).resolve().parent
USER_AGENT = "WisentProductGuidelines/1.0 (+https://wisent.ai)"
MAX_PAGE_BYTES = 8 * 1024 * 1024
MAX_IMAGE_BYTES = 16 * 1024 * 1024
MAX_CANDIDATES = 14
TARGET_SIZE = (1400, 1000)
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
)


@dataclass(frozen=True)
class Candidate:
    url: str
    hint: str
    order: int
    origin: str


class ImageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.candidates: list[Candidate] = []
        self._order = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = {key.lower(): value or "" for key, value in attrs}
        if tag.lower() == "meta":
            key = (values.get("property") or values.get("name") or "").lower()
            if key in {"og:image", "og:image:secure_url", "twitter:image", "twitter:image:src"}:
                self._add(values.get("content", ""), key, "meta")
        if tag.lower() in {"img", "source"}:
            hint = " ".join((values.get("alt", ""), values.get("title", ""), values.get("class", "")))
            for field in ("src", "data-src", "data-lazy-src", "data-original"):
                self._add(values.get(field, ""), hint, tag.lower())
            for field in ("srcset", "data-srcset"):
                for item in values.get(field, "").split(","):
                    self._add(item.strip().split(" ", 1)[0], hint, f"{tag.lower()}-srcset")
        if tag.lower() == "link" and "image_src" in values.get("rel", "").lower():
            self._add(values.get("href", ""), values.get("title", ""), "link")

    def _add(self, url: str, hint: str, origin: str) -> None:
        if not url:
            return
        self.candidates.append(Candidate(html.unescape(url), hint.strip(), self._order, origin))
        self._order += 1


def fetch(url: str, maximum: int, *, accept: str) -> tuple[bytes, str, str]:
    request = urllib.request.Request(
        url,
        headers={"User-Agent": USER_AGENT, "Accept": accept},
    )
    with urllib.request.urlopen(request, timeout=15) as response:
        content_type = response.headers.get_content_type()
        final_url = response.geturl()
        data = response.read(maximum + 1)
    if len(data) > maximum:
        raise ValueError(f"response exceeds {maximum} bytes")
    return data, content_type, final_url


def candidate_urls(page_url: str, body: bytes, content_type: str) -> list[Candidate]:
    if content_type.startswith("image/"):
        return [Candidate(page_url, "direct image", 0, "direct")]
    text = body.decode("utf-8", errors="replace")
    text = text.replace("\\/", "/").replace("\\u0026", "&").replace("\\u003d", "=")
    parser = ImageParser()
    parser.feed(text)
    for match in re.finditer(r"url\((?:&quot;|['\"])?(https?://[^)'\"\s]+)", text, flags=re.I):
        parser._add(match.group(1), "css background", "css")
    for match in re.finditer(r"https?://[^\s'\"<>]+\.(?:png|jpe?g|webp)(?:\?[^\s'\"<>]*)?", text, flags=re.I):
        parser._add(match.group(0), "embedded image URL", "embedded")

    unique: dict[str, Candidate] = {}
    for candidate in parser.candidates:
        resolved = urllib.parse.urljoin(page_url, candidate.url)
        parsed = urllib.parse.urlparse(resolved)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            continue
        clean = urllib.parse.quote(resolved.replace("&amp;", "&"), safe=":/?&=#%+@,;[]!$'()*")
        unique.setdefault(clean, Candidate(clean, candidate.hint, candidate.order, candidate.origin))
    return sorted(unique.values(), key=preflight_score, reverse=True)[:MAX_CANDIDATES]


def preflight_score(candidate: Candidate) -> float:
    text = f"{candidate.url} {candidate.hint}".lower()
    score = 0.0
    for word, weight in (
        ("screenshot", 35), ("screen", 18), ("interface", 24), ("dashboard", 22),
        ("window", 16), ("workflow", 14), ("product", 8), ("hero", 5), ("app", 4),
    ):
        if word in text:
            score += weight
    for word, weight in (
        ("logo", -45), ("icon", -38), ("avatar", -40), ("badge", -50),
        ("favicon", -60), ("opengraph", -22), ("emoji", -45), ("spinner", -45),
    ):
        if word in text:
            score += weight
    if candidate.origin == "meta":
        score += 9
    if candidate.origin.endswith("srcset"):
        score += 7
    score -= candidate.order * 0.015
    return score


def decode_candidate(candidate: Candidate) -> tuple[Image.Image, bytes, str] | None:
    try:
        data, content_type, final_url = fetch(candidate.url, MAX_IMAGE_BYTES, accept="image/avif,image/webp,image/png,image/jpeg,image/*")
        image = Image.open(io.BytesIO(data))
        image.load()
        if min(image.width, image.height) < 260 or max(image.width, image.height) < 480:
            return None
        if image.width / image.height > 5.2 or image.height / image.width > 3.2:
            return None
        return image, data, final_url
    except (OSError, ValueError, urllib.error.URLError):
        return None


def image_score(candidate: Candidate, image: Image.Image) -> float:
    area = image.width * image.height
    score = preflight_score(candidate) + math.log2(max(area, 1)) * 3
    ratio = image.width / image.height
    if 1.15 <= ratio <= 2.4:
        score += 14
    elif 0.45 <= ratio < 1.15:
        score += 8
    if image.width >= 1000:
        score += 8
    if image.height >= 600:
        score += 7
    if image.width == image.height:
        score -= 18
    return score


def select_image(page_url: str) -> tuple[Candidate, Image.Image, str]:
    body, content_type, final_page_url = fetch(page_url, MAX_PAGE_BYTES, accept="text/html,application/xhtml+xml,image/*")
    candidates = candidate_urls(final_page_url, body, content_type)
    decoded: list[tuple[float, Candidate, Image.Image, str]] = []
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = {executor.submit(decode_candidate, candidate): candidate for candidate in candidates}
        for future in as_completed(futures):
            result = future.result()
            if result is None:
                continue
            image, _data, final_url = result
            candidate = futures[future]
            decoded.append((image_score(candidate, image), candidate, image, final_url))
    if not decoded:
        raise ValueError("no qualifying image found")
    _score, candidate, image, final_url = max(decoded, key=lambda item: item[0])
    return candidate, image, final_url


def stable_provenance_url(value: str) -> str:
    parsed = urllib.parse.urlsplit(value)
    if parsed.hostname in {
        "private-user-images.githubusercontent.com",
        "github-production-user-asset-6210df.s3.amazonaws.com",
    }:
        return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, parsed.path, "", ""))
    return value


def store_image(catalog_dir: Path, index: int, name: str, page_url: str) -> dict:
    candidate, image, final_image_url = select_image(page_url)
    if image.mode not in {"RGB", "RGBA"}:
        image = image.convert("RGBA")
    if image.mode == "RGBA":
        background = Image.new("RGB", image.size, "white")
        background.paste(image, mask=image.getchannel("A"))
        image = background
    else:
        image = image.convert("RGB")
    original_size = {"width": image.width, "height": image.height}
    image.thumbnail(TARGET_SIZE, Image.Resampling.LANCZOS)
    output = io.BytesIO()
    image.save(output, format="WEBP", quality=82, method=6)
    payload = output.getvalue()
    slug = re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")[:60]
    relative_path = Path("images") / f"{index:02d}-{slug}.webp"
    destination = catalog_dir / relative_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(payload)
    return {
        "source_page_url": page_url,
        "source_image_url": stable_provenance_url(final_image_url),
        "local_path": relative_path.as_posix(),
        "capture_kind": "official-source-image",
        "captured_at": time.strftime("%Y-%m-%d", time.gmtime()),
        "format": "webp",
        "width": image.width,
        "height": image.height,
        "original_width": original_size["width"],
        "original_height": original_size["height"],
        "bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "source_hint": candidate.hint or candidate.origin,
    }


def collect_catalog(slug: str, *, replace: bool) -> tuple[int, list[dict]]:
    source_path = ROOT / slug / "sources.json"
    catalog = json.loads(source_path.read_text())
    failures: list[dict] = []
    collected = 0
    for index, example in enumerate(catalog["examples"], 1):
        if example.get("visual") and not replace:
            example["visual"]["source_page_url"] = example["source_url"]
            source_image_url = example["visual"].get("source_image_url")
            if source_image_url:
                example["visual"]["source_image_url"] = stable_provenance_url(source_image_url)
                source_path.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n")
            continue
        try:
            capture_url = example.get("visual_source_url") or example["source_url"]
            capture_kind = "official-source-image"
            try:
                visual = store_image(source_path.parent, index, example["name"], capture_url)
            except Exception:
                capture_url = (
                    "https://image.thum.io/get/width/1400/crop/1000/noanimate/"
                    f"{example['source_url']}"
                )
                visual = store_image(source_path.parent, index, example["name"], capture_url)
                capture_kind = "remote-page-screenshot"
            visual["source_page_url"] = example["source_url"]
            visual["capture_kind"] = capture_kind
            example["visual"] = visual
            collected += 1
            print(f"{slug} {index:02d}/50 image {example['name']}", flush=True)
        except Exception as error:
            failures.append({"index": index, "name": example["name"], "url": example.get("visual_source_url") or example["source_url"], "error": str(error)})
            print(f"{slug} {index:02d}/50 FAILED {example['name']}: {error}", flush=True)
        source_path.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n")
    failure_path = source_path.parent / "image-collection-failures.json"
    if failures:
        failure_path.write_text(json.dumps(failures, indent=2, ensure_ascii=False) + "\n")
    else:
        failure_path.unlink(missing_ok=True)
    return collected, failures


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("catalogs", nargs="*")
    parser.add_argument("--replace", action="store_true")
    args = parser.parse_args()
    unknown = sorted(set(args.catalogs) - set(CATALOGS))
    if unknown:
        parser.error(f"unknown catalog(s): {', '.join(unknown)}")
    selected = args.catalogs or list(CATALOGS)
    total_failures = 0
    for slug in selected:
        collected, failures = collect_catalog(slug, replace=args.replace)
        total_failures += len(failures)
        print(f"{slug}: collected={collected} failures={len(failures)}", flush=True)
    if total_failures:
        raise SystemExit(f"image collection left {total_failures} unresolved entries")


if __name__ == "__main__":
    main()
