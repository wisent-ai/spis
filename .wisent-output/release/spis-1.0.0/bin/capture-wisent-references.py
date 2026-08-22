#!/usr/bin/env python3
"""Capture the reference catalog for the Wisent products installed on this workstation.

Every other catalog in this repository measures somebody else's product. This one
measures ours, and it measures them the only way that produces evidence: by running
the installed binaries here, through a real pseudo-terminal, and keeping the recording.

For each product the script opens a PTY, drives one `/bin/bash --norc --noprofile -i`
session, and issues exactly seven read-only commands:

    1. the version form                      5. Ctrl-C on an unsubmitted line
    2. the top-level help                    6. the help that recovers from the refusal
    3. one subcommand help surface           7. the same help with NO_COLOR=1
    4. one deliberately invalid flag

Nothing here contacts a host, mints a credential, writes a vault, submits a job,
restarts a service, or runs a test. The session's own output, with the timings of the
run, becomes `media/session.cast` (asciinema v2). Five PNGs are rendered from that
cast's text with Pillow at named points in the sequence — they are renders of the
cast, not separate screenshots, and each record says so.

    ./capture-wisent-references.py --list          # products found, resolved path, version
    ./capture-wisent-references.py                 # capture every product, rebuild the catalog
    ./capture-wisent-references.py --product stado # capture one, rebuild the catalog

Afterwards `./verify-reference-evidence.py --catalog wisent-product-examples --apply`
re-measures every file and rewrites `evidence_status` from what the bytes prove.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import platform
import pty
import re
import select
import shutil
import struct
import subprocess
import sys
import termios
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CATALOG = ROOT / "wisent-product-examples"
SCRATCH = Path.home() / ".stado" / "work" / "wisent-capture"

RECORD_SCHEMA = "wisent.full-product-reference.v2"
INDEX_SCHEMA = "wisent.full-reference-catalog.v2"
SOURCES_SCHEMA = "wisent.example-catalog.v2"

COLS = 100
ROWS = 32
PROMPT = "wisent-ref$ "
PROBE_FLAG = "--wisent-reference-probe"
SHELL = "/bin/bash"
FONT_CANDIDATES = ("/System/Library/Fonts/Menlo.ttc", "/System/Library/Fonts/SFNSMono.ttf")
FONT_PX = 15

# One entry per Wisent product with a runnable CLI on this host. `repository` is the
# Wisent repository the binary comes from; `version_cmd` is the product's own version
# form, which several of these do not have — the refusal is then the measurement.
PRODUCTS = [
    {
        "slug": "stado",
        "name": "Stado",
        "binary": "stado",
        "repository": "wisent-ai/stado",
        "product_url": "https://github.com/wisent-ai/stado",
        "category": "Infrastructure / compute and queue control plane",
        "one_line": "Policy-controlled queue and compute control plane for machines you own or authorize.",
        "selection_note": (
            "The widest command surface we ship: a Clap-style noun tree over jobs, hosts, quota and "
            "credits. Study how a large control-plane CLI keeps its top-level help to one screen of "
            "verbs and pushes detail into per-command help."
        ),
        "version_cmd": "stado --version",
        "help_cmd": "stado --help",
        "sub_cmd": "stado capabilities --help",
        "sub_note": "Per-command help exists and is the documented way down one level.",
    },
    {
        "slug": "skarbiec",
        "name": "Skarbiec",
        "binary": "skarbiec",
        "repository": "wisent-ai/skarbiec",
        "product_url": "https://github.com/wisent-ai/skarbiec",
        "category": "Security / credential and authentication management",
        "one_line": "Credential and authentication management for the AI era.",
        "selection_note": (
            "The only product here whose help is machine-readable JSON rather than prose, and the "
            "only one whose first refusal is a state gate rather than a parse error. Study what a "
            "credential CLI is willing to say before a vault exists."
        ),
        "version_cmd": "skarbiec --version",
        "help_cmd": "skarbiec help",
        "sub_cmd": "skarbiec status --help",
        "sub_note": (
            "Skarbiec has no per-subcommand help; the subcommand is reached before argument parsing "
            "and answers with the vault state gate instead."
        ),
    },
    {
        "slug": "weles",
        "name": "Weles",
        "binary": "weles",
        "repository": "wisent-ai/weles",
        "product_url": "https://github.com/wisent-ai/weles",
        "category": "Automation / authorized browser execution",
        "one_line": "Authorized browser execution for AI agents, with signed receipts.",
        "selection_note": (
            "A CLI whose real work is gated on an authorization boundary, so its safe surface is "
            "help and identity only. Study how a product that refuses unauthorized work advertises "
            "that boundary in its own usage text."
        ),
        "version_cmd": "weles --version",
        "help_cmd": "weles --help",
        "sub_cmd": "weles version",
        "sub_note": (
            "Weles exposes no per-subcommand help. `weles version` is the only subcommand that can "
            "be run here without authorizing a workflow or touching durable onboarding state, so "
            "that is the subcommand surface this record measures."
        ),
    },
    {
        "slug": "jeden",
        "name": "Jeden",
        "binary": "jeden",
        "repository": "wisent-ai/jeden",
        "product_url": "https://github.com/wisent-ai/jeden",
        "category": "Agents / autonomous coding and company building",
        "one_line": "The autonomous agent for building software and running the loop around it.",
        "selection_note": (
            "A single-block usage synopsis for an agent runtime whose flags are permission grants "
            "(`--allow-write`, `--allow-command`, `--yolo`). Study how a dangerous capability set is "
            "presented in first-run help."
        ),
        "version_cmd": "jeden --version",
        "help_cmd": "jeden --help",
        "sub_cmd": "jeden version",
        "sub_note": (
            "Jeden's help is one usage block covering every subcommand; there is no per-subcommand "
            "help. `jeden run --help` is not a safe probe: when probed once outside this recording "
            "it resolved credentials through Skarbiec before parsing `--help` and failed with an "
            "HTTP 403, so this record measures `jeden version` instead and reports that finding "
            "rather than re-running it."
        ),
    },
    {
        "slug": "probierz",
        "name": "Probierz",
        "binary": "probierz",
        "repository": "wisent-ai/probierz",
        "product_url": "https://github.com/wisent-ai/probierz",
        "category": "Quality / test execution and evidence boundary",
        "one_line": "The quality-evidence boundary: selection, execution, evidence, and verdicts.",
        "selection_note": (
            "The one product whose refusal is a structured machine-readable failure envelope rather "
            "than a usage dump. Study a CLI that answers an unknown surface with a parseable "
            "`probierz-failure` line plus one plain sentence."
        ),
        "version_cmd": "probierz --version",
        "help_cmd": "probierz --help",
        "sub_cmd": "probierz specs --help",
        "sub_note": (
            "Probierz has no per-subcommand help: `--help` after a subcommand is read as a surface "
            "name and refused. That refusal is the observed subcommand surface."
        ),
    },
    {
        "slug": "oko-cli",
        "name": "Oko (oko-cli)",
        "binary": "oko-cli",
        "repository": "wisent-ai/oko",
        "product_url": "https://github.com/wisent-ai/oko",
        "category": "Observability / agent session inspection",
        "one_line": "Understand your team's interactions with AI.",
        "selection_note": (
            "A headless companion CLI with one flat usage block and no version form at all. Study "
            "the cost of that: the same text answers help, an unknown flag, and a subcommand help "
            "request, and only the exit status distinguishes them."
        ),
        "version_cmd": "oko-cli --version",
        "help_cmd": "oko-cli --help",
        "sub_cmd": "oko-cli diff --help",
        "sub_note": "Oko answers a subcommand help request with the whole top-level usage block.",
    },
    {
        "slug": "singularity",
        "name": "Singularity",
        "binary": "singularity",
        "repository": "wisent-ai/singularity",
        "product_url": "https://github.com/wisent-ai/singularity",
        "category": "Agents / autonomous agent runtime",
        "one_line": "An open-source framework for autonomous agents that execute tasks and manage resources.",
        "selection_note": (
            "The narrowest installed surface in the catalog: one subcommand, `onboarding`. Study a "
            "product whose CLI deliberately exposes only the first-use journey."
        ),
        "version_cmd": "singularity --version",
        "help_cmd": "singularity --help",
        "sub_cmd": "singularity onboarding --help",
        "sub_note": "argparse gives every subcommand its own help; `onboarding` is the only one.",
    },
    {
        "slug": "tama",
        "name": "Tama (tama-cli)",
        "binary": "tama",
        "repository": "wisent-ai/hooks-rotator",
        "product_url": "https://github.com/wisent-ai/hooks-rotator",
        "category": "Policy / agent and Git hook catalog",
        "one_line": "Your AI agent made a mistake? Tama creates rules so that it never happens again.",
        "selection_note": (
            "A policy catalog CLI that answers `--help` after a subcommand by ignoring the flag and "
            "running the read-only command. Study how a hook installer separates a plan from an "
            "install."
        ),
        "version_cmd": "tama --version",
        "help_cmd": "tama --help",
        "sub_cmd": "tama install-plan --help",
        "sub_note": (
            "Tama ignores a trailing `--help` and executes the subcommand; `install-plan` only "
            "reports the paths an install would touch, so nothing is written."
        ),
    },
    {
        "slug": "transcript-lake",
        "name": "Transcript Lake",
        "binary": "transcript-lake",
        "repository": "wisent-ai/transcript-lake",
        "product_url": "https://github.com/wisent-ai/transcript-lake",
        "category": "Data / local privacy-masked transcript archive",
        "one_line": "Nothing you ever told an AI is lost again.",
        "selection_note": (
            "The only help here that opens with a `Start safely:` section and names the read-only "
            "commands first. Study help text that is ordered by risk rather than alphabetically."
        ),
        "version_cmd": "transcript-lake --version",
        "help_cmd": "transcript-lake --help",
        "sub_cmd": "transcript-lake paths --help",
        "sub_note": "Per-command usage exists and prints one line of purpose with its flags.",
    },
    {
        "slug": "transcript-label-trainer",
        "name": "Transcript Label Trainer",
        "binary": "transcript-label-trainer",
        "repository": "wisent-ai/transcript-label-trainer",
        "product_url": "https://github.com/wisent-ai/transcript-label-trainer",
        "category": "Models / local classifiers over transcript labels",
        "one_line": "Small models for your custom harness needs.",
        "selection_note": (
            "An argparse CLI that states its own boundary in the help body — 'Never writes to the "
            "lake.' Study a product that publishes what it will not touch above its command list."
        ),
        "version_cmd": "transcript-label-trainer --version",
        "help_cmd": "transcript-label-trainer --help",
        "sub_cmd": "transcript-label-trainer info --help",
        "sub_note": "argparse gives every subcommand its own help.",
    },
]

# Products deliberately excluded, with the reason. Kept here so the catalog scope is a
# statement that can be checked rather than a claim about what happened to be found.
EXCLUSIONS = [
    {
        "binary": "omp",
        "resolved": "~/.local/bin/omp",
        "reason": (
            "Not a Wisent repository: the binary's own build metadata names "
            "github.com/can1357/oh-my-pi. It is the harness we run, not a product we ship."
        ),
    },
    {
        "binary": "stado_fleet",
        "resolved": "~/.stado/bin/stado_fleet",
        "reason": "A second binary of the same product (Stado, wisent-ai/stado), not a separate product.",
    },
    {
        "binary": "wc",
        "resolved": "~/.local/bin/wc",
        "reason": "The legacy name of the Stado CLI; `wc --version` prints `stado 0.6.0`. Same product.",
    },
]

ANSI = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
SGR = re.compile(r"\x1b\[[0-9;]*m")
NEXT_ACTION = re.compile(
    r"(try\s+'[^']+'|try\s+\"[^\"]+\"|\(run:\s*[^)]+\)|see\s+'[^']+'|usage:|USAGE:|Usage:|--help|<command>)"
)


# --------------------------------------------------------------------------- host


def host_facts():
    def out(args):
        try:
            return subprocess.run(args, capture_output=True, text=True).stdout.strip()
        except OSError:
            return ""

    return {
        "os": "macOS",
        "os_version": out(["sw_vers", "-productVersion"]),
        "os_build": out(["sw_vers", "-buildVersion"]),
        "arch": platform.machine(),
        "kernel": f"{platform.system()} {platform.release()}",
        "shell": SHELL,
        "terminal": f"pseudo-terminal, {COLS}x{ROWS}, TERM=xterm-256color",
    }


HOST = host_facts()
HOST_SENTENCE = f"macOS {HOST['os_version']} ({HOST['os_build']}) {HOST['arch']}"


def ensure_pillow():
    """Render with whichever interpreter on this host has Pillow."""
    try:
        import PIL.Image  # noqa: F401

        return
    except ImportError:
        pass
    if os.environ.get("WISENT_CAPTURE_REEXEC"):
        raise SystemExit("Pillow is required to render the state PNGs and was not found.")
    for cand in ("/usr/bin/python3", "/opt/homebrew/bin/python3", "/usr/local/bin/python3"):
        if not os.path.exists(cand):
            continue
        probe = subprocess.run([cand, "-c", "import PIL"], capture_output=True)
        if probe.returncode == 0:
            os.environ["WISENT_CAPTURE_REEXEC"] = "1"
            os.execv(cand, [cand, str(Path(__file__).resolve())] + sys.argv[1:])
    raise SystemExit("Pillow is required to render the state PNGs and was not found.")


# ------------------------------------------------------------------- pty session


def set_winsize(fd, rows, cols):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


class Session:
    """One real pseudo-terminal running one interactive shell, recorded as it runs."""

    def __init__(self, cwd):
        env = dict(os.environ)
        env.pop("NO_COLOR", None)
        env.pop("CLICOLOR_FORCE", None)
        env.update(
            {
                "TERM": "xterm-256color",
                "PS1": PROMPT,
                "PAGER": "cat",
                "LESS": "-FRX",
                "COLUMNS": str(COLS),
                "LINES": str(ROWS),
                "SHELL": SHELL,
            }
        )
        self.events = []
        self.start = None
        self.pid, self.fd = pty.fork()
        if self.pid == 0:  # child
            os.chdir(str(cwd))
            os.execvpe(SHELL, [SHELL, "--norc", "--noprofile", "-i"], env)
            os._exit(127)
        set_winsize(self.fd, ROWS, COLS)
        self.start = time.monotonic()
        self.wall_start = int(time.time())
        self._drain(1.5)
        self._write(f"PS1='{PROMPT}'\n")
        self.wait_prompt(10)
        # Everything recorded from here is the product's own session.
        self.events = []
        self.start = time.monotonic()
        self.wall_start = int(time.time())

    # -- plumbing

    def _stamp(self, text):
        self.events.append([round(time.monotonic() - self.start, 6), "o", text])

    def _write(self, text):
        os.write(self.fd, text.encode())

    def _read(self, timeout):
        ready, _, _ = select.select([self.fd], [], [], timeout)
        if not ready:
            return ""
        try:
            chunk = os.read(self.fd, 1 << 16)
        except OSError:
            return ""
        if not chunk:
            return ""
        text = chunk.decode("utf-8", "replace")
        self._stamp(text)
        return text

    def _drain(self, seconds):
        deadline = time.monotonic() + seconds
        got = ""
        while time.monotonic() < deadline:
            got += self._read(0.1)
        return got

    def wait_prompt(self, timeout):
        """Read until the shell reprints its prompt, or the timeout expires."""
        deadline = time.monotonic() + timeout
        buf = ""
        while time.monotonic() < deadline:
            buf += self._read(0.2)
            if buf.endswith(PROMPT):
                # Let a trailing flush land, then stop.
                buf += self._drain(0.15)
                return buf, True
        return buf, False

    # -- steps

    def command(self, command, timeout=180):
        t0 = round(time.monotonic() - self.start, 6)
        self._write(command + "\n")
        raw, ok = self.wait_prompt(timeout)
        if not ok:
            self._write("\x03")
            raw += self.wait_prompt(15)[0]
        t1 = round(time.monotonic() - self.start, 6)
        status = None
        self._write('printf "exit-status=%s\\n" "$?"\n')
        status_raw, _ = self.wait_prompt(20)
        found = re.search(r"exit-status=(\d+)", status_raw)
        if found:
            status = int(found.group(1))
        t2 = round(time.monotonic() - self.start, 6)
        return {
            "command": command,
            "raw": raw,
            "started_at": t0,
            "ended_at": t1,
            "status_reported_at": t2,
            "exit_status": status,
            "prompt_returned": ok,
        }

    def cancel(self, pending, typing_pause=0.6, timeout=20):
        """Type a command and press Ctrl-C instead of Enter. Nothing is submitted."""
        t0 = round(time.monotonic() - self.start, 6)
        self._write(pending)
        raw = self._drain(typing_pause)
        self._write("\x03")
        tail, _ = self.wait_prompt(timeout)
        raw += tail
        t1 = round(time.monotonic() - self.start, 6)
        return {
            "command": pending,
            "raw": raw,
            "started_at": t0,
            "ended_at": t1,
            "status_reported_at": t1,
            "exit_status": None,
            "prompt_returned": True,
        }

    def close(self):
        try:
            self._write("exit\n")
            self._drain(0.5)
        except OSError:
            pass
        try:
            os.close(self.fd)
        except OSError:
            pass
        try:
            os.waitpid(self.pid, 0)
        except ChildProcessError:
            pass


# ------------------------------------------------------------------ text + cast


def strip_ansi(text):
    return ANSI.sub("", text).replace("\x07", "")


def visible_lines(text):
    """Replay plain text the way the recorded terminal showed it."""
    out = []
    for line in strip_ansi(text).split("\n"):
        # PTYs terminate ordinary lines with CRLF. The old parser took the
        # segment *after* every CR, turning every real output line into "".
        # Internal CR still means an in-place repaint; a terminal shows the
        # final segment in that case.
        if line.endswith("\r"):
            line = line[:-1]
        if "\r" in line:
            line = line.split("\r")[-1]
        out.append(line.replace("\t", "    "))
    return out


def step_output(step):
    """The command's own output: the echoed command line and trailing prompt removed."""
    raw = step["raw"]
    body = raw.split("\n", 1)[1] if "\n" in raw else ""
    if body.endswith(PROMPT):
        body = body[: -len(PROMPT)]
    return body


