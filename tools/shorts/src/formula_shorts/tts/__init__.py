"""Voice providers, and how `auto` picks one."""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path

from .base import Provider, TtsError
from .edge import Edge
from .elevenlabs import ElevenLabs
from .kokoro import Kokoro
from .say import Say

#: Tried in this order by `auto`: the best voice that is set up wins.
ORDER = ("elevenlabs", "kokoro", "edge", "say")
NAMES = (*ORDER, "auto")


def make(name: str) -> Provider:
    return {"elevenlabs": ElevenLabs, "kokoro": Kokoro, "edge": Edge, "say": Say}[name]()


def candidates(name: str, factory: Callable[[str], Provider] = make) -> list[Provider]:
    """The providers to try for `name`: just that one, or for `auto` every
    available one in `ORDER`."""
    if name != "auto":
        if name not in ORDER:
            raise TtsError(f"unknown provider {name!r}; choose from {', '.join(NAMES)}")
        return [factory(name)]
    return [p for p in map(factory, ORDER) if p.unavailable() is None]


def speak_all(
    texts: list[str],
    provider: str,
    voice: str | None,
    speed: float,
    out_dir: Path,
    log: Callable[[str], None] = print,
    factory: Callable[[str], Provider] = make,
) -> tuple[Provider, list[Path]]:
    """Record every text with one provider. The first scene decides which:
    under `auto`, a provider that fails on it is skipped (with the reason
    logged) and the next one tries, so every scene has the same voice."""
    tried = candidates(provider, factory)
    if not tried:
        raise TtsError("no voice provider is available")
    for p in tried:
        reason = p.unavailable()
        if reason:
            raise TtsError(f"{p.name}: {reason}")
        # The voice belongs to the provider asked for; others use their own.
        v = voice if provider != "auto" or p is tried[0] else None
        first = out_dir / f"01.{p.extension}"
        try:
            log(f"voice: {p.name} ({v or p.default_voice or 'default'})")
            p.synthesize(texts[0], v, speed, first)
        except TtsError as e:
            if provider != "auto":
                raise
            log(f"  {p.name} failed: {e}; trying the next provider")
            continue
        files = [first]
        for n, text in enumerate(texts[1:], 2):
            out = out_dir / f"{n:02d}.{p.extension}"
            p.synthesize(text, v, speed, out)
            files.append(out)
        return p, files
    raise TtsError("every voice provider failed; see the log above")


__all__ = ["NAMES", "ORDER", "Provider", "TtsError", "candidates", "make", "speak_all"]
