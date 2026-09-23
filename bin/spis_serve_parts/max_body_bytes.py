"""Parts of spis-serve, split by the tama size splitter; spis-serve imports every name back."""

from __future__ import annotations
import threading
from pathlib import Path


MAX_BODY_BYTES = 1024 * 1024

# One streamed job at a time: each job's log events stay on its own response.
JOB_LOCK = threading.Lock()


class ServeError(Exception):
    """An error that becomes a non-2xx {"error": sentence} envelope."""

    def __init__(self, status: int, message: str):
        super().__init__(message)
        self.status = status


def bad_request(message: str) -> ServeError:
    return ServeError(400, message)


def require_text(body: dict, key: str, sentence: str) -> str:
    value = body.get(key)
    if not isinstance(value, str) or not value.strip():
        raise bad_request(sentence)
    return value.strip()


def operation_argv(name: str, body: dict) -> list[str]:
    """Map one endpoint to the exact spis subcommand invocation it replaces."""
    if name == "corpus-adopt":
        path = require_text(body, "path", "adopting a corpus requires its directory path")
        if not Path(path).is_absolute():
            raise bad_request("adopting a corpus requires an absolute directory path")
        return ["corpus", "adopt", path]
    if name == "catalogs-check":
        return ["generate-example-catalogs", "--check"]
    if name == "drift":
        return ["check-upstream-drift"]
    if name == "verify":
        return ["verify-reference-evidence"]
    if name == "capture-dry-run":
        catalog = require_text(body, "catalog", "a capture plan requires a catalog slug")
        return ["capture-widths", catalog, "--dry-run"]
    if name == "docs-status":
        return ["docs-corpus", "status"]
    if name == "docs-search":
        query = require_text(body, "query", "searching the docs corpus requires a query")
        argv = ["docs-corpus", "search", "--query", query]
        site = body.get("site")
        if isinstance(site, str) and site.strip():
            argv += ["--site", site.strip()]
        limit = body.get("limit")
        if limit is not None:
            if not isinstance(limit, int) or isinstance(limit, bool) or limit < 1:
                raise bad_request("a docs search limit must be a positive number")
            argv += ["--limit", str(limit)]
        return argv
    if name == "docs-show":
        site = require_text(body, "site", "reading a docs page requires a site slug")
        url = require_text(body, "url", "reading a docs page requires its URL")
        return ["docs-corpus", "show", "--site", site, "--url", url]
    if name == "guidelines":
        slug = require_text(body, "slug", "deriving guidelines requires a catalog slug")
        return ["guidelines", slug]
    if name == "reference-add":
        slug = require_text(body, "slug", "adding a record requires a catalog slug")
        record_name = require_text(body, "name", "adding a record requires a name")
        source_url = require_text(body, "sourceUrl", "adding a record requires a source URL")
        category = require_text(body, "category", "adding a record requires a category")
        note = require_text(body, "selectionNote", "adding a record requires a selection note")
        visual = require_text(body, "visual", "adding a record requires an image path")
        return [
            "reference-record", "add", slug,
            "--name", record_name,
            "--source-url", source_url,
            "--category", category,
            "--selection-note", note,
            "--visual", visual,
        ]
    if name == "reference-remove":
        slug = require_text(body, "slug", "removing a record requires a catalog slug")
        number = body.get("number")
        if not isinstance(number, int) or isinstance(number, bool) or number < 1:
            raise bad_request("removing a record requires its number")
        argv = ["reference-record", "remove", slug, str(number)]
        if body.get("force"):
            argv.append("--force")
        return argv
    raise ServeError(404, f"unknown endpoint: POST /v1/{name}")


def refusal_sentence(stderr_text: str) -> str:
    """The tool's own refusal: its last stderr line, without the log prefix."""
    lines = [line.strip() for line in stderr_text.splitlines() if line.strip()]
    if not lines:
        return "The operation failed without reporting a reason."
    sentence = lines[-1]
    if sentence.startswith("error: "):
        sentence = sentence[len("error: "):]
    return sentence


# Endpoints that answer with the subcommand's JSON document instead of a stream.
JSON_ENDPOINTS = {"docs-status", "docs-search", "docs-show"}
