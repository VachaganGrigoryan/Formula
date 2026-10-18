import pytest

from formula_shorts.script import (
    ScriptError,
    estimate_seconds,
    parse,
    parse_size,
    validate,
)

CATALOG = [
    {"id": "lorenz", "name": "Lorenz attractor", "state_params": ["count", "rho", "sigma"]},
    {"id": "a-star", "name": "A* search", "state_params": ["mud", "speed"]},
]

SCRIPT = """---
tool: lorenz
title: The butterfly effect
theme: light
post:
  title: Chaos in one minute
  hashtags: [math, chaos]
---
Notes before the first scene are ignored.

## One tiny difference
state: {"count": 2}
Two paths start one hundred-thousandth apart.
For a while, you cannot tell them apart.

## Then they part
equations: false
say: Then, suddenly, they part.
Then they part.
"""


def test_parses_front_matter_and_scenes():
    s = parse(SCRIPT)
    assert s.tool == "lorenz"
    assert s.title == "The butterfly effect"
    assert s.theme == "light"
    assert s.post["hashtags"] == ["math", "chaos"]
    assert [x.heading for x in s.scenes] == ["One tiny difference", "Then they part"]
    first, second = s.scenes
    assert first.state == {"count": 2}
    assert first.caption == (
        "Two paths start one hundred-thousandth apart. For a while, you cannot tell them apart."
    )
    assert first.equations and first.say is None
    assert not second.equations
    assert second.caption == "Then they part."
    assert second.spoken == "Then, suddenly, they part."
    assert s.words == first.words + 4
    assert validate(s, CATALOG) == []


def test_defaults():
    s = parse("---\ntool: lorenz\n---\n## A\nHello there.\n")
    assert (s.size, s.theme, s.chrome, s.provider, s.speed) == (
        "shorts",
        "dark",
        False,
        "auto",
        1.0,
    )
    assert s.voice is None and s.music is None and s.warmup is None


@pytest.mark.parametrize(
    "text, message",
    [
        ("## A\nhi", "front matter"),
        ("---\ntool: lorenz\n## A\nhi", "closing ---"),
        ("---\ntitle: x\n---\n## A\nhi", "tool: <id>"),
        ("---\ntool: lorenz\nvoyce: x\n---\n## A\nhi", "unknown front matter keys: voyce"),
        ("---\ntool: lorenz\n---\n## A\nstate: {oops}\nhi", "not JSON"),
        ("---\ntool: lorenz\n---\n## A\nstate: [1]\nhi", "JSON object"),
        ("---\ntool: lorenz\n---\n## A\nequations: maybe\nhi", "equations must be true or false"),
        ("---\ntool: lorenz\n---\n## A\nrestart: yes\nhi", "restart must be true or false"),
    ],
)
def test_parse_errors(text, message):
    with pytest.raises(ScriptError, match=message):
        parse(text)


def test_validation_lists_every_problem():
    long = " ".join(["word"] * 50)
    text = (
        "---\ntool: lorenz\nsize: 1081x1920\ntheme: blue\nspeed: 3\n---\n"
        f'## A\nstate: {{"rho": 1, "bogus": 2}}\n{long}\n## B\nequations: false\n'
    )
    errors = validate(parse(text), CATALOG)
    joined = "\n".join(errors)
    assert "even numbers" in joined
    assert "theme must be dark or light" in joined
    assert "speed must be between" in joined
    assert "scene 1 has 50 words" in joined
    assert "unknown state bogus; lorenz accepts count, rho, sigma" in joined
    assert "scene 2 ('B') has no caption" in joined


def test_unknown_tool_and_empty_script():
    errors = validate(parse("---\ntool: nope\n---\n"), CATALOG)
    assert any("unknown tool 'nope'; known: lorenz, a-star" in e for e in errors)
    assert any("no `## ` scenes" in e for e in errors)


def test_too_many_scenes_and_words():
    scenes = "".join(f"## S{i}\n{' '.join(['w'] * 30)}\n" for i in range(9))
    errors = validate(parse(f"---\ntool: lorenz\n---\n{scenes}"), CATALOG)
    assert any("9 scenes" in e for e in errors)
    assert any("270 words in total" in e for e in errors)


def test_sizes():
    assert parse_size("shorts") == (1080, 1920)
    assert parse_size("square") == (1080, 1080)
    assert parse_size("landscape") == (1920, 1080)
    assert parse_size("720x1280") == (720, 1280)
    for bad in ("big", "0x100", "101x200"):
        with pytest.raises(ScriptError):
            parse_size(bad)


def test_estimate_seconds_counts_spoken_words():
    s = parse(SCRIPT).scenes[1]
    assert estimate_seconds(s) == round(4 / 2.6 + 0.6, 2)
