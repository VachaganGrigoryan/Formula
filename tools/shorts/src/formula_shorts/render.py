"""Running `formula-render` (tools/capture), building it first if needed."""

from __future__ import annotations

import json
import os
import subprocess
from functools import cache
from pathlib import Path
from typing import Any


class RenderError(RuntimeError):
    pass


def repo_root() -> Path:
    """The Formula checkout: `FORMULA_ROOT`, else the folder above this
    package that holds `tools/capture/Cargo.toml`."""
    env = os.environ.get("FORMULA_ROOT")
    if env:
        return Path(env).resolve()
    for start in (Path.cwd(), Path(__file__).resolve()):
        for p in (start, *start.parents):
            if (p / "tools" / "capture" / "Cargo.toml").is_file():
                return p
    raise RenderError("cannot find the Formula repository; set FORMULA_ROOT")


@cache
def binary() -> Path:
    """`FORMULA_RENDER`, or a release build of formula-render (cargo keeps it
    up to date, so this is quick after the first time)."""
    env = os.environ.get("FORMULA_RENDER")
    if env:
        return Path(env)
    root = repo_root()
    manifest = root / "tools" / "capture" / "Cargo.toml"
    r = subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            str(manifest),
            "--bin",
            "formula-render",
        ],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise RenderError(f"building formula-render failed:\n{r.stderr[-2000:]}")
    return root / "tools" / "capture" / "target" / "release" / "formula-render"


@cache
def catalog() -> list[dict[str, Any]]:
    r = subprocess.run([str(binary()), "tools"], capture_output=True, text=True)
    if r.returncode != 0:
        raise RenderError(f"formula-render tools: {r.stderr.strip()}")
    return json.loads(r.stdout)


def render(storyboard: Path, out: Path, contact_only: bool = False, echo=print) -> dict[str, Any]:
    """Draw the storyboard; returns formula-render's JSON summary."""
    args = [str(binary()), str(storyboard), "--out", str(out)]
    if contact_only:
        args.append("--contact-only")
    proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    assert proc.stderr is not None and proc.stdout is not None
    for line in proc.stderr:
        echo(line.rstrip())
    stdout = proc.stdout.read()
    if proc.wait() != 0:
        raise RenderError("formula-render failed; see the log above")
    return json.loads(stdout.strip().splitlines()[-1])
