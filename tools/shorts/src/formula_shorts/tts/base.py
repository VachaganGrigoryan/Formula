"""What every voice provider does."""

from __future__ import annotations

from abc import ABC, abstractmethod
from pathlib import Path


class TtsError(RuntimeError):
    """A provider could not speak (no key, no network, no model)."""


class Provider(ABC):
    #: Short name used on the command line.
    name: str
    #: Voice used when the script and the command line name none.
    default_voice: str | None
    #: Extension of the files `synthesize` writes.
    extension: str

    @abstractmethod
    def unavailable(self) -> str | None:
        """Why the provider cannot run here, or None when it can."""

    @abstractmethod
    def synthesize(self, text: str, voice: str | None, speed: float, out: Path) -> None:
        """Speak `text` into `out` (which has `extension`). Raise TtsError."""

    @abstractmethod
    def voices(self) -> list[str]:
        """Voice names this provider accepts."""
