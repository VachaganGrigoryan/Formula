"""Kokoro, a small open text-to-speech model that runs locally (kokoro-onnx).

The model (about 330 MB) is downloaded once by `formula-shorts setup kokoro`
into `tools/shorts/models/`, which git ignores.
"""

from __future__ import annotations

from pathlib import Path

import httpx

from .base import Provider, TtsError

RELEASE = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.1"
#: File name and the smallest size a complete download can have.
FILES = {"kokoro-v1.0.onnx": 300_000_000, "voices-v1.0.bin": 20_000_000}
MODELS = Path(__file__).resolve().parents[3] / "models"


def missing(models: Path = MODELS) -> list[str]:
    return [
        name
        for name, size in FILES.items()
        if not (models / name).is_file() or (models / name).stat().st_size < size
    ]


def download(models: Path = MODELS, echo=print) -> None:
    """Fetch the model files that are missing or incomplete."""
    models.mkdir(parents=True, exist_ok=True)
    for name in missing(models):
        url = f"{RELEASE}/{name}"
        part = models / f"{name}.part"
        echo(f"downloading {url}")
        with httpx.stream("GET", url, follow_redirects=True, timeout=None) as r:
            r.raise_for_status()
            total = int(r.headers.get("content-length", 0))
            done = 0
            with part.open("wb") as f:
                for chunk in r.iter_bytes(1 << 20):
                    f.write(chunk)
                    done += len(chunk)
                    if total and done % (32 << 20) < (1 << 20):
                        echo(f"  {done >> 20} of {total >> 20} MB")
        part.rename(models / name)
    still = missing(models)
    if still:
        raise TtsError(f"download incomplete: {', '.join(still)}")


class Kokoro(Provider):
    name = "kokoro"
    default_voice = "af_heart"
    extension = "wav"

    def __init__(self, models: Path = MODELS) -> None:
        self.models = models
        self._engine = None

    def unavailable(self) -> str | None:
        gone = missing(self.models)
        if gone:
            return "the model is not downloaded; run `formula-shorts setup kokoro`"
        return None

    def _load(self):
        if self._engine is None:
            from kokoro_onnx import Kokoro as Engine

            self._engine = Engine(
                str(self.models / "kokoro-v1.0.onnx"), str(self.models / "voices-v1.0.bin")
            )
        return self._engine

    def synthesize(self, text: str, voice: str | None, speed: float, out: Path) -> None:
        if self.unavailable():
            raise TtsError(self.unavailable())
        import soundfile

        try:
            samples, rate = self._load().create(
                text, voice=voice or self.default_voice, speed=speed, lang="en-us"
            )
        except Exception as e:  # unknown voice, phonemizer problems
            raise TtsError(f"kokoro: {e}") from e
        soundfile.write(str(out), samples, rate)

    def voices(self) -> list[str]:
        if self.unavailable():
            raise TtsError(self.unavailable())
        return sorted(self._load().get_voices())
