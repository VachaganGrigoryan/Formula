from pathlib import Path

import pytest

from formula_shorts import tts
from formula_shorts.tts.base import Provider, TtsError
from formula_shorts.tts.edge import rate
from formula_shorts.tts.elevenlabs import ElevenLabs, looks_like_id, pick
from formula_shorts.tts.kokoro import Kokoro, missing


class Fake(Provider):
    extension = "wav"
    default_voice = "default"

    def __init__(self, name, ready=True, fails=False):
        self.name = name
        self.ready = ready
        self.fails = fails
        self.calls = []

    def unavailable(self):
        return None if self.ready else "not set up"

    def synthesize(self, text, voice, speed, out):
        self.calls.append((text, voice, speed))
        if self.fails:
            raise TtsError("boom")
        out.write_bytes(b"x")

    def voices(self):
        return ["default"]


def factory(fakes):
    return lambda name: fakes[name]


def fakes(**overrides):
    base = {n: Fake(n) for n in tts.ORDER}
    for name, kw in overrides.items():
        base[name] = Fake(name, **kw)
    return base


def test_auto_prefers_the_best_ready_provider():
    f = fakes()
    assert [p.name for p in tts.candidates("auto", factory(f))] == list(tts.ORDER)
    f = fakes(elevenlabs={"ready": False}, kokoro={"ready": False})
    assert [p.name for p in tts.candidates("auto", factory(f))] == ["edge", "say"]


def test_a_named_provider_is_the_only_candidate():
    f = fakes()
    assert [p.name for p in tts.candidates("kokoro", factory(f))] == ["kokoro"]
    with pytest.raises(TtsError, match="unknown provider"):
        tts.candidates("polly", factory(f))


def test_auto_falls_through_on_the_first_scene_and_keeps_one_voice(tmp_path: Path):
    f = fakes(elevenlabs={"fails": True})
    lines = []
    p, files = tts.speak_all(["one", "two", "three"], "auto", "Brian", 1.1, tmp_path, lines.append,
                             factory(f))  # fmt: skip
    assert p.name == "kokoro"
    assert [x.name for x in files] == ["01.wav", "02.wav", "03.wav"]
    assert f["elevenlabs"].calls == [("one", "Brian", 1.1)]
    # The voice named for the first choice is not forced on the next one.
    assert f["kokoro"].calls == [("one", None, 1.1), ("two", None, 1.1), ("three", None, 1.1)]
    assert f["edge"].calls == []
    assert any("elevenlabs failed" in line for line in lines)


def test_a_named_provider_does_not_fall_through(tmp_path: Path):
    f = fakes(edge={"fails": True})
    with pytest.raises(TtsError, match="boom"):
        tts.speak_all(["one"], "edge", None, 1.0, tmp_path, lambda _: None, factory(f))
    f = fakes(kokoro={"ready": False})
    with pytest.raises(TtsError, match="not set up"):
        tts.speak_all(["one"], "kokoro", None, 1.0, tmp_path, lambda _: None, factory(f))


def test_nothing_available(tmp_path: Path):
    f = {n: Fake(n, ready=False) for n in tts.ORDER}
    with pytest.raises(TtsError, match="no voice provider"):
        tts.speak_all(["one"], "auto", None, 1.0, tmp_path, lambda _: None, factory(f))


def test_edge_rate_strings():
    assert rate(1.0) == "+0%"
    assert rate(1.1) == "+10%"
    assert rate(0.85) == "-15%"


def test_elevenlabs_voice_choice():
    voices = [
        {"name": "Sarah - warm", "voice_id": "a" * 20},
        {"name": "Adam", "voice_id": "b" * 20},
    ]
    assert pick(voices, None) == "b" * 20
    assert pick(voices, "sarah") == "a" * 20
    with pytest.raises(TtsError, match="no ElevenLabs voice named 'Zed'"):
        pick(voices, "Zed")
    assert pick([{"name": "Zed", "voice_id": "z"}], None) == "z"
    assert looks_like_id("21m00Tcm4TlvDq8ikWAM")
    assert not looks_like_id("Brian")


def test_elevenlabs_needs_a_key(monkeypatch):
    monkeypatch.delenv("ELEVENLABS_API_KEY", raising=False)
    assert "ELEVENLABS_API_KEY" in (ElevenLabs().unavailable() or "")
    monkeypatch.setenv("ELEVENLABS_API_KEY", "k")
    assert ElevenLabs().unavailable() is None


def test_kokoro_needs_complete_model_files(tmp_path: Path):
    assert len(missing(tmp_path)) == 2
    assert "setup kokoro" in (Kokoro(tmp_path).unavailable() or "")
    (tmp_path / "voices-v1.0.bin").write_bytes(b"short")
    assert "voices-v1.0.bin" in missing(tmp_path)


def test_elevenlabs_request_shape(tmp_path: Path, monkeypatch):
    import httpx

    monkeypatch.setenv("ELEVENLABS_API_KEY", "secret")
    seen = []

    def handler(request: httpx.Request) -> httpx.Response:
        seen.append(request)
        if request.url.path == "/v1/voices":
            return httpx.Response(200, json={"voices": [{"name": "Brian", "voice_id": "b" * 20}]})
        return httpx.Response(200, content=b"ID3audio")

    client = httpx.Client(transport=httpx.MockTransport(handler), headers={"xi-api-key": "secret"})
    out = tmp_path / "01.mp3"
    ElevenLabs(client).synthesize("Hello there.", None, 1.5, out)
    assert out.read_bytes() == b"ID3audio"
    post = seen[-1]
    assert post.url.path == f"/v1/text-to-speech/{'b' * 20}"
    assert post.url.params["output_format"] == "mp3_44100_128"
    assert post.headers["xi-api-key"] == "secret"
    import json

    body = json.loads(post.content)
    assert body["text"] == "Hello there."
    assert body["model_id"] == "eleven_multilingual_v2"
    assert body["voice_settings"]["speed"] == 1.2  # clamped to the API's range


def test_elevenlabs_reports_http_errors(tmp_path: Path, monkeypatch):
    import httpx

    monkeypatch.setenv("ELEVENLABS_API_KEY", "secret")
    client = httpx.Client(
        transport=httpx.MockTransport(lambda r: httpx.Response(401, text="invalid key"))
    )
    with pytest.raises(TtsError, match="HTTP 401 invalid key"):
        ElevenLabs(client).synthesize("Hi.", "b" * 20, 1.0, tmp_path / "x.mp3")
