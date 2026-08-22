#!/usr/bin/env python3
"""Refresh the curated README example snapshots from GitHub.

The script writes verbatim README snapshots plus source metadata. It requires an
authenticated GitHub CLI (`gh auth login`) and makes read-only GitHub API calls.
"""

from __future__ import annotations

import base64
import json
import shutil
import subprocess
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OUTPUT = ROOT / "readme-examples"

# Deliberately broad: product pages, libraries, CLIs, infrastructure, databases,
# developer tools, and AI systems expose different effective README patterns.
SOURCES = [
    ("sindresorhus/awesome", "curation"),
    ("facebook/react", "framework"),
    ("vuejs/core", "framework"),
    ("sveltejs/svelte", "framework"),
    ("vercel/next.js", "framework"),
    ("vitejs/vite", "tooling"),
    ("tailwindlabs/tailwindcss", "tooling"),
    ("shadcn-ui/ui", "design-system"),
    ("supabase/supabase", "platform"),
    ("n8n-io/n8n", "automation"),
    ("hoppscotch/hoppscotch", "developer-tool"),
    ("AppFlowy-IO/AppFlowy", "application"),
    ("rustdesk/rustdesk", "application"),
    ("calcom/cal.com", "application"),
    ("directus/directus", "platform"),
    ("strapi/strapi", "platform"),
    ("pocketbase/pocketbase", "backend"),
    ("immich-app/immich", "application"),
    ("home-assistant/core", "platform"),
    ("mattermost/mattermost", "application"),
    ("RocketChat/Rocket.Chat", "application"),
    ("grafana/grafana", "observability"),
    ("prometheus/prometheus", "observability"),
    ("kubernetes/kubernetes", "infrastructure"),
    ("hashicorp/terraform", "infrastructure"),
    ("ansible/ansible", "infrastructure"),
    ("docker/compose", "infrastructure"),
    ("localstack/localstack", "developer-tool"),
    ("minio/minio", "storage"),
    ("redis/redis", "database"),
    ("duckdb/duckdb", "database"),
    ("ClickHouse/ClickHouse", "database"),
    ("qdrant/qdrant", "database"),
    ("milvus-io/milvus", "database"),
    ("ollama/ollama", "ai-infrastructure"),
    ("ggml-org/llama.cpp", "ai-infrastructure"),
    ("vllm-project/vllm", "ai-infrastructure"),
    ("huggingface/transformers", "ai-library"),
    ("fastapi/fastapi", "library"),
    ("pydantic/pydantic", "library"),
    ("astral-sh/uv", "tooling"),
    ("astral-sh/ruff", "tooling"),
    ("pytest-dev/pytest", "tooling"),
    ("psf/requests", "library"),
    ("denoland/deno", "runtime"),
    ("oven-sh/bun", "runtime"),
    ("neovim/neovim", "developer-tool"),
    ("helix-editor/helix", "developer-tool"),
    ("BurntSushi/ripgrep", "cli"),
    ("sharkdp/bat", "cli"),
]


