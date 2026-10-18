"""`formula-shorts`: turn a narration script into a finished Short.

formula-shorts tools                     # tool ids, names and state fields
formula-shorts voices [--provider P]     # voices a provider offers
formula-shorts setup kokoro              # download the local Kokoro model
formula-shorts build script.md [options] # record, render and mux
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from datetime import datetime
from pathlib import Path
from typing import TextIO

from . import audio, outputs, render, storyboard, tts
from .script import Script, ScriptError, estimate_seconds, parse, validate


class Log:
    """Lines to stderr and to the build's `build.log`."""

    def __init__(self) -> None:
        self.file: TextIO | None = None
        self.lines: list[str] = []

    def open(self, path: Path) -> None:
        self.file = path.open("w")
        for line in self.lines:
            self.file.write(line + "\n")

    def __call__(self, line: str) -> None:
        print(line, file=sys.stderr, flush=True)
        self.lines.append(line)
        if self.file:
            self.file.write(line + "\n")
            self.file.flush()


def _apply_overrides(script: Script, a: argparse.Namespace) -> None:
    for key in ("size", "theme", "provider", "voice", "speed", "music", "warmup"):
        value = getattr(a, key, None)
        if value is not None:
            setattr(script, key, value)
    if a.chrome is not None:
        script.chrome = a.chrome