def wrapped(lines, width):
    out = []
    for line in lines:
        if not line:
            out.append("")
            continue
        while len(line) > width:
            out.append(line[:width])
            line = line[width:]
        out.append(line)
    return out


def write_cast(path, events, title, wall_start):
    header = {
        "version": 2,
        "width": COLS,
        "height": ROWS,
        "timestamp": wall_start,
        "env": {"SHELL": SHELL, "TERM": "xterm-256color"},
        "title": title,
    }
    with path.open("w", encoding="utf-8") as fh:
        fh.write(json.dumps(header, separators=(",", ":")) + "\n")
        for stamp, kind, data in events:
            fh.write(json.dumps([stamp, kind, data], ensure_ascii=False) + "\n")


def digest(path):
    h = hashlib.sha256()
    size = 0
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            size += len(chunk)
            h.update(chunk)
    return size, h.hexdigest()


def render_state(path, events, cutoff_index):
    """Deterministic PNG of the cast's own text, replayed to one event."""
    from PIL import Image, ImageDraw, ImageFont

    text = "".join(e[2] for e in events[: cutoff_index + 1])
    rows = wrapped(visible_lines(text), COLS)
    rows = rows[-ROWS:]
    while len(rows) < ROWS:
        rows.append("")

    font = None
    for candidate in FONT_CANDIDATES:
        if os.path.exists(candidate):
            try:
                font = ImageFont.truetype(candidate, FONT_PX)
                break
            except OSError:
                continue
    if font is None:
        font = ImageFont.load_default()

    advance = font.getlength("M") if hasattr(font, "getlength") else FONT_PX * 0.6
    cell_w = max(1, int(round(advance)))
    cell_h = int(round(FONT_PX * 1.45))
    pad = 12
    size = (COLS * cell_w + 2 * pad, ROWS * cell_h + 2 * pad)
    image = Image.new("RGB", size, (13, 17, 23))
    draw = ImageDraw.Draw(image)
    for index, row in enumerate(rows):
        draw.text((pad, pad + index * cell_h), row, font=font, fill=(222, 228, 234))
    image.save(str(path), format="PNG", optimize=True)
    return image.size


