"""One definition of the reference-evidence contract, shared by every tool here.

`verify-reference-evidence.py` measures records against this contract and
`generate-example-catalogs.py` refuses to render anything that violates it. Both used
to carry their own copy of the vocabulary, which is how the corpus ended up with
`mp4`, `video/mp4`, `cast` and `asciinema-v2-terminal-cast` all meaning two things,
and how a validator came to require fields (`states[].name`,
`states[].source_motion_path`) that no record has ever had.

Change a rule here and both tools change with it.
"""

from __future__ import annotations

import re

CATALOG_SCHEMA = "wisent.example-catalog.v2"
RECORD_SCHEMA = "wisent.full-product-reference.v2"
INDEX_SCHEMA = "wisent.full-reference-catalog.v2"

# Declared media kind (any historical spelling) -> canonical kind.
MOTION_KINDS = {
    "mp4": "video-mp4",
    "video/mp4": "video-mp4",
    "h264": "video-mp4",
    "video-mp4": "video-mp4",
    "webm": "video-webm",
    "video/webm": "video-webm",
    "video-webm": "video-webm",
    "gif": "animated-gif",
    "image/gif": "animated-gif",
    "animated-gif": "animated-gif",
    "webp": "animated-webp",
    "image/webp": "animated-webp",
    "animated-webp": "animated-webp",
    "cast": "terminal-cast",
    "asciinema-v2-terminal-cast": "terminal-cast",
    "terminal-cast": "terminal-cast",
}

CANONICAL_MOTION_KINDS = frozenset(MOTION_KINDS.values())
STILL_KIND = "still-image"

# ffprobe container name -> canonical kind.
CONTAINER_KIND = {
    "mov,mp4,m4a,3gp,3g2,mj2": "video-mp4",
    "matroska,webm": "video-webm",
    "gif": "animated-gif",
    "webp_pipe": "animated-webp",
    "webp": "animated-webp",
    "image2": STILL_KIND,
    "png_pipe": STILL_KIND,
    "mjpeg": STILL_KIND,
}

MOTION_SUFFIXES = frozenset({".gif", ".webp", ".mp4", ".webm", ".cast"})
STATE_SUFFIXES = frozenset({".png", ".webp", ".jpg", ".jpeg"})

# How the motion was obtained, strongest evidence first. Only the first two are a
# product this workspace drove; the third is media the product's owner published.
PROVENANCE_CLASSES = (
    "local-product-run",
    "local-browser-recording",
    "upstream-owner-media",
    "unclassified",
)
LOCAL_PROVENANCE = frozenset({"local-product-run", "local-browser-recording"})

LOCAL_RUN_PATTERNS = (
    r"\bpseudo-?terminal\b",
    r"\bPTY\b",
    r"asciinema",
    r"terminal cast",
    r"real executable",
    r"stdout/stderr",
    r"real installed product recorded",
    r"isolated temporary working directory",
    r"local run of the installed product",
)

LOCAL_BROWSER_PATTERNS = (
    r"\bWeles\b",
    r"patched Chromium",
    r"browser recording",
    r"screencast",
)

UPSTREAM_PATTERNS = (
    r"yt-dlp",
    r"YouTube",
    r"Cobalt",
    r"download of",
    r"direct download",
    r"official[- ][\w -]*(media|asset|recording|stream|preview|trailer|tour|download)",
    r"official-product",
    r"product-site",
    r"apptrailers",
    r"play-games",
    r"publisher",
    r"video channel",
    r"downloaded",
)

# An upstream animated asset can legitimately be a fraction of a second long; the
# floor only rejects an asset too short to show a transition at all.
MIN_MOTION_SECONDS = 0.2
MIN_MOTION_FRAMES = 2
MIN_STATES = 3
MIN_JOURNEY_STEPS = 5
MIN_INTERACTIONS = 8
MIN_ACCESSIBILITY_OBSERVATIONS = 3
STATE_MATCH_MAX_DIFF = 12  # mean abs difference, 0-255, for a proven frame match

INTERACTION_FIELDS = (
    "name",
    "trigger",
    "response",
    "feedback",
    "cancellation",
    "failure",
    "recovery",
    "evidence",
)

MOTION_ANALYSIS_FIELDS = (
    "trigger",
    "start_state",
    "end_state",
    "continuity",
    "timing_class",
    "interruption_or_reversal",
    "feedback",
    "reduced_motion_equivalent",
)

# The corpus spelled two of these fields three different ways. Records are rewritten
# to the canonical spelling; the aliases stay here so an old record still normalizes.
MOTION_ANALYSIS_ALIASES = {
    "interruption_reversal": "interruption_or_reversal",
    "interruption_and_reversal": "interruption_or_reversal",
    "reduced_motion_or_nonanimated_equivalent": "reduced_motion_equivalent",
}

# Extra keys a motion analysis may carry beyond the required eight.
MOTION_ANALYSIS_OPTIONAL = ("source_title", "evidence", "timing_description")

TIMING_CLASSES = frozenset(
    {"instant", "sub-second", "one-to-three-seconds", "multi-second", "continuous"}
)

TIMING_CLASS_ALIASES = {
    "direct-manipulation feedback followed by a short product transition": "one-to-three-seconds",
    "immediate selection feedback followed by short asynchronous settling within the 15-second excerpt.": "one-to-three-seconds",
    "immediate control feedback followed by task-dependent result feedback": "one-to-three-seconds",
    "extended guided walkthrough": "multi-second",
    "extended guided demonstration": "multi-second",
    "brief component feedback": "sub-second",
    "short guided sequence": "one-to-three-seconds",
    "rapid microinteraction": "sub-second",
}


def canonical_timing_class(value: str | None) -> str | None:
    if value is None:
        return None
    normalized = value.strip().lower()
    if normalized in TIMING_CLASSES:
        return normalized
    return TIMING_CLASS_ALIASES.get(normalized)

JOURNEY_FIELDS = (
    "actor",
    "goal",
    "prerequisites",
    "steps",
    "failure_route",
    "recovery_route",
    "completion_evidence",
)

JOURNEY_STEP_FIELDS = ("index", "user_action", "system_response", "state", "evidence")

RECORD_FIELDS = (
    "schema",
    "name",
    "product_url",
    "evidence_status",
    "evidence_gaps",
    "upstream_owner",
    "captured_at",
    "motion",
    "motion_provenance",
    "states",
    "interactions",
    "journey",
    "accessibility",
)

EVIDENCE_STATUSES = frozenset({"complete", "partial"})


def canonical_motion_kind(declared: str | None) -> str | None:
    if declared is None:
        return None
    return MOTION_KINDS.get(str(declared).strip().lower())


def classify_provenance(capture_method: str | None, media_kind: str | None) -> str:
    """Derive the provenance class from the recorded capture method.

    The order matters: a locally driven run wins over any wording about the source,
    because a cast we recorded of a product we installed is our observation even when
    the product's own site is cited as the subject.
    """
    text = capture_method or ""
    for pattern in LOCAL_RUN_PATTERNS:
        if re.search(pattern, text, re.I):
            return "local-product-run"
    for pattern in LOCAL_BROWSER_PATTERNS:
        if re.search(pattern, text, re.I):
            return "local-browser-recording"
    for pattern in UPSTREAM_PATTERNS:
        if re.search(pattern, text, re.I):
            return "upstream-owner-media"
    if media_kind == "terminal-cast":
        return "local-product-run"
    return "unclassified"
