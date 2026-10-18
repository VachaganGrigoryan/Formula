from datetime import datetime
from pathlib import Path

from formula_shorts import audio, outputs, storyboard
from formula_shorts.script import parse

SCRIPT = parse(
    """---
tool: lorenz
size: square
chrome: true
warmup: 0
post:
  hashtags: [math, "#chaos"]
---
## One
state: {"rho": 15}
First caption.
## Two
equations: false
restart: true
Second caption.
"""
)


def test_scene_seconds_add_a_gap_and_a_tail():
    assert audio.scene_seconds([2.0, 3.0]) == [2.35, 3.95]
    assert audio.scene_seconds([]) == []


def test_storyboard_carries_the_script():
    b = storyboard.build(SCRIPT, [2.35, 3.95], fps=24, scale=1.5)
    assert (b["width"], b["height"], b["fps"], b["scale"]) == (1080, 1080, 24, 1.5)
    assert b["chrome"] is True and b["theme"] == "dark"
    assert b["warmup"] == 0
    assert "title" not in b and "kicker" not in b
    assert b["scenes"][0] == {
        "heading": "One",
        "caption": "First caption.",
        "seconds": 2.35,
        "equations": True,
        "state": {"rho": 15},
    }
    assert "state" not in b["scenes"][1] and b["scenes"][1]["equations"] is False
    assert b["scenes"][1]["restart"] is True and "restart" not in b["scenes"][0]
    assert storyboard.starts(0.5, [2.35, 3.95]) == [0.5, 2.85]


def test_narration_pads_each_scene_and_delays_the_lead_in():
    args = audio.narration_args([Path("a.wav"), Path("b.wav")], [2.35, 3.95], 0.5, Path("n.m4a"))
    graph = args[args.index("-filter_complex") + 1]
    assert "[0:a]apad=whole_dur=2.350,atrim=0:2.350[s0]" in graph
    assert "[1:a]apad=whole_dur=3.950,atrim=0:3.950[s1]" in graph
    assert "[s0][s1]concat=n=2:v=0:a=1,adelay=500:all=1,loudnorm" in graph
    assert args[args.index("-map") + 1] == "[voice]"
    assert args[args.index("-t") + 1] == "6.800"
    assert "-stream_loop" not in args


def test_music_is_looped_quiet_and_faded():
    args = audio.narration_args([Path("a.wav")], [4.0], 0.5, Path("n.m4a"), Path("m.mp3"))
    assert args[args.index("-stream_loop") + 2 : args.index("-stream_loop") + 4] == ["-i", "m.mp3"]
    graph = args[args.index("-filter_complex") + 1]
    assert (
        "[1:a]aresample=48000,volume=-18dB,atrim=0:4.500,afade=t=out:st=3.000:d=1.5[bed]" in graph
    )
    assert "[voice][bed]amix=inputs=2:duration=first:normalize=0[out]" in graph
    assert args[args.index("-map") + 1] == "[out]"


def test_mux_copies_both_streams():
    args = audio.mux_args(Path("v.mp4"), Path("n.m4a"), Path("s.mp4"))
    assert args[-1] == "s.mp4"
    assert "-shortest" in args and args.count("copy") == 2


def test_folder_names_by_date_and_tool(tmp_path: Path):
    now = datetime(2026, 10, 18, 21, 5)
    a = outputs.folder(tmp_path, "lorenz", now)
    b = outputs.folder(tmp_path, "lorenz", now)
    assert a.name == "2026-10-18_2105-lorenz"
    assert b.name == "2026-10-18_2105-lorenz-2"
    assert a.is_dir() and b.is_dir()


def test_srt():
    text = outputs.srt(["Hello.", "World."], [0.5, 2.85], [2.35, 63.95])
    assert text == (
        "1\n00:00:00,500 --> 00:00:02,850\nHello.\n\n2\n00:00:02,850 --> 00:01:06,800\nWorld.\n"
    )


def test_post_uses_the_front_matter_or_falls_back():
    text = outputs.post(SCRIPT, 6.8, "Lorenz attractor")
    assert text.startswith("# Lorenz attractor\n\nFirst caption. Second caption.\n\n#math #chaos\n")
    assert "Length: 6.8 s" in text and "`lorenz`" in text
