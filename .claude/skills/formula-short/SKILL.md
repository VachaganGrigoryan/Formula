---
name: formula-short
description: Use when asked to make a YouTube Short, Instagram Reel, TikTok or other vertical/narrated video of a Formula tool or formula (e.g. "make a short about Lorenz", "/formula-short a-star focus on heuristics"), or to re-voice, re-theme or resize one already generated.
---

# Formula Short

## Overview

`tools/shorts` turns a narration script into a finished video: the voice records each scene, `formula-render` draws the live tool at the exact size, ffmpeg mixes them. You write the script; the pipeline does the rest. Every build lands in `generated/<YYYY-MM-DD_HHMM>-<tool>/` (gitignored).

## Steps

1. **Pick the tool.** `cd tools/shorts && uv run formula-shorts tools` lists ids, names and the `state` fields each accepts. Match the user's words to one id; ask (AskUserQuestion) only if two fit.
2. **Read** `assets/content/<id>/doc.md` (Overview, Intuition, Applications) and the user's notes.
3. **Write** `generated/drafts/<id>-script.md` following `references/script-guide.md`.
4. **Dry run**, then read the contact sheet (one frame per scene) with the Read tool:
   `uv run formula-shorts build ../../generated/drafts/<id>-script.md --dry-run`
   Check every caption against its frame: colors, counts and claims must match what is on screen. Fix the script and dry-run again until it does.
5. **Build** with the voice asked for (default `--provider auto`):
   `uv run formula-shorts build ../../generated/drafts/<id>-script.md [--provider edge|kokoro|elevenlabs|say] [--voice NAME] [--chrome] [--theme light] [--size square]`
6. **Report** from the printed JSON: `short.mp4` path, length, provider and voice, any `warnings`. Mention `post.md` (title, description, hashtags) and `captions.srt`.

## Quick reference

| Want | Do |
|---|---|
| Voices | `formula-shorts voices --provider edge` (Microsoft neural, free) / `kokoro` (local) / `elevenlabs` |
| Kokoro first use | `formula-shorts setup kokoro` (downloads ~350 MB once) |
| ElevenLabs | user exports `ELEVENLABS_API_KEY`; free plan ~10k characters a month |
| Workbench look | `--chrome` (viewport frame and title tab) |
| Other sizes | `--size square`, `landscape`, or `WxH` |
| Music | `--music file.mp3` (looped at -18 dB, fades out) |

## Common mistakes

- **The picture freezes.** Step-through tools (sorts, searches, MST, BST) finish in seconds. Slow them with `state: {"speed": N}` in the first scene and use `restart: true` to run them again later.
- **Captions describe what is not there.** Only the contact sheet shows what the viewer sees. Never claim a color or count you have not checked on it.
- **The voice reads symbols.** Captions may say `A*`, `ρ`, `x²`; give such scenes a `say:` line in plain words ("A star", "rho", "x squared").
- **Unknown `state` keys** fail validation; use only the fields `formula-shorts tools` lists for that tool.
- **A caption over three lines** shrinks and triggers a warning; split the scene instead.
