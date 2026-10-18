"""The narration script: YAML front matter, then one `## Heading` per scene.

```markdown
---
tool: lorenz
title: The butterfly effect
post:
  title: Two paths, one hundred-thousandth apart
  hashtags: [math, chaos]
---

## One tiny difference
state: {"count": 2}
Two paths start one hundred-thousandth apart.

## Then they part
equations: false
say: Then, suddenly, they part.
Then they part.
```

Inside a scene, `state: {json}` sets formula parameters when the scene starts,
`equations: false` hides the equation lines, `restart: true` starts the tool
over (a search or sort that finished runs again), and `say: text` is spoken instead
of the caption (for symbols the voice would misread). Every other line is the
caption, joined with spaces.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from typing import Any

import yaml

SIZES = {
    "shorts": (1080, 1920),
    "square": (1080, 1080),
    "landscape": (1920, 1080),
}

MAX_SCENES = 8
MAX_SCENE_WORDS = 45
MAX_TOTAL_WORDS = 220
WORDS_PER_SECOND = 2.6

_DIRECTIVE = re.compile(r"^(state|equations|restart|say):\s*(.*)$")


class ScriptError(ValueError):
    """A script that cannot be built, with every problem listed."""


@dataclass
class Scene:
    heading: str
    caption: str
    say: str | None = None
    state: dict[str, Any] | None = None
    equations: bool = True
    restart: bool = False

    @property
    def spoken(self) -> str:
        return self.say or self.caption

    @property
    def words(self) -> int:
        return len(self.spoken.split())


@dataclass
class Script:
    tool: str
    scenes: list[Scene]
    title: str | None = None
    kicker: str | None = None
    size: str = "shorts"
    theme: str = "dark"
    chrome: bool = False
    voice: str | None = None
    provider: str = "auto"
    speed: float = 1.0
    music: str | None = None
    warmup: float | None = None
    post: dict[str, Any] = field(default_factory=dict)

    @property
    def words(self) -> int:
        return sum(s.words for s in self.scenes)

    @property
    def characters(self) -> int:
        return sum(len(s.spoken) for s in self.scenes)


def parse_size(size: str) -> tuple[int, int]:
    """`shorts`, `square`, `landscape` or `WxH` (both even)."""
    if size in SIZES:
        return SIZES[size]
    m = re.fullmatch(r"(\d+)x(\d+)", size.strip())
    if not m:
        raise ScriptError(f"size {size!r} is not shorts, square, landscape or WxH")
    w, h = int(m.group(1)), int(m.group(2))
    if w <= 0 or h <= 0 or w % 2 or h % 2:
        raise ScriptError(f"size {size!r} needs positive even numbers")
    return w, h


def _front_matter(text: str) -> tuple[dict[str, Any], str]:
    if not text.startswith("---"):
        raise ScriptError("the script must start with --- front matter naming the tool")
    parts = text.split("\n---", 1)
    if len(parts) != 2:
        raise ScriptError("the front matter has no closing ---")
    head = parts[0].removeprefix("---")
    body = parts[1].split("\n", 1)[1] if "\n" in parts[1] else ""
    meta = yaml.safe_load(head) or {}
    if not isinstance(meta, dict):
        raise ScriptError("the front matter must be key: value lines")
    return meta, body


def _scene(heading: str, lines: list[str]) -> Scene:
    caption: list[str] = []
    scene = Scene(heading=heading, caption="")
    for raw in lines:
        line = raw.strip()
        if not line or line.startswith("<!--"):
            continue
        m = _DIRECTIVE.match(line)
        if not m:
            caption.append(line)
            continue
        key, value = m.groups()
        if key == "state":
            try:
                state = json.loads(value)
            except json.JSONDecodeError as e:
                raise ScriptError(f"scene {heading!r}: state is not JSON: {e}") from e
            if not isinstance(state, dict):
                raise ScriptError(f"scene {heading!r}: state must be a JSON object")
            scene.state = state
        elif key in ("equations", "restart"):
            if value.strip().lower() not in ("true", "false"):
                raise ScriptError(f"scene {heading!r}: {key} must be true or false")
            setattr(scene, key, value.strip().lower() == "true")
        else:
            scene.say = value.strip() or None
    scene.caption = " ".join(caption)
    return scene


def parse(text: str) -> Script:
    meta, body = _front_matter(text)
    scenes: list[Scene] = []
    heading: str | None = None
    lines: list[str] = []
    for line in body.splitlines():
        if line.startswith("## "):
            if heading is not None:
                scenes.append(_scene(heading, lines))
            heading, lines = line[3:].strip(), []
        elif heading is not None:
            lines.append(line)
    if heading is not None:
        scenes.append(_scene(heading, lines))
    if "tool" not in meta:
        raise ScriptError("the front matter needs `tool: <id>`")
    known = {f.name for f in Script.__dataclass_fields__.values()} - {"scenes"}
    unknown = sorted(set(meta) - known)
    if unknown:
        raise ScriptError(f"unknown front matter keys: {', '.join(unknown)}")
    return Script(scenes=scenes, **{k: v for k, v in meta.items()})


def validate(script: Script, catalog: list[dict[str, Any]]) -> list[str]:
    """Every problem with `script`, as sentences. Empty when it can be built."""
    errors: list[str] = []
    tool = next((t for t in catalog if t["id"] == script.tool), None)
    if tool is None:
        ids = ", ".join(t["id"] for t in catalog)
        errors.append(f"unknown tool {script.tool!r}; known: {ids}")
    try:
        parse_size(str(script.size))
    except ScriptError as e:
        errors.append(str(e))
    if script.theme not in ("dark", "light"):
        errors.append(f"theme must be dark or light, not {script.theme!r}")
    if not 0.5 <= float(script.speed) <= 2.0:
        errors.append(f"speed must be between 0.5 and 2, not {script.speed}")
    if not script.scenes:
        errors.append("the script has no `## ` scenes")
    if len(script.scenes) > MAX_SCENES:
        errors.append(f"{len(script.scenes)} scenes; keep it to {MAX_SCENES} or fewer")
    for n, s in enumerate(script.scenes, 1):
        if not s.caption:
            errors.append(f"scene {n} ({s.heading!r}) has no caption text")
        if s.words > MAX_SCENE_WORDS:
            errors.append(f"scene {n} has {s.words} words; keep each under {MAX_SCENE_WORDS}")
        if s.state and tool is not None:
            bad = sorted(set(s.state) - set(tool["state_params"]))
            if bad:
                allowed = ", ".join(tool["state_params"]) or "none"
                names = ", ".join(bad)
                errors.append(
                    f"scene {n} sets unknown state {names}; {script.tool} accepts {allowed}"
                )
    if script.words > MAX_TOTAL_WORDS:
        errors.append(f"{script.words} words in total; keep it under {MAX_TOTAL_WORDS} (~80 s)")
    return errors


def estimate_seconds(scene: Scene) -> float:
    """How long a scene's narration takes, before it is recorded."""
    return round(scene.words / WORDS_PER_SECOND + 0.6, 2)
