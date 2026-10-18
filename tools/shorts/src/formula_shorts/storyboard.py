"""The storyboard `formula-render` draws (see tools/capture/src/shorts)."""

from __future__ import annotations

from typing import Any

from .script import Script, parse_size

LEAD_IN = 0.5


def build(
    script: Script,
    seconds: list[float],
    *,
    fps: int = 30,
    scale: float = 2.0,
    lead_in: float = LEAD_IN,
) -> dict[str, Any]:
    width, height = parse_size(str(script.size))
    board: dict[str, Any] = {
        "tool": script.tool,
        "width": width,
        "height": height,
        "fps": fps,
        "scale": scale,
        "theme": script.theme,
        "chrome": bool(script.chrome),
        "lead_in": lead_in,
        "scenes": [],
    }
    for key in ("title", "kicker", "warmup"):
        value = getattr(script, key)
        if value is not None:
            board[key] = value
    for scene, s in zip(script.scenes, seconds, strict=True):
        entry: dict[str, Any] = {
            "heading": scene.heading,
            "caption": scene.caption,
            "seconds": s,
            "equations": scene.equations,
        }
        if scene.state:
            entry["state"] = scene.state
        if scene.restart:
            entry["restart"] = True
        board["scenes"].append(entry)
    return board


def starts(lead_in: float, seconds: list[float]) -> list[float]:
    out, t = [], lead_in
    for s in seconds:
        out.append(round(t, 3))
        t += s
    return out
