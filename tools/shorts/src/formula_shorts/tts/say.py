"""macOS's built-in `say`: offline and instant, robotic. The last resort."""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

from .base import Provider, TtsError

#: Words per minute at speed 1.
WPM = 180


class Say(Provider):
    name = "say"
    default_voice = "Samantha"
    extension = "aiff"

    def unavailable(self) -> str | None:
        return None if shutil.which("say") else "`say` exists only on macOS"

    def synthesize(self, text: str, voice: str | None, speed: float, out: Path) -> None:
        voice = voice or self.default_voice
        args = ["say", "-r", str(round(WPM * speed)), "-o", str(out)]
        if voice in self.voices():
            args[1:1] = ["-v", voice]
        r = subprocess.run([*args, text], capture_output=True, text=True)
        if r.returncode != 0 or not out.exists():
            raise TtsError(f"say: {r.stderr.strip() or r.returncode}")

    def voices(self) -> list[str]:
        r = subprocess.run(["say", "-v", "?"], capture_output=True, text=True)
        return [m.group(1).strip() for m in re.finditer(r"^(.+?)\s{2,}\w\w_", r.stdout, re.M)]