def github_token() -> str:
    gh = shutil.which("gh")
    if gh is None:
        raise RuntimeError("GitHub CLI is not installed")
    result = subprocess.run(
        [gh, "auth", "token"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def get_json(url: str, token: str) -> dict:
    request = urllib.request.Request(
        url,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "wisent-product-guidelines-readme-curation",
        },
    )
    with urllib.request.urlopen(request) as response:
        return json.load(response)


def fetch_source(index: int, requested_repo: str, category: str, token: str) -> dict:
    repo = get_json(f"https://api.github.com/repos/{requested_repo}", token)
    readme = get_json(f"https://api.github.com/repos/{requested_repo}/readme", token)
    canonical_repo = repo["full_name"]
    filename_repo = canonical_repo.lower().replace("/", "-").replace(".", "-")
    suffix = Path(readme["path"]).suffix.lower() or ".md"
    return {
        "number": index,
        "filename": f"{index:02d}-{filename_repo}{suffix}",
        "repository": canonical_repo,
        "category": category,
        "description": repo.get("description") or "",
        "repository_url": repo["html_url"],
        "default_branch": repo["default_branch"],
        "license_spdx": (repo.get("license") or {}).get("spdx_id") or "NOASSERTION",
        "stars_at_capture": repo.get("stargazers_count"),
        "readme_path": readme["path"],
        "readme_blob_sha": readme["sha"],
        "readme_url": readme["html_url"],
        "content": base64.b64decode(readme["content"]).decode("utf-8"),
    }


def render_index(entries: list[dict], captured_at: str) -> str:
    lines = [
        "# Open-source README examples",
        "",
        "Fifty verbatim README snapshots from established open-source repositories. This is a curated reference set, not a ranking. Use it to compare information architecture, product positioning, quick starts, trust signals, support boundaries, and contribution paths.",
        "",
        "The snapshots remain the work of their respective projects and are governed by each source repository's license. Review patterns; do not copy project names, artwork, badges, or claims. Relative images and links may only render correctly in the upstream repository.",
        "",
        f"Captured from GitHub on `{captured_at}`. `sources.json` records the README blob SHA, upstream URL, repository license identifier, and capture-time metadata for every file. Run `../sync-readme-examples.py` to refresh the catalog.",
        "Derived guidance: [README Best Practices](../readme-best-practices.md). Generated measurements: [analysis.json](analysis.json).",
        "",
        "| # | Repository | Category | Snapshot | Source | License |",
        "|---:|---|---|---|---|---|",
    ]
    for entry in entries:
        lines.append(
            f'| {entry["number"]} | `{entry["repository"]}` | {entry["category"]} | '
            f'[{entry["filename"]}]({entry["filename"]}) | [upstream]({entry["readme_url"]}) | '
            f'`{entry["license_spdx"]}` |'
        )
    lines.extend(
        [
            "",
            "## What to study",
            "",
            "- **First screen:** name, one-sentence promise, visual identity, and trust signals.",
            "- **Audience and problem:** how quickly the intended user and concrete outcome become clear.",
            "- **Progressive disclosure:** the transition from promise to proof, quick start, deeper docs, and contribution guidance.",
            "- **Operational honesty:** maturity, prerequisites, platform limits, security, support, and licensing.",
            "- **Actionability:** whether commands are copyable and whether the expected result is visible.",
            "- **Navigation:** how a large project keeps the root README useful without duplicating its documentation site.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> None:
    if len({repo for repo, _ in SOURCES}) != len(SOURCES):
        raise RuntimeError("The curated source list contains a duplicate repository")

    token = github_token()
    entries = []
    with ThreadPoolExecutor() as pool:
        futures = {
            pool.submit(fetch_source, index, repo, category, token): repo
            for index, (repo, category) in enumerate(SOURCES, start=int(bool(SOURCES)))
        }
        for future in as_completed(futures):
            entries.append(future.result())
    entries.sort(key=lambda entry: entry["number"])

    OUTPUT.mkdir(parents=True, exist_ok=True)
    expected = {entry["filename"] for entry in entries}
    for old_snapshot in OUTPUT.glob("[0-9][0-9]-*"):
        if old_snapshot.name not in expected:
            old_snapshot.unlink()

    for entry in entries:
        (OUTPUT / entry["filename"]).write_text(entry.pop("content"), encoding="utf-8")

    captured_at = datetime.now(timezone.utc).date().isoformat()
    metadata = {
        "schema": "wisent.readme-examples",
        "captured_at": captured_at,
        "count": len(entries),
        "sources": entries,
    }
    (OUTPUT / "sources.json").write_text(
        json.dumps(metadata, ensure_ascii=False, indent=len("  ")) + "\n",
        encoding="utf-8",
    )
    (OUTPUT / "README.md").write_text(render_index(entries, captured_at), encoding="utf-8")
    print(f"Wrote {len(entries)} README snapshots to {OUTPUT}")


if __name__ == "__main__":
    main()