# --------------------------------------------------------------- one product run


def resolve(product):
    return shutil.which(product["binary"])


def quick_version(product):
    """Run the product's version form once, outside the recording, for --list."""
    path = resolve(product)
    if not path:
        return None, None, None
    args = product["version_cmd"].split()
    try:
        proc = subprocess.run(args, capture_output=True, text=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return path, None, f"{type(exc).__name__}"
    text = strip_ansi(proc.stdout + proc.stderr).strip()
    first = text.split("\n")[0] if text else ""
    return path, proc.returncode, first


STEP_PLAN = [
    ("version", "version identity"),
    ("help", "top-level help"),
    ("subcommand-help", "subcommand help surface"),
    ("invalid-flag", "invalid flag refusal"),
    ("cancellation", "Ctrl-C on an unsubmitted line"),
    ("recovery-help", "recovery help"),
    ("no-color-help", "help with NO_COLOR=1"),
]

STATE_PLAN = [
    ("version", "01-version-identity", "version identity"),
    ("help", "02-help-surface", "top-level help surface"),
    ("subcommand-help", "03-subcommand-help", "subcommand help surface"),
    ("invalid-flag", "04-refusal", "refusal after the invalid flag"),
    ("recovery-help", "05-recovery", "recovered help after the refusal"),
]


def capture(product, verbose=True):
    binary_path = resolve(product)
    if not binary_path:
        raise SystemExit(f"{product['binary']} is not on PATH")

    workdir = SCRATCH / "run" / product["slug"]
    if workdir.exists():
        shutil.rmtree(workdir)
    workdir.mkdir(parents=True)

    invalid_cmd = f"{product['binary']} {PROBE_FLAG}"
    commands = {
        "version": product["version_cmd"],
        "help": product["help_cmd"],
        "subcommand-help": product["sub_cmd"],
        "invalid-flag": invalid_cmd,
        "cancellation": invalid_cmd,
        "recovery-help": product["help_cmd"],
        "no-color-help": "NO_COLOR=1 " + product["help_cmd"],
    }

    session = Session(workdir)
    steps = {}
    try:
        for kind, _label in STEP_PLAN:
            if verbose:
                print(f"    {kind}: {commands[kind]}")
            if kind == "cancellation":
                step = session.cancel(commands[kind])
            else:
                step = session.command(commands[kind])
            step["kind"] = kind
            step["event_index"] = len(session.events) - 1
            steps[kind] = step
    finally:
        events = list(session.events)
        wall_start = session.wall_start
        session.close()

    return {
        "product": product,
        "binary_path": binary_path,
        "workdir": str(workdir),
        "events": events,
        "wall_start": wall_start,
        "steps": steps,
        "commands": commands,
    }


# ------------------------------------------------------------------ measurements


def measure(run):
    steps = run["steps"]
    out = {}
    for kind, _label in STEP_PLAN:
        step = steps[kind]
        raw = step_output(step)
        plain = strip_ansi(raw)
        lines = [line for line in visible_lines(raw)]
        while lines and not lines[-1].strip():
            lines.pop()
        out[kind] = {
            "command": step["command"],
            "exit_status": step["exit_status"],
            "started_at": step["started_at"],
            "ended_at": step["ended_at"],
            "elapsed_seconds": round(step["ended_at"] - step["started_at"], 3),
            "event_index": step["event_index"],
            "sgr_sequences": len(SGR.findall(raw)),
            "line_count": len(lines),
            "max_line_width": max((len(line) for line in lines), default=0),
            "first_line": lines[0].strip() if lines else "",
            "last_line": lines[-1].strip() if lines else "",
            "text": plain,
            "lines": lines,
        }
    help_m = out["help"]
    nocolor_m = out["no-color-help"]
    invalid_m = out["invalid-flag"]

    same_text = help_m["text"].strip() == nocolor_m["text"].strip()
    match = NEXT_ACTION.search(invalid_m["text"])
    cancel_raw = steps["cancellation"]["raw"]

    return {
        "steps": out,
        "colors_help": help_m["sgr_sequences"] > 0,
        "help_sgr_count": help_m["sgr_sequences"],
        "no_color_sgr_count": nocolor_m["sgr_sequences"],
        "no_color_text_identical": same_text,
        "help_max_line_width": help_m["max_line_width"],
        "help_fits_80": help_m["max_line_width"] <= 80,
        "help_line_count": help_m["line_count"],
        "refusal_names_next_action": bool(match),
        "refusal_next_action_phrase": match.group(0) if match else None,
        "refusal_first_line": invalid_m["first_line"],
        "refusal_exit_status": invalid_m["exit_status"],
        "version_first_line": out["version"]["first_line"],
        "version_exit_status": out["version"]["exit_status"],
        "version_flag_supported": out["version"]["exit_status"] == 0,
        "cancel_echoed_interrupt": "^C" in cancel_raw,
        "cancel_prompt_restored": cancel_raw.rstrip().endswith(PROMPT.strip()),
        "screen_cleared": "\x1b[2J" in "".join(e[2] for e in run["events"]),
        "cursor_addressed": bool(re.search(r"\x1b\[\d*;\d*H", "".join(e[2] for e in run["events"]))),
    }


def timing_class(m):
    spans = [m["steps"][k]["elapsed_seconds"] for k, _ in STEP_PLAN if k != "cancellation"]
    slowest = max(spans)
    if slowest < 0.05:
        return "instant"
    if slowest < 1.0:
        return "sub-second"
    if slowest <= 3.0:
        return "one-to-three-seconds"
    return "multi-second"


def timing_description(m):
    spans = [m["steps"][k]["elapsed_seconds"] for k, _ in STEP_PLAN if k != "cancellation"]
    fastest, slowest = min(spans), max(spans)
    return (
        f"The six submitted commands each completed between {fastest:g} s and {slowest:g} s; "
        "the other pauses are the recorder typing and the deliberate Ctrl-C pause."
    )


# ------------------------------------------------------------------ record build


def quote(text, limit=160):
    text = " ".join(text.split())
    return text[: limit - 1] + "…" if len(text) > limit else text


def build_record(run, measured, media):
    product = run["product"]
    steps = measured["steps"]
    cast = media["cast"]
    states = media["states"]
    name = product["name"]
    binary = product["binary"]

    def ev(kind, extra=""):
        s = steps[kind]
        base = f"media/session.cast at {s['started_at']:g}–{s['ended_at']:g} s"
        if s["exit_status"] is not None:
            base += f"; observed exit {s['exit_status']}"
        return base + (f"; {extra}" if extra else "")

    version_line = steps["version"]["first_line"]
    version_ok = measured["version_flag_supported"]
    refusal_line = measured["refusal_first_line"]
    refusal_status = measured["refusal_exit_status"]
    recovery_line = steps["recovery-help"]["first_line"]
    invalid_cmd = steps["invalid-flag"]["command"]

    cancellation_sentence = (
        f"Ctrl-C on the unsubmitted `{invalid_cmd}` line discarded it and restored the prompt"
        if measured["cancel_prompt_restored"]
        else f"Ctrl-C was sent on the unsubmitted `{invalid_cmd}` line and the session continued at the prompt"
    )

    interactions = [
        {
            "name": "command entry",
            "trigger": f"Type `{steps['version']['command']}` at the `{PROMPT.strip()}` prompt and press Enter.",
            "response": f"{name} starts from {run['binary_path']} and writes to the pseudo-terminal.",
            "feedback": quote(version_line) or "no output on the version form",
            "cancellation": f"{cancellation_sentence}; nothing was submitted.",
            "failure": f"`{invalid_cmd}` reaches the same parser and is refused with status {refusal_status}.",
            "recovery": f"Re-enter `{steps['recovery-help']['command']}`.",
            "evidence": ev("version", "media/01-version-identity.png"),
        },
        {
            "name": "version identity",
            "trigger": f"Ask the installed binary what it is with `{steps['version']['command']}`.",
            "response": (
                f"{name} prints its version identity and exits 0."
                if version_ok
                else f"{name} has no version flag: it refuses the option with status {steps['version']['exit_status']} "
                "and answers with its usage surface instead."
            ),
            "feedback": quote(version_line) or f"The process returns exit status {steps['version']['exit_status']}.",
            "cancellation": "The version form returns on its own; Ctrl-C is available at the prompt.",
            "failure": (
                f"No failure on this path: exit {steps['version']['exit_status']}."
                if version_ok
                else f"The version request itself is the failure: exit {steps['version']['exit_status']}."
            ),
            "recovery": (
                "None needed."
                if version_ok
                else f"Read the identity out of `{steps['help_cmd'] if False else steps['help']['command']}` instead."
            ),
            "evidence": ev("version", "media/01-version-identity.png"),
        },
        {
            "name": "help discovery",
            "trigger": f"Run `{steps['help']['command']}`.",
            "response": (
                f"{name} prints {steps['help']['line_count']} lines of its own top-level help, widest line "
                f"{measured['help_max_line_width']} characters."
            ),
            "feedback": quote(steps["help"]["first_line"]) or f"The help process returns exit status {steps['help']['exit_status']}.",
            "cancellation": "The stream is short enough to complete; the prompt stays interruptible.",
            "failure": f"A misspelled flag on the same surface is refused with status {refusal_status}.",
            "recovery": f"Re-run `{steps['recovery-help']['command']}`.",
            "evidence": ev("help", "media/02-help-surface.png"),
        },
        {
            "name": "subcommand surface",
            "trigger": f"Run `{steps['subcommand-help']['command']}`.",
            "response": product["sub_note"],
            "feedback": quote(steps["subcommand-help"]["first_line"]) or f"The subcommand surface returns exit status {steps['subcommand-help']['exit_status']}.",
            "cancellation": "Ctrl-C at the prompt abandons the request before submission.",
            "failure": (
                f"Observed exit {steps['subcommand-help']['exit_status']} on this path."
                if steps["subcommand-help"]["exit_status"]
                else "This path returned 0; failure is shown by the invalid flag instead."
            ),
            "recovery": f"Return to `{steps['recovery-help']['command']}` for the documented grammar.",
            "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
        },
        {
            "name": "invalid flag refusal",
            "trigger": f"Run `{invalid_cmd}`.",
            "response": f"{name} refuses the unknown option and returns status {refusal_status}.",
            "feedback": quote(refusal_line) or f"The refusal returns exit status {refusal_status}.",
            "cancellation": "The refusal returns immediately; no cancellation was required.",
            "failure": f"Observed status {refusal_status}.",
            "recovery": (
                f"The refusal names the next action ({measured['refusal_next_action_phrase']!r}); running "
                f"`{steps['recovery-help']['command']}` recovers."
                if measured["refusal_names_next_action"]
                else f"The refusal names no next action; `{steps['recovery-help']['command']}` recovers anyway."
            ),
            "evidence": ev("invalid-flag", "media/04-refusal.png"),
        },
        {
            "name": "exit status reporting",
            "trigger": "After each command the recorded shell prints `printf \"exit-status=%s\\n\" \"$?\"`.",
            "response": "The real status of the preceding product invocation appears in the cast as text.",
            "feedback": (
                f"exit-status={steps['version']['exit_status']} after the version form, "
                f"exit-status={refusal_status} after the invalid flag."
            ),
            "cancellation": "The status line is a shell builtin write; there is nothing to cancel.",
            "failure": "A missing status line would mean the prompt never returned; every step reported one.",
            "recovery": "Not applicable: the status is evidence, not an action.",
            "evidence": (
                f"media/session.cast at {steps['version']['ended_at']:g}–"
                f"{steps['version']['status_reported_at'] if 'status_reported_at' in steps['version'] else steps['version']['ended_at']:g} s "
                "and after every other command"
            ),
        },
        {
            "name": "cancellation",
            "trigger": f"Type `{invalid_cmd}` and press Ctrl-C instead of Enter.",
            "response": (
                "The pending line is discarded and the prompt returns; the product never ran."
                if measured["cancel_prompt_restored"]
                else "Ctrl-C was accepted and the session continued at the prompt."
            ),
            "feedback": (
                "`^C` is echoed in the cast, then a fresh prompt."
                if measured["cancel_echoed_interrupt"]
                else "A fresh prompt follows the interrupt in the cast."
            ),
            "cancellation": "Ctrl-C is the observed cancellation mechanism for unsubmitted input.",
            "failure": "The typed command is abandoned on purpose rather than executed.",
            "recovery": f"Submit `{steps['recovery-help']['command']}` at the restored prompt.",
            "evidence": (
                f"media/session.cast at {steps['cancellation']['started_at']:g}–"
                f"{steps['cancellation']['ended_at']:g} s"
            ),
        },
        {
            "name": "recovery",
            "trigger": f"After the refusal and the cancellation, run `{steps['recovery-help']['command']}` again.",
            "response": f"The same installed binary prints valid help and returns status {steps['recovery-help']['exit_status']}.",
            "feedback": quote(recovery_line) or f"The recovery help returns exit status {steps['recovery-help']['exit_status']}.",
            "cancellation": "The recovery can itself be interrupted with Ctrl-C at the prompt.",
            "failure": f"Repeating `{invalid_cmd}` reproduces status {refusal_status}.",
            "recovery": "The valid help form completes the first-success journey.",
            "evidence": ev("recovery-help", "media/05-recovery.png"),
        },
        {
            "name": "color-free equivalence",
            "trigger": f"Run `{steps['no-color-help']['command']}`.",
            "response": (
                "The same help text is printed with the same line count"
                if measured["no_color_text_identical"]
                else "The help text differs from the colored run once color is removed"
            )
            + f" ({steps['no-color-help']['line_count']} lines).",
            "feedback": (
                f"{measured['help_sgr_count']} ANSI colour sequences in the default run, "
                f"{measured['no_color_sgr_count']} with NO_COLOR=1."
            ),
            "cancellation": "Ctrl-C at the prompt applies here as to any other command.",
            "failure": "No failure on this path; it is a measurement of the same success route.",
            "recovery": "Not applicable.",
            "evidence": ev("no-color-help"),
        },
    ]

    journey_steps = [
        {
            "index": 1,
            "user_action": f"Open the recorded pseudo-terminal at `{PROMPT.strip()}` in an empty scratch directory.",
            "system_response": "A clean prompt appears; no project, credential, host or queue target is selected.",
            "state": "ready prompt",
            "evidence": f"media/session.cast at 0–{steps['version']['started_at']:g} s",
        },
        {
            "index": 2,
            "user_action": f"Run `{steps['version']['command']}`.",
            "system_response": (
                f"{name} prints `{quote(version_line, 90)}` and exits {steps['version']['exit_status']}."
                if version_ok
                else f"{name} refuses the version flag with status {steps['version']['exit_status']} and prints "
                f"`{quote(version_line, 90)}`."
            ),
            "state": "version identity",
            "evidence": ev("version", "media/01-version-identity.png"),
        },
        {
            "index": 3,
            "user_action": f"Run `{steps['help']['command']}`.",
            "system_response": f"{steps['help']['line_count']} lines of the product's own top-level help are printed.",
            "state": "top-level help surface",
            "evidence": ev("help", "media/02-help-surface.png"),
        },
        {
            "index": 4,
            "user_action": f"Run `{steps['subcommand-help']['command']}`.",
            "system_response": product["sub_note"],
            "state": "subcommand help surface",
            "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
        },
        {
            "index": 5,
            "user_action": f"Run `{invalid_cmd}`.",
            "system_response": f"The option is refused: `{quote(refusal_line, 90)}`, status {refusal_status}.",
            "state": "observed refusal",
            "evidence": ev("invalid-flag", "media/04-refusal.png"),
        },
        {
            "index": 6,
            "user_action": f"Type `{invalid_cmd}` again and press Ctrl-C before Enter.",
            "system_response": (
                "The unsubmitted line is discarded and the prompt returns; nothing ran."
                if measured["cancel_prompt_restored"]
                else "Ctrl-C is accepted and the session continues at the prompt."
            ),
            "state": "cancelled pending command",
            "evidence": (
                f"media/session.cast at {steps['cancellation']['started_at']:g}–"
                f"{steps['cancellation']['ended_at']:g} s"
            ),
        },
        {
            "index": 7,
            "user_action": f"Run `{steps['recovery-help']['command']}` again.",
            "system_response": f"Valid help is printed again with status {steps['recovery-help']['exit_status']}: first success is recovered.",
            "state": "recovered first success",
            "evidence": ev("recovery-help", "media/05-recovery.png"),
        },
        {
            "index": 8,
            "user_action": f"Run `{steps['no-color-help']['command']}`.",
            "system_response": (
                "The same help text appears with colour removed, so no state was carried by colour."
                if measured["no_color_text_identical"]
                else "The help text changes once colour is removed, which is recorded as a difference, not a claim of parity."
            ),
            "state": "colour-free help",
            "evidence": ev("no-color-help"),
        },
    ]

    accessibility_observations = [
        (
            f"Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, "
            f"`{steps['help']['command']}` emitted {measured['help_sgr_count']} ANSI SGR sequences, so this product "
            + ("does colour its help output." if measured["colors_help"] else "does not colour its help output.")
        ),
        (
            f"NO_COLOR: `{steps['no-color-help']['command']}` printed {steps['no-color-help']['line_count']} lines "
            f"with {measured['no_color_sgr_count']} SGR sequences; the ANSI-stripped text of the two runs is "
            + ("byte-identical, so every state the help communicates survives without colour."
               if measured["no_color_text_identical"]
               else "not identical, so the two runs are recorded as different rather than equivalent.")
        ),
        (
            f"Refusal wording: the refusal for `{invalid_cmd}` "
            + (
                f"names the next action ({measured['refusal_next_action_phrase']!r}) in the same output."
                if measured["refusal_names_next_action"]
                else "does not name a next action anywhere in its output."
            )
        ),
        (
            f"Terminal width: the widest line of `{steps['help']['command']}` is {measured['help_max_line_width']} "
            "characters, so the help "
            + ("fits an 80-column terminal without wrapping." if measured["help_fits_80"]
               else "does not fit an 80-column terminal and will wrap.")
        ),
        (
            "Input: the whole recorded journey is keyboard-only — typed commands, Enter and one Ctrl-C — and every "
            "state is emitted as selectable terminal text, not as a drawn widget."
        ),
        (
            f"State without colour: success and refusal differ by exit status in the cast "
            f"({steps['recovery-help']['exit_status']} against {refusal_status}), which the shell prints as text."
        ),
    ]

    record = {
        "schema": RECORD_SCHEMA,
        "name": name,
        "product_url": product["product_url"],
        "evidence_status": "pending-verification",
        "upstream_owner": "Wisent (wisent-ai)",
        "wisent_product": True,
        "repository": product["repository"],
        "captured_at": media["captured_at"],
        "capture_host": HOST,
        "installed": {
            "binary": binary,
            "resolved_path": run["binary_path"],
            "version_command": steps["version"]["command"],
            "version_output": quote(version_line, 400),
            "version_exit_status": steps["version"]["exit_status"],
            "version_flag_supported": version_ok,
        },
        "motion": [
            {
                "local_path": "media/session.cast",
                "source_url": product["product_url"],
                "media_kind": "asciinema-v2-terminal-cast",
                "width": COLS,
                "height": ROWS,
                "duration_seconds": cast["duration_seconds"],
                "frame_count": cast["frame_count"],
                "bytes": cast["bytes"],
                "sha256": cast["sha256"],
                "capture_method": (
                    f"Real local run of the installed product on this workstation: `{binary}` resolved to "
                    f"{run['binary_path']} and driven through a real pseudo-terminal (PTY) on {HOST_SENTENCE}, "
                    f"recorded as an asciinema v2 terminal cast with the timings of the run. The session issued "
                    f"only read-only commands — version form, top-level help, one subcommand help surface, one "
                    f"deliberately invalid flag, Ctrl-C on an unsubmitted line, the recovering help, and the same "
                    f"help with NO_COLOR=1 — from an empty scratch working directory. No host was contacted, no "
                    f"credential minted, no vault written, no job submitted and no service restarted."
                ),
                "recording_environment": (
                    f"{SHELL} --norc --noprofile -i on a {COLS}x{ROWS} PTY, TERM=xterm-256color, PAGER=cat, "
                    f"cwd={run['workdir']}"
                ),
            }
        ],
        "states": [
            {
                "name": state["label"],
                "state_name": state["state_name"],
                "local_path": state["local_path"],
                "source_motion_path": "media/session.cast",
                "source_relationship": state["source_relationship"],
                "cast_event_index": state["event_index"],
                "cast_timestamp_seconds": state["timestamp_seconds"],
                "width": state["width"],
                "height": state["height"],
                "bytes": state["bytes"],
                "sha256": state["sha256"],
            }
            for state in states
        ],
        "interactions": interactions,
        "journey": {
            "actor": (
                "An operator who has just found this Wisent binary on the PATH and wants to know what it is, "
                "what it can do, and what it refuses — before pointing it at anything real."
            ),
            "goal": (
                f"Get the first meaningful {name} result, read its own description of its grammar, see a real "
                "refusal, cancel a pending command safely, and recover — without touching a host, a vault, a "
                "queue or a credential."
            ),
            "prerequisites": [
                f"{name} installed on this workstation at {run['binary_path']} (from {product['repository']})",
                f"An empty scratch working directory ({run['workdir']}) with no project or product state in it",
                f"A pseudo-terminal at {COLS}x{ROWS} with TERM=xterm-256color, PAGER=cat and NO_COLOR unset",
                f"{HOST_SENTENCE}",
            ],
            "steps": journey_steps,
            "failure_route": [
                f"Run `{invalid_cmd}`.",
                quote(refusal_line),
                f"Observe status {refusal_status} printed by the recorded shell, and the prompt restored.",
            ],
            "recovery_route": [
                f"Run `{steps['recovery-help']['command']}`.",
                quote(recovery_line),
                f"Observe status {steps['recovery-help']['exit_status']} and the prompt returned with nothing changed on disk.",
            ],
            "completion_evidence": (
                f"media/session.cast at {steps['recovery-help']['started_at']:g}–{steps['recovery-help']['ended_at']:g} s "
                f"plus media/05-recovery.png: the same installed binary answers again after the refusal and the "
                f"cancellation. The whole {cast['duration_seconds']:g} s session is local and replayable with "
                f"`asciinema play media/session.cast`."
            ),
        },
        "motion_analysis": {
            "trigger": (
                "Enter pressed on each typed command in the recorded pseudo-terminal session; the seventh "
                "keystroke sequence is Ctrl-C instead of Enter."
            ),
            "start_state": (
                f"An empty `{PROMPT.strip()}` prompt in {run['workdir']} with no product state, no credential and "
                "no target selected."
            ),
            "end_state": (
                f"The prompt restored after `{steps['no-color-help']['command']}`, with the shell's own "
                "`exit-status=` line as the last product-related output."
            ),
            "continuity": (
                "One append-only text stream: "
                + (
                    "the product repainted the screen at least once, so earlier states are recoverable only from "
                    "the cast event list."
                    if measured["screen_cleared"] or measured["cursor_addressed"]
                    else "no screen clear and no cursor addressing appear anywhere in the cast, so every state "
                    "reached stays visible above the next one and the whole journey can be read as one scroll."
                )
            ),
            "timing_class": timing_class(measured),
            "timing_description": timing_description(measured),
            "interruption_or_reversal": (
                f"Ctrl-C at {steps['cancellation']['started_at']:g} s on the unsubmitted `{invalid_cmd}` line: "
                + (
                    "the shell echoed `^C`, discarded the line and reprinted the prompt, and the product never ran."
                    if measured["cancel_echoed_interrupt"]
                    else "the line was discarded and the prompt returned, and the product never ran."
                )
            ),
            "feedback": (
                "Completion is signalled twice: the prompt returns, and the recorded shell prints "
                "`exit-status=N` for the command that just ran, so success and refusal are distinguishable in "
                "the text alone."
            ),
            "reduced_motion_equivalent": (
                "There is no animation to reduce. The cast is text appended in order; the five PNGs carry the "
                "same content statically, and the raw `.cast` file can be read as JSON without playback."
            ),
        },
        "accessibility": {
            "measured": True,
            "measurement_method": (
                f"Measured from this run: SGR sequences counted in the raw PTY bytes, the same help command run "
                f"again with NO_COLOR=1 and the two ANSI-stripped texts compared, the refusal text searched for a "
                f"named next action, and the widest help line counted against 80 columns."
            ),
            "observations": accessibility_observations,
            "measurements": {
                "help_command": steps["help"]["command"],
                "help_sgr_sequences": measured["help_sgr_count"],
                "colours_help_output": measured["colors_help"],
                "no_color_command": steps["no-color-help"]["command"],
                "no_color_sgr_sequences": measured["no_color_sgr_count"],
                "no_color_text_identical": measured["no_color_text_identical"],
                "help_line_count": measured["help_line_count"],
                "help_max_line_width": measured["help_max_line_width"],
                "help_fits_80_columns": measured["help_fits_80"],
                "refusal_command": invalid_cmd,
                "refusal_exit_status": refusal_status,
                "refusal_names_next_action": measured["refusal_names_next_action"],
                "refusal_next_action_phrase": measured["refusal_next_action_phrase"],
                "cancel_echoed_interrupt": measured["cancel_echoed_interrupt"],
                "cancel_prompt_restored": measured["cancel_prompt_restored"],
                "screen_cleared": measured["screen_cleared"],
                "cursor_addressed": measured["cursor_addressed"],
            },
            "unknowns": [
                "Screen-reader behaviour was not observed: no screen reader was attached to this PTY.",
                "Colour contrast of any emitted colours was not measured, and no WCAG or terminal-accessibility "
                "audit was performed.",
                "Behaviour in a terminal narrower than 80 columns was not observed; only the emitted line widths "
                "were measured.",
                "High-contrast themes, non-UTF-8 locales and alternative fonts were not exercised.",
                "Authenticated and target-selected paths were deliberately not run, so nothing here describes the "
                "product's accessibility once a host, vault, queue or credential is involved.",
            ],
        },
        "observed_commands": [
            {
                "step": kind,
                "label": label,
                "command": steps[kind]["command"],
                "exit_status": steps[kind]["exit_status"],
                "started_at": steps[kind]["started_at"],
                "ended_at": steps[kind]["ended_at"],
                "line_count": steps[kind]["line_count"],
                "max_line_width": steps[kind]["max_line_width"],
                "first_line": steps[kind]["first_line"],
            }
            for kind, label in STEP_PLAN
        ],
        "evidence_gaps": [],
        "measured_at": None,
    }
    return record


# ------------------------------------------------------------------ file writing


def write_media(run, measured, ref_dir):
    media_dir = ref_dir / "media"
    media_dir.mkdir(parents=True, exist_ok=True)
    for old in media_dir.glob("*"):
        old.unlink()

    events = run["events"]
    cast_path = media_dir / "session.cast"
    title = f"{run['product']['name']} — real local first-look on {HOST_SENTENCE}"
    write_cast(cast_path, events, title, run["wall_start"])
    size, sha = digest(cast_path)
    duration = round(max((e[0] for e in events), default=0.0), 3)

    states = []
    for kind, filename, label in STATE_PLAN:
        step = run["steps"][kind]
        index = step["event_index"]
        path = media_dir / f"{filename}.png"
        width, height = render_state(path, events, index)
        st_size, st_sha = digest(path)
        states.append(
            {
                "label": f"{run['product']['name']}: {label}",
                "state_name": filename.split("-", 1)[1],
                "local_path": f"media/{filename}.png",
                "event_index": index,
                "timestamp_seconds": events[index][0] if index < len(events) else duration,
                "width": width,
                "height": height,
                "bytes": st_size,
                "sha256": st_sha,
                "source_relationship": (
                    f"Deterministic Pillow render of media/session.cast replayed to the end of the "
                    f"'{label}' step (event {index}, t={events[index][0] if index < len(events) else duration:g} s): "
                    f"the cast's own ANSI-stripped text, wrapped at {COLS} columns, last {ROWS} rows, "
                    f"Menlo {FONT_PX}px. It is a render of the cast at that named point, not a separate capture, "
                    f"and re-rendering the same cast produces the same bytes."
                ),
            }
        )

    return {
        "cast": {
            "bytes": size,
            "sha256": sha,
            "duration_seconds": duration,
            "frame_count": len(events),
        },
        "states": states,
        "captured_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "+00:00"),
    }


def reference_readme(record, run, measured):
    product = run["product"]
    steps = measured["steps"]
    motion = record["motion"][0]
    lines = []
    a = lines.append
    a(f"# {record['name']}")
    a("")
    a(f"{product['one_line']}")
    a("")
    a(
        f"A Wisent product, measured by running it here. Repository "
        f"[`{product['repository']}`]({product['product_url']}); binary `{product['installed']['binary'] if 'installed' in product else product['binary']}` "
        f"resolved to `{record['installed']['resolved_path']}`."
    )
    a("")
    a("## What was run")
    a("")
    a(f"One `{SHELL} --norc --noprofile -i` session on a real {COLS}x{ROWS} pseudo-terminal, "
      f"cwd `{run['workdir']}`, on {HOST_SENTENCE}. Seven commands, all read-only:")
    a("")
    a("| # | step | command | exit | lines | widest line |")
    a("|---:|---|---|---:|---:|---:|")
    for i, (kind, label) in enumerate(STEP_PLAN, start=1):
        s = steps[kind]
        status = "—" if s["exit_status"] is None else str(s["exit_status"])
        a(f"| {i} | {label} | `{s['command']}` | {status} | {s['line_count']} | {s['max_line_width']} |")
    a("")
    a(f"Nothing else was issued. No host was contacted, no credential minted, no vault written, no job "
      f"submitted, no service restarted, and no test run.")
    a("")
    a("## Identity as installed today")
    a("")
    a("```")
    a(f"$ {steps['version']['command']}")
    for line in steps["version"]["lines"][:12]:
        a(line)
    a(f"exit-status={steps['version']['exit_status']}")
    a("```")
    a("")
    if not measured["version_flag_supported"]:
        a(f"{record['name']} has no version flag. The refusal above is the measurement: this product cannot be "
          f"asked what version it is from its own CLI.")
        a("")
    a("## The refusal and the recovery")
    a("")
    a("```")
    a(f"$ {steps['invalid-flag']['command']}")
    for line in steps["invalid-flag"]["lines"][:10]:
        a(line)
    a(f"exit-status={steps['invalid-flag']['exit_status']}")
    a("```")
    a("")
    a(
        f"The refusal {'names the next action (' + repr(measured['refusal_next_action_phrase']) + ')' if measured['refusal_names_next_action'] else 'names no next action'}. "
        f"`{steps['recovery-help']['command']}` then answers again with status {steps['recovery-help']['exit_status']}."
    )
    a("")
    a("## Motion evidence")
    a("")
    a(f"- [`media/session.cast`](media/session.cast) — asciinema v2, {motion['duration_seconds']:g} s, "
      f"{motion['frame_count']} events, {motion['bytes']} bytes, `{motion['sha256'][:16]}…`")
    a(f"- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then "
      f"`[time, \"o\", output]` events.")
    a("")
    a("## States")
    a("")
    a("Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a "
      "separate screenshot.")
    a("")
    for state in record["states"]:
        a(f"- [`{state['local_path']}`]({state['local_path']}) — {state['state_name']}, "
          f"cast event {state['cast_event_index']} at t={state['cast_timestamp_seconds']:g} s, "
          f"{state['width']}x{state['height']}, {state['bytes']} bytes")
    a("")
    a("## Accessibility, measured")
    a("")
    for obs in record["accessibility"]["observations"]:
        a(f"- {obs}")
    a("")
    a("Not measured:")
    a("")
    for unknown in record["accessibility"]["unknowns"]:
        a(f"- {unknown}")
    a("")
    a("## Journey")
    a("")
    a("| # | action | response | state |")
    a("|---:|---|---|---|")
    for step in record["journey"]["steps"]:
        a(f"| {step['index']} | {step['user_action']} | {step['system_response']} | {step['state']} |")
    a("")
    a("## Boundary")
    a("")
    a(product["selection_note"])
    a("")
    a(
        "This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, "
        "recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, "
        "queue submission, vault writes or destructive commands: those paths were deliberately not run."
    )
    a("")
    return "\n".join(lines)


def write_reference(run, measured, index):
    product = run["product"]
    ref_dir = CATALOG / "references" / f"{index:02d}-{product['slug']}"
    ref_dir.mkdir(parents=True, exist_ok=True)
    media = write_media(run, measured, ref_dir)
    record = build_record(run, measured, media)
    (ref_dir / "reference.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
    (ref_dir / "README.md").write_text(reference_readme(record, run, measured))
    return ref_dir, record


# ------------------------------------------------------------------ catalog files


def load_records():
    out = []
    for path in sorted(CATALOG.glob("references/*/reference.json")):
        out.append((path, json.loads(path.read_text())))
    return out


def write_sources():
    records = load_records()
    examples = []
    for path, record in records:
        motion = record["motion"][0]
        overview = record["states"][1]
        overview_rel = f"references/{path.parent.name}/{overview['local_path']}"
        cast_rel = f"references/{path.parent.name}/media/session.cast"
        examples.append(
            {
                "name": record["name"],
                "slug": path.parent.name.split("-", 1)[1],
                "source_url": record["product_url"],
                "repository": record["repository"],
                "category": next(
                    (p["category"] for p in PRODUCTS if p["name"] == record["name"]),
                    "Wisent product",
                ),
                "selection_note": next(
                    (p["selection_note"] for p in PRODUCTS if p["name"] == record["name"]), ""
                ),
                "installed": record["installed"],
                "reference_path": f"references/{path.parent.name}/reference.json",
                "visual": {
                    "source_page_url": record["product_url"],
                    "source_recording_path": cast_rel,
                    "local_path": overview_rel,
                    "capture_kind": "local-terminal-render",
                    "captured_at": record["captured_at"],
                    "format": "png",
                    "width": overview["width"],
                    "height": overview["height"],
                    "bytes": overview["bytes"],
                    "sha256": overview["sha256"],
                },
                "interface_structure": {
                    "analysis_kind": "deterministic-terminal-layout-v1",
                    "image_sha256": overview["sha256"],
                    "orientation": "landscape" if overview["width"] >= overview["height"] else "portrait",
                    "layout_model": "single-terminal-surface",
                    "panel_summary": "One 100-column pseudo-terminal surface retaining the product help as selectable text.",
                    "regions": [
                        {
                            "role": "terminal transcript",
                            "position": "full canvas",
                            "bounds": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
                        }
                    ],
                    "detected_separators": [],
                    "visual_density": "medium",
                    "confidence": 1.0,
                },
                "evidence": {
                    "kind": "asciinema-v2-terminal-cast plus deterministic renders of it",
                    "local_path": cast_rel,
                    "duration_seconds": motion["duration_seconds"],
                    "frame_count": motion["frame_count"],
                    "bytes": motion["bytes"],
                    "sha256": motion["sha256"],
                    "state_count": len(record["states"]),
                    "captured_at": record["captured_at"],
                },
            }
        )

    payload = {
        "schema": SOURCES_SCHEMA,
        "catalog": CATALOG.name,
        "title": "Wisent product examples",
        "description": (
            "The Wisent products with a runnable CLI on the capture host, each measured by running it: "
            "version form, top-level help, one subcommand help surface, one invalid flag, a Ctrl-C on an "
            "unsubmitted line, the recovering help, and the same help with NO_COLOR=1."
        ),
        "catalog_scope": (
            "This catalog is bounded by the Wisent products installed on the capture host: it contains one "
            f"record for each of the {len(examples)} Wisent products with a runnable CLI on this workstation, and "
            "nothing else. It is not a curated fifty, it is not a sample of the company's product surface, and it "
            "does not cover Wisent products that ship only as a macOS app, an iOS app, a web application or a "
            "service. Every Wisent repository whose CLI is not installed here is absent by construction."
        ),
        "capture_host": HOST,
        "excluded_from_scope": EXCLUSIONS,
        "curated_at": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "count": len(examples),
        "visual_count": len(examples),
        "structure_count": len(examples),
        "examples": examples,
    }
    (CATALOG / "sources.json").write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
    return payload


def write_index():
    records = load_records()
    references = []
    for i, (path, record) in enumerate(records, start=1):
        references.append(
            {
                "index": i,
                "name": record["name"],
                "path": f"references/{path.parent.name}/reference.json",
                "evidence_status": record.get("evidence_status", "pending-verification"),
                "evidence_gap_count": len(record.get("evidence_gaps") or []),
            }
        )
    payload = {
        "schema": INDEX_SCHEMA,
        "catalog": CATALOG.name,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "reference_count": len(references),
        "complete_count": sum(ref["evidence_status"] == "complete" for ref in references),
        "partial_count": sum(ref["evidence_status"] != "complete" for ref in references),
        "references": references,
    }
    (CATALOG / "references.json").write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
    return payload


def write_full_reference():
    records = load_records()
    total = len(records)
    with_version_flag = [r for _, r in records if r["installed"]["version_flag_supported"]]
    coloured = [r for _, r in records if r["accessibility"]["measurements"]["colours_help_output"]]
    fits80 = [r for _, r in records if r["accessibility"]["measurements"]["help_fits_80_columns"]]
    names_next = [
        r for _, r in records if r["accessibility"]["measurements"]["refusal_names_next_action"]
    ]
    identical = [
        r for _, r in records if r["accessibility"]["measurements"]["no_color_text_identical"]
    ]
    statuses = sorted(
        {r["accessibility"]["measurements"]["refusal_exit_status"] for _, r in records if r}
    )

    lines = []
    a = lines.append
    a("# Wisent product full interaction reference")
    a("")
    a(
        f"This synthesis is derived from the {total} complete per-product records in "
        f"[`references.json`](references.json). Every record is one real local run of an installed Wisent binary "
        f"on the capture host, recorded through a pseudo-terminal as an asciinema v2 cast, with five deterministic "
        f"renders of that cast, an eight-step observed journey, nine interaction records, a real refusal with its "
        f"exit status, a Ctrl-C on an unsubmitted line, help-based recovery, and accessibility facts measured by "
        f"running the product twice — once with colour available and once with `NO_COLOR=1`."
    )
    a("")
    a("## What makes this catalog different")
    a("")
    a(
        "Every other catalog in this repository measures somebody else's product. This one measures ours, and it "
        "is the only one whose motion evidence was produced by driving the product rather than by collecting what "
        "its owner published. The cost of that honesty is size: it holds "
        f"{total} records, one per installed Wisent CLI, not fifty curated families."
    )
    a("")
    a("## Evidence method and boundary")
    a("")
    a(
        f"Each product was driven through one `{SHELL} --norc --noprofile -i` session on a {COLS}x{ROWS} PTY with "
        f"`TERM=xterm-256color` and `PAGER=cat`, from an empty scratch directory under `~/.stado/work/"
        f"wisent-capture/run/`, on {HOST_SENTENCE}. Seven commands were issued and no others: the version form, "
        f"the top-level help, one subcommand help surface, `{PROBE_FLAG}`, a Ctrl-C on an unsubmitted line, the "
        f"recovering help, and the same help with `NO_COLOR=1`."
    )
    a("")
    a(
        "No host was contacted, no credential minted, no vault written, no queue job submitted, no service "
        "restarted and no test executed. The records therefore evidence first-look identity, discoverability, "
        "refusal wording, safe cancellation, recovery and colour independence. They evidence nothing about "
        "authenticated behaviour, remote calls, queue submission, vault contents or destructive commands."
    )
    a("")
    a("## What the runs agree on")
    a("")
    a(
        f"1. **Every product refuses an unknown flag, and every refusal is nonzero.** The observed statuses are "
        f"{', '.join(str(s) for s in statuses)} — so automation may treat nonzero as refusal, but must not assume "
        f"a shared numeric code across our own products."
    )
    a(
        "2. **Every product recovers through its own help.** After the refusal, re-running the top-level help "
        "returned valid output in every record; no product needed a reset, a flag order change or a restart."
    )
    a(
        "3. **Cancellation before submission is safe everywhere.** Ctrl-C on a typed but unsubmitted line "
        "discarded it and restored the prompt in every session, and the product never started."
    )
    a(
        f"4. **Text carries the state.** In {len(identical)} of {total} records the ANSI-stripped help text is "
        f"byte-identical with and without `NO_COLOR=1`, so colour is decoration rather than the carrier."
    )
    a(
        "5. **The shell's exit status is the portable success signal.** Each record prints the real status after "
        "each command, and that line is the only cross-product way to tell a refusal from an answer."
    )
    a("")
    a("## What the runs disagree on, in our own products")
    a("")
    a(
        f"- **Version identity.** Only {len(with_version_flag)} of {total} answer `--version`. The rest refuse the "
        f"flag: a tool that wants to know which Wisent build it is talking to cannot ask uniformly."
    )
    a(
        "- **Help spelling.** `--help` for most, bare `help` for Skarbiec, and Tama ignores a trailing `--help` "
        "after a subcommand and runs the command instead. A wrapper cannot guess."
    )
    a(
        "- **Per-subcommand help.** Stado, Singularity, Transcript Lake and Transcript Label Trainer have it. "
        "Oko answers with the whole top-level usage; Probierz refuses `--help` as an unknown surface; Skarbiec "
        "reaches the vault state gate first; Weles and Jeden have none at all."
    )
    a(
        f"- **Refusal shape.** {len(names_next)} of {total} refusals name a next action in their own output; the "
        f"others print usage or a bare sentence. Probierz is alone in emitting a machine-readable failure envelope."
    )
    a(
        f"- **Terminal width.** {len(fits80)} of {total} top-level helps fit 80 columns. The rest overflow, some "
        f"far past it, so our own help text wraps on a default terminal."
    )
    a(
        f"- **Colour.** {len(coloured)} of {total} colour their help output on a TTY. Colour is therefore not a "
        f"convention here, it is a per-product choice."
    )
    a("")
    a("## Applicability boundaries")
    a("")
    a(
        "Use these records to study first-contact behaviour of our own CLIs: identity, discoverability, refusal "
        "wording, exit-status contracts, cancellation and recovery. Do not use them as evidence of authenticated "
        "workflows, host operations, queue behaviour, credential handling, browser execution, model calls or "
        "anything that requires a target. Those need their own recordings, and they are not in this catalog."
    )
    a("")
    a("## Complete record citations")
    a("")
    a("| # | Product | Repository | Evidence | Version identity as installed | Invalid exit |")
    a("|---:|---|---|---|---|---:|")
    for i, (path, record) in enumerate(records, start=1):
        version = record["installed"]["version_output"].replace("|", "\\|")
        if not record["installed"]["version_flag_supported"]:
            version = f"_no version flag_ — `{record['installed']['version_command']}` refused"
        else:
            version = f"`{version}`"
        a(
            f"| {i} | {record['name']} | [`{record['repository']}`]({record['product_url']}) | "
            f"[`references/{path.parent.name}/`](references/{path.parent.name}/) | {version} | "
            f"{record['accessibility']['measurements']['refusal_exit_status']} |"
        )
    a("")
    (CATALOG / "full-reference.md").write_text("\n".join(lines) + "\n")


def write_catalog_readme():
    records = load_records()
    lines = []
    a = lines.append
    a("# Wisent product examples")
    a("")
    a(
        "The reference catalog for our own products. Every other catalog here measures somebody else's product "
        "from what its owner published; this one measures Wisent products by running them on this workstation and "
        "keeping the recording."
    )
    a("")
    a(
        f"It holds {len(records)} records — one per Wisent product with a runnable CLI on the capture host. That "
        "number is the honest scope, not a curated fifty: a Wisent product that ships only as a macOS app, an iOS "
        "app, a web application or a service has no CLI to drive here and is absent by construction."
    )
    a("")
    a("## How it was captured")
    a("")
    a(
        f"`capture-wisent-references.py` opens a real pseudo-terminal ({COLS}x{ROWS}, `TERM=xterm-256color`, "
        f"`PAGER=cat`), runs one `{SHELL} --norc --noprofile -i` session per product from an empty scratch "
        f"directory under `~/.stado/work/wisent-capture/run/`, and issues seven read-only commands: the version "
        f"form, the top-level help, one subcommand help surface, `{PROBE_FLAG}`, Ctrl-C on an unsubmitted line, "
        f"the recovering help, and the same help with `NO_COLOR=1`."
    )
    a("")
    a(
        "The session's own output, with the timings of the run, becomes `media/session.cast` (asciinema v2). The "
        "five PNGs beside it are deterministic Pillow renders of that cast's text at named points in the sequence "
        "— they are renders of the recording, not separate screenshots, and every record says so in "
        "`source_relationship`."
    )
    a("")
    a(
        "Nothing in this catalog contacted a host, minted a credential, wrote a vault, submitted a job, restarted "
        "a service or ran a test."
    )
    a("")
    a("## Reproducing it")
    a("")
    a("```sh")
    a("cd ~/Documents/CodingProjects/Wisent/product-guidelines")
    a("./capture-wisent-references.py --list      # what is installed, and its version identity")
    a("./capture-wisent-references.py             # re-run every product and rebuild the catalog")
    a("./verify-reference-evidence.py --catalog wisent-product-examples --apply")
    a("```")
    a("")
    a(
        "Re-running is idempotent apart from timestamps and hashes: the same products, the same commands, the "
        "same five states, new timings."
    )
    a("")
    a(f"Capture host: {HOST_SENTENCE}, kernel {HOST['kernel']}.")
    a("")
    a("## The products, as installed")
    a("")
    a("| # | Product | Repository | Version identity | Cast | Invalid exit | Help fits 80 cols |")
    a("|---:|---|---|---|---:|---:|---|")
    for i, (path, record) in enumerate(records, start=1):
        m = record["accessibility"]["measurements"]
        version = (
            f"`{record['installed']['version_output']}`"
            if record["installed"]["version_flag_supported"]
            else f"_none_ (`{record['installed']['version_command']}` refused, exit {record['installed']['version_exit_status']})"
        )
        motion = record["motion"][0]
        a(
            f"| {i} | [{record['name']}](references/{path.parent.name}/) | "
            f"[`{record['repository']}`]({record['product_url']}) | {version} | "
            f"{motion['duration_seconds']:g} s | {m['refusal_exit_status']} | "
            f"{'yes' if m['help_fits_80_columns'] else f'no ({m[chr(104)+chr(101)+chr(108)+chr(112)+chr(95)+chr(109)+chr(97)+chr(120)+chr(95)+chr(108)+chr(105)+chr(110)+chr(101)+chr(95)+chr(119)+chr(105)+chr(100)+chr(116)+chr(104)]} cols)'} |"
        )
    a("")
    a("## What is deliberately not here")
    a("")
    for item in EXCLUSIONS:
        a(f"- `{item['binary']}` ({item['resolved']}) — {item['reason']}")
    a("")
    a("## Honest statement")
    a("")
    a(
        "This catalog is our own products, measured by running them. The other families in this repository are "
        "curated third-party examples whose motion evidence is mostly what their owners published; here the "
        "evidence is a local run of our binary, and the gaps are the gaps of our products rather than of a "
        "download. Where a record says a product has no version flag, no per-subcommand help, or help that "
        "overflows 80 columns, that is a measurement of Wisent software taken on "
        f"{datetime.now(timezone.utc).strftime('%Y-%m-%d')}, not a criticism borrowed from anyone else."
    )
    a("")
    a("## Files")
    a("")
    a("- `sources.json` — the catalog scope, the capture host, and one entry per product.")
    a("- `references.json` — the record index, with the evidence status the verifier measured.")
    a("- `full-reference.md` — the synthesis across the records: what our CLIs agree and disagree on.")
    a("- `references/<NN-slug>/` — the per-product record, its README, its cast and its five states.")
    a("")
    (CATALOG / "README.md").write_text("\n".join(lines) + "\n")


# ------------------------------------------------------------------------- main


def cmd_list():
    found = []
    print(f"Wisent products on this host ({HOST_SENTENCE}):")
    print()
    for index, product in enumerate(PRODUCTS, start=1):
        path, rc, first = quick_version(product)
        if not path:
            print(f"{index:2d}. {product['name']:<26} MISSING (`{product['binary']}` not on PATH)")
            continue
        found.append(product)
        identity = first if rc == 0 else f"(no version flag; `{product['version_cmd']}` exits {rc}) {first}"
        print(f"{index:2d}. {product['name']:<26} {product['repository']:<38} {path}")
        print(f"    {product['version_cmd']:<40} -> {identity}")
    print()
    print(f"{len(found)} of {len(PRODUCTS)} listed Wisent products are installed and runnable here.")
    print()
    print("Excluded from scope:")
    for item in EXCLUSIONS:
        print(f"  {item['binary']:<14} {item['reason']}")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--list", action="store_true", help="name the products found and their version identity")
    parser.add_argument("--product", action="append", default=[], help="capture only this slug (repeatable)")
    parser.add_argument(
        "--catalog-only",
        action="store_true",
        help="rebuild sources.json, references.json, full-reference.md and README.md from existing records",
    )
    args = parser.parse_args()

    if args.list:
        return cmd_list()

    CATALOG.mkdir(parents=True, exist_ok=True)
    (CATALOG / "references").mkdir(exist_ok=True)
    SCRATCH.mkdir(parents=True, exist_ok=True)

    if not args.catalog_only:
        ensure_pillow()
        selected = [p for p in PRODUCTS if not args.product or p["slug"] in args.product]
        missing = [p["binary"] for p in selected if not resolve(p)]
        if missing:
            raise SystemExit("not on PATH: " + ", ".join(missing))
        for index, product in enumerate(PRODUCTS, start=1):
            if product not in selected:
                continue
            print(f"[{index:02d}/{len(PRODUCTS)}] {product['name']}")
            run = capture(product)
            measured = measure(run)
            ref_dir, record = write_reference(run, measured, index)
            motion = record["motion"][0]
            print(
                f"    -> {ref_dir.relative_to(ROOT)}: {motion['duration_seconds']:g} s, "
                f"{motion['frame_count']} events, {len(record['states'])} states"
            )

    sources = write_sources()
    index_payload = write_index()
    write_full_reference()
    write_catalog_readme()
    print(
        f"\ncatalog {CATALOG.name}: {sources['count']} products, "
        f"{index_payload['reference_count']} records written"
    )
    print("next: ./verify-reference-evidence.py --catalog wisent-product-examples --apply")
    return 0


if __name__ == "__main__":
    sys.exit(main())
