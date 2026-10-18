"""Where a build's files go, and the text files that ship with the video."""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

from .script import Script


def folder(root: Path, tool: str, now: datetime) -> Path:
    """`<root>/<YYYY-MM-DD_HHMM>-<tool>`, with `-2`, `-3`... when taken."""
    base = root / f"{now:%Y-%m-%d_%H%M}-{tool}"
    path, n = base, 2
    while path.exists():
        path = base.with_name(f"{base.name}-{n}")
        n += 1
    path.mkdir(parents=True)
    return path


def _clock(t: float) -> str:
    ms = round(t * 1000)
    h, ms = divmod(ms, 3_600_000)
    m, ms = divmod(ms, 60_000)
    s, ms = divmod(ms, 1000)
    return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"


def srt(captions: list[str], starts: list[float], seconds: list[float]) -> str:
    """One subtitle cue per scene, for platforms that take a captions file."""
    cues = []
    for n, (text, start, length) in enumerate(zip(captions, starts, seconds, strict=True), 1):
        cues.append(f"{n}\n{_clock(start)} --> {_clock(start + length)}\n{text}\n")
    return "\n".join(cues)


def post(script: Script, seconds: float, tool_name: str) -> str:
    """Title, description and hashtags to paste into the upload form."""
    p = script.post or {}
    title = p.get("title") or script.title or tool_name
    description = p.get("description") or " ".join(s.caption for s in script.scenes)
    tags = p.get("hashtags") or []
    hashtags = " ".join(t if str(t).startswith("#") else f"#{t}" for t in tags)
    lines = [
        f"# {title}",
        "",
        description,
        "",
        hashtags,
        "",
        "---",
        f"Tool: {tool_name} (`{script.tool}`)  ·  Length: {seconds:.1f} s",
        "Made with Formula: https://github.com/VachaganGrigoryan/Formula",
    ]
    return "\n".join(lines).replace("\n\n\n", "\n\n") + "\n"
