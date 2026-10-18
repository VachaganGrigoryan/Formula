"""Audio with ffmpeg: scene lengths, the narration track, and the final mux.

The functions named `*_args` only build command lines, so tests can check
them without running ffmpeg.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

#: Silence after each scene's narration, and extra hold after the last one.
GAP = 0.35
TAIL = 0.6
SAMPLE_RATE = 48000
LOUDNESS = "loudnorm=I=-16:TP=-1.5:LRA=11"
MUSIC_DB = -18
MUSIC_FADE = 1.5


class AudioError(RuntimeError):
    pass


def run(args: list[str]) -> None:
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode != 0:
        raise AudioError(f"{args[0]} failed: {r.stderr.strip()[-600:]}")


def duration(path: Path) -> float:
    r = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
        capture_output=True,
        text=True,
    )
    try:
        return float(r.stdout.strip())
    except ValueError as e:
        raise AudioError(f"could not read the length of {path}: {r.stderr.strip()}") from e


def normalize_args(src: Path, dst: Path) -> list[str]:
    """Any audio file to 48 kHz mono WAV."""
    return [
        "ffmpeg", "-y", "-loglevel", "error", "-i", str(src),
        "-ar", str(SAMPLE_RATE), "-ac", "1", str(dst),
    ]  # fmt: skip


def scene_seconds(spoken: list[float]) -> list[float]:
    """Each scene's length on screen: its narration plus a pause, and a
    longer hold after the last one."""
    out = [round(d + GAP, 3) for d in spoken]
    if out:
        out[-1] = round(out[-1] + TAIL, 3)
    return out


def narration_args(
    wavs: list[Path],
    seconds: list[float],
    lead_in: float,
    out: Path,
    music: Path | None = None,
) -> list[str]:
    """One track: lead-in silence, each scene padded to its exact length,
    loudness-normalized, optionally over looped music that fades out."""
    total = lead_in + sum(seconds)
    args = ["ffmpeg", "-y", "-loglevel", "error"]
    for w in wavs:
        args += ["-i", str(w)]
    if music:
        args += ["-stream_loop", "-1", "-i", str(music)]
    parts = [f"[{i}:a]apad=whole_dur={s:.3f},atrim=0:{s:.3f}[s{i}]" for i, s in enumerate(seconds)]
    joined = "".join(f"[s{i}]" for i in range(len(wavs)))
    delay = round(lead_in * 1000)
    parts.append(
        f"{joined}concat=n={len(wavs)}:v=0:a=1,adelay={delay}:all=1,{LOUDNESS},"
        f"aresample={SAMPLE_RATE}[voice]"
    )
    if music:
        m = len(wavs)
        fade_at = max(0.0, total - MUSIC_FADE)
        parts.append(
            f"[{m}:a]aresample={SAMPLE_RATE},volume={MUSIC_DB}dB,atrim=0:{total:.3f},"
            f"afade=t=out:st={fade_at:.3f}:d={MUSIC_FADE}[bed]"
        )
        parts.append("[voice][bed]amix=inputs=2:duration=first:normalize=0[out]")
        label = "[out]"
    else:
        label = "[voice]"
    args += ["-filter_complex", ";".join(parts), "-map", label]
    args += ["-t", f"{total:.3f}", "-c:a", "aac", "-b:a", "160k", "-ar", str(SAMPLE_RATE), str(out)]
    return args


def mux_args(video: Path, audio: Path, out: Path) -> list[str]:
    """The silent video and the narration into the finished Short."""
    return [
        "ffmpeg", "-y", "-loglevel", "error", "-i", str(video), "-i", str(audio),
        "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy", "-c:a", "copy",
        "-shortest", "-movflags", "+faststart", str(out),
    ]  # fmt: skip