def build(a: argparse.Namespace) -> int:
    log = Log()
    source = Path(a.script)
    try:
        script = parse(source.read_text())
    except (OSError, ScriptError) as e:
        print(f"formula-shorts: {e}", file=sys.stderr)
        return 2
    _apply_overrides(script, a)
    catalog = render.catalog()
    errors = validate(script, catalog)
    if errors:
        print("formula-shorts: the script needs changes:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 2
    tool = next(t for t in catalog if t["id"] == script.tool)

    root = Path(a.out) if a.out else render.repo_root() / "generated"
    out = outputs.folder(root, script.tool, datetime.now())
    log.open(out / "build.log")
    shutil.copy(source, out / "script.md")
    log(f"building {tool['name']} into {out}")
    log(f"{len(script.scenes)} scenes, {script.words} words, {script.characters} characters")

    summary: dict = {"folder": str(out), "tool": script.tool, "dry_run": a.dry_run}
    wavs: list[Path] = []
    if a.dry_run:
        seconds = audio.scene_seconds([estimate_seconds(s) - audio.GAP for s in script.scenes])
    else:
        scenes = out / "scenes"
        scenes.mkdir()
        if script.provider in ("elevenlabs", "auto"):
            log(f"ElevenLabs would use about {script.characters} credits if it is chosen")
        try:
            provider, raw = tts.speak_all(
                [s.spoken for s in script.scenes],
                script.provider,
                script.voice,
                float(script.speed),
                scenes,
                log,
            )
        except tts.TtsError as e:
            log(f"error: {e}")
            return 1
        for f in raw:
            wav = f.with_suffix(".wav") if f.suffix != ".wav" else f
            if f != wav:
                audio.run(audio.normalize_args(f, wav))
                f.unlink()
            wavs.append(wav)
        spoken = [audio.duration(w) for w in wavs]
        seconds = audio.scene_seconds(spoken)
        summary["provider"] = provider.name
        summary["voice"] = script.voice or provider.default_voice

    board = storyboard.build(script, seconds, fps=a.fps, scale=a.scale)
    (out / "storyboard.json").write_text(json.dumps(board, indent=2, ensure_ascii=False) + "\n")
    result = render.render(out / "storyboard.json", out, contact_only=a.dry_run, echo=log)
    summary["warnings"] = result.get("warnings", [])
    summary["seconds"] = result["seconds"]
    summary["contact"] = result["contact"]
    summary["thumbnail"] = result["thumbnail"]

    lead = board["lead_in"]
    starts = storyboard.starts(lead, seconds)
    (out / "captions.srt").write_text(
        outputs.srt([s.caption for s in script.scenes], starts, seconds)
    )
    (out / "post.md").write_text(outputs.post(script, result["seconds"], tool["name"]))

    if not a.dry_run:
        narration = out / "narration.m4a"
        music = Path(script.music) if script.music else None
        if music and not music.is_file():
            log(f"error: music file {music} not found")
            return 1
        audio.run(audio.narration_args(wavs, seconds, lead, narration, music))
        short = out / "short.mp4"
        audio.run(audio.mux_args(Path(result["video"]), narration, short))
        summary["short"] = str(short)
        summary["seconds"] = round(audio.duration(short), 2)

    (out / "build.json").write_text(json.dumps(summary, indent=2) + "\n")
    for w in summary["warnings"]:
        log(f"warning: {w}")
    log(f"done: {summary.get('short') or summary['contact']} ({summary['seconds']:.1f} s)")
    print(json.dumps(summary))
    return 0


def tools_cmd(a: argparse.Namespace) -> int:
    catalog = render.catalog()
    if a.json:
        print(json.dumps(catalog, indent=2))
        return 0
    for t in catalog:
        params = ", ".join(t["state_params"])
        print(f"{t['id']:<24} {t['alias']}  {t['name']}")
        print(f"{'':<24}      state: {params or '-'}")
    return 0


def voices_cmd(a: argparse.Namespace) -> int:
    p = tts.make(a.provider)
    reason = p.unavailable()
    if reason:
        print(f"{p.name}: {reason}", file=sys.stderr)
        return 1
    for v in p.voices():
        print(v)
    return 0


def setup_cmd(a: argparse.Namespace) -> int:
    from .tts import kokoro

    kokoro.download(echo=lambda s: print(s, file=sys.stderr))
    print(f"Kokoro is ready in {kokoro.MODELS}", file=sys.stderr)
    return 0


def parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog="formula-shorts", description=(__doc__ or "").split("\n")[0])
    sub = ap.add_subparsers(dest="command", required=True)

    t = sub.add_parser("tools", help="list the tools a Short can show")
    t.add_argument("--json", action="store_true")
    t.set_defaults(run=tools_cmd)

    v = sub.add_parser("voices", help="list a provider's voices")
    v.add_argument("--provider", choices=tts.ORDER, default="edge")
    v.set_defaults(run=voices_cmd)

    s = sub.add_parser("setup", help="download a local voice model")
    s.add_argument("what", choices=["kokoro"])
    s.set_defaults(run=setup_cmd)

    b = sub.add_parser("build", help="make a Short from a script")
    b.add_argument("script")
    b.add_argument("--size", help="shorts (1080x1920), square, landscape or WxH")
    b.add_argument("--provider", choices=tts.NAMES)
    b.add_argument("--voice")
    b.add_argument("--speed", type=float, help="speaking speed, 1 is normal")
    b.add_argument("--theme", choices=["dark", "light"])
    b.add_argument("--chrome", action=argparse.BooleanOptionalAction, default=None,
                   help="show the workbench viewport instead of the clean stage")  # fmt: skip
    b.add_argument("--music", help="background music file, looped under the voice")
    b.add_argument("--warmup", type=float, help="seconds the tool runs before the first frame")
    b.add_argument("--fps", type=int, default=30)
    b.add_argument("--scale", type=float, default=2.0, help="pixels per layout point")
    b.add_argument("--out", help="folder for builds (default: <repo>/generated)")
    b.add_argument("--dry-run", action="store_true",
                   help="no voice: estimate timings and render only contact.png")  # fmt: skip
    b.set_defaults(run=build)
    return ap


def main(argv: list[str] | None = None) -> int:
    a = parser().parse_args(argv)
    try:
        return a.run(a)
    except (render.RenderError, audio.AudioError, tts.TtsError) as e:
        print(f"formula-shorts: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
