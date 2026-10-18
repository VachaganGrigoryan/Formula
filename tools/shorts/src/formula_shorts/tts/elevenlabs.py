"""ElevenLabs text to speech over its REST API.

Needs `ELEVENLABS_API_KEY`. The free plan includes about 10,000 characters a
month, roughly ten Shorts. A voice may be given by name or by voice id.
"""

from __future__ import annotations

import os
import re
from pathlib import Path

import httpx

from .base import Provider, TtsError

API = "https://api.elevenlabs.io/v1"
MODEL = "eleven_multilingual_v2"
#: Premade voices tried in order when none is named.
PREFERRED = ("Brian", "Adam", "Rachel")


def _key() -> str | None:
    return os.environ.get("ELEVENLABS_API_KEY") or None


def looks_like_id(voice: str) -> bool:
    return re.fullmatch(r"[A-Za-z0-9]{20}", voice) is not None


def pick(voices: list[dict], wanted: str | None) -> str:
    """The voice id for `wanted` (a name, case-insensitive), or a default."""
    by_name = {v["name"].split(" - ")[0].strip().lower(): v["voice_id"] for v in voices}
    if wanted:
        if wanted.lower() in by_name:
            return by_name[wanted.lower()]
        raise TtsError(f"no ElevenLabs voice named {wanted!r}; try `formula-shorts voices`")
    for name in PREFERRED:
        if name.lower() in by_name:
            return by_name[name.lower()]
    if not voices:
        raise TtsError("the ElevenLabs account has no voices")
    return voices[0]["voice_id"]


class ElevenLabs(Provider):
    name = "elevenlabs"
    default_voice = None
    extension = "mp3"

    def __init__(self, client: httpx.Client | None = None) -> None:
        self._client = client
        self._voices: list[dict] | None = None

    def _http(self) -> httpx.Client:
        if self._client is None:
            self._client = httpx.Client(timeout=60, headers={"xi-api-key": _key() or ""})
        return self._client

    def unavailable(self) -> str | None:
        return None if _key() else "ELEVENLABS_API_KEY is not set"

    def _list(self) -> list[dict]:
        voices = self._voices
        if voices is None:
            r = self._http().get(f"{API}/voices")
            if r.status_code != 200:
                raise TtsError(f"ElevenLabs voices: HTTP {r.status_code} {r.text[:200]}")
            voices = self._voices = list(r.json().get("voices", []))
        return voices

    def synthesize(self, text: str, voice: str | None, speed: float, out: Path) -> None:
        if self.unavailable():
            raise TtsError(self.unavailable())
        voice_id = voice if voice and looks_like_id(voice) else pick(self._list(), voice)
        body = {
            "text": text,
            "model_id": MODEL,
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75,
                "speed": max(0.7, min(1.2, speed)),
            },
        }
        r = self._http().post(
            f"{API}/text-to-speech/{voice_id}",
            params={"output_format": "mp3_44100_128"},
            json=body,
        )
        if r.status_code != 200:
            raise TtsError(f"ElevenLabs: HTTP {r.status_code} {r.text[:200]}")
        out.write_bytes(r.content)

    def voices(self) -> list[str]:
        return sorted(v["name"] for v in self._list())
