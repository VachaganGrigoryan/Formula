"""Microsoft neural voices through the Edge read-aloud service (`edge-tts`).

Free and needs no account, only the network.
"""

from __future__ import annotations

import asyncio
from pathlib import Path

from .base import Provider, TtsError


def rate(speed: float) -> str:
    """edge-tts's rate string for a speed factor: 1.1 -> "+10%"."""
    return f"{round((speed - 1.0) * 100):+d}%"


VOICE = "en-US-AndrewMultilingualNeural"


class Edge(Provider):
    name = "edge"
    default_voice = VOICE
    extension = "mp3"

    def unavailable(self) -> str | None:
        try:
            import edge_tts  # noqa: F401
        except ImportError:
            return "the edge-tts package is not installed"
        return None

    def synthesize(self, text: str, voice: str | None, speed: float, out: Path) -> None:
        import edge_tts

        async def run() -> None:
            await edge_tts.Communicate(text, voice or VOICE, rate=rate(speed)).save(str(out))

        try:
            asyncio.run(run())
        except Exception as e:  # network, unknown voice, service changes
            raise TtsError(f"edge-tts: {e}") from e
        if not out.exists() or out.stat().st_size == 0:
            raise TtsError("edge-tts returned no audio")

    def voices(self) -> list[str]:
        import edge_tts

        found = asyncio.run(edge_tts.list_voices())
        return sorted(v["ShortName"] for v in found if v["Locale"].startswith("en-"))
