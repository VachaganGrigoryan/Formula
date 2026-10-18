# Writing a Formula Short script

## Shape

- **4 to 6 scenes**, 15 to 30 words each, **90 to 140 words** in total (about 35 to 55 seconds).
- **Scene 1 is the hook**: a question or a surprising claim in the first sentence. No "In this video".
- Then: what the formula says, how it behaves (tie each scene to what changes on screen), one place it is used, and a closing line worth remembering.
- Spoken English, short sentences, one idea per scene. The caption *is* the narration.
- The validator rejects more than 8 scenes, more than 45 words in a scene, or more than 220 in total.

## Format

```markdown
---
tool: lorenz                 # required: an id from `formula-shorts tools`
title: The butterfly effect  # optional, default: the tool's name
kicker: Chaos theory         # optional, default: the tool's category
theme: dark                  # dark | light
chrome: false                # true: workbench viewport with its title tab
size: shorts                 # shorts (1080x1920) | square | landscape | WxH
provider: auto               # auto | edge | kokoro | elevenlabs | say
voice: en-US-AndrewMultilingualNeural   # optional, per provider
speed: 1.0                   # 0.5 to 2
warmup: 3                    # seconds the tool runs before frame one
music: /path/to/bed.mp3      # optional
post:
  title: Two paths, one hundred-thousandth apart   # 60 characters or fewer
  description: One or two sentences for the upload form.
  hashtags: [math, chaos, physics, shorts]          # 3 to 6
---

## Scene heading (shown above the caption, 2 to 4 words)
state: {"rho": 15}       # optional: tool parameters set when the scene starts
equations: false         # optional: hide the equation lines in this scene
restart: true            # optional: start the tool over (fresh run)
say: Lower rho below twenty-four.   # optional: what the voice says instead
Lower ρ below 24.        # the caption (and narration unless `say:` is given)
```

Front matter values can be overridden on the command line (`--theme`, `--chrome`, `--size`, `--provider`, `--voice`, `--speed`, `--music`, `--warmup`).

## Pacing the animation

Each scene lasts as long as its narration plus a short pause, so the tool must have something to show for the whole video:

| Tool kind | Do |
|---|---|
| Continuous (Lorenz, Fourier, flow field, Life, Lissajous) | Nothing special; `state` changes are the story beats. |
| Step-through (sorts, BFS/DFS/Dijkstra/A*, Kruskal/Prim, BST) | `state: {"speed": N}` in scene 1 so one run spans several scenes (pathfinding default is 60 steps/s; try 10 to 20). `restart: true` to replay, e.g. at a faster speed. `warmup: 0` to start from the beginning. |
| Still or interactive (lerp, normalization, luminance, distance) | Change parameters scene by scene with `state`; the dry run shows whether each change is visible. |

## Worked example (Lorenz, 73 words, ~28 s)

```markdown
---
tool: lorenz
title: The butterfly effect
post:
  title: Two paths, one hundred-thousandth apart
  hashtags: [math, chaos, physics, shorts]
---

## No randomness at all
Three simple equations. No dice, no noise. And yet nobody can predict where this point will be a minute from now.

## One tiny difference
state: {"count": 2}
Two paths start one hundred-thousandth apart. For a while they move as one.

## Then they part
Their gap doubles again and again, until they swing around different wings.

## Turn down the heat
state: {"rho": 15}
equations: false
say: Lower rho below twenty-four, and the chaos dies. Both paths spiral into a calm center.
Lower ρ below 24, and the chaos dies. Both paths spiral into a calm center.

## The butterfly effect
state: {"rho": 28}
That is why weather forecasts fade after a week. Tiny errors grow.
```

## Voices

| Provider | Needs | Good voices |
|---|---|---|
| `edge` | network | `en-US-AndrewMultilingualNeural`, `en-US-AvaMultilingualNeural`, `en-GB-RyanNeural` |
| `kokoro` | `formula-shorts setup kokoro` once | `af_heart`, `am_michael`, `bf_emma`, `bm_george` |
| `elevenlabs` | `ELEVENLABS_API_KEY` | a voice name from `formula-shorts voices --provider elevenlabs` |
| `say` | macOS | `Samantha`, `Daniel` (robotic; offline fallback) |

`auto` takes the first one that is set up, in the order elevenlabs, kokoro, edge, say. If it fails on the first scene, the next one records the whole Short, so the voice never changes mid-video.
