# Mercury-Motion — Marketing Strategy
**Version:** 0.1.0-draft
**Date:** 2026-03-22
**Status:** Draft

---

## Brand Identity

### Name & Mythology
**Mercury-Motion** — named for Mercury, the Roman god of speed, travel, and communication. The fastest messenger of the gods. God of commerce and craftsmanship. The name carries the core promise without explanation.

- **Core brand word:** Fast. Not "fast for open source." Just fast.
- **Secondary brand words:** Precise. Honest. Professional. Free.
- **Tone:** Confident, technical, zero hype. Let the benchmarks speak.

### Tagline Options
- *"Video at the speed of thought."*
- *"Describe it. Render it. Done."*
- *"The video tool Remotion should have been."* (aggressive — use carefully)
- *"Fast like the god it's named for."*

### Brand Philosophy
Mercury-Motion does not market itself with adjectives. It markets with demonstrations. Every claim is backed by a number, a screenshot, or a side-by-side comparison. Cinematographers and developers are both highly skeptical audiences — they respond to evidence, not enthusiasm.

---

## Two Audiences, One Product

Mercury-Motion serves two distinct audiences who almost never overlap, yet both live inside the same tool:

| | Developer-Creators | Filmmaker-Colorists |
|---|---|---|
| **Identity** | I write code to make videos | I shoot and grade film |
| **Current tool** | Remotion | DaVinci Resolve |
| **Pain** | It's too slow and too complex | The good tools cost money or are closed |
| **What they want** | Speed, simplicity, JSON, CLI | Color science, film emulation, roto, free |
| **What they hate** | Node/npm/Chrome/React overhead | Slop, fake "AI enhance", subscription traps |
| **Where they live** | HN, r/rust, X, Dev.to | r/cinematography, YouTube, Bluesky, BMD forums |
| **Mercury-Motion hook** | Core engine | FilmTools |

These audiences **amplify each other**. A tool that cinematographers trust with their color grade, and that developers trust with their render pipeline, earns a kind of cross-community legitimacy that neither alone can provide.

---

## The Marketing Weapons

### Weapon 1: The Benchmark Page
A live, reproducible, open benchmark at `mercury-motion.dev/benchmark`.

**What it shows:** Same video rendered on the same machine spec — Mercury-Motion vs Remotion, side by side, with a running timer. Not a marketing claim. A reproducible command you can run yourself.

```
Simple text animation (10s, 1080p, 30fps):
  Remotion:       8 min 14 sec
  Mercury-Motion: 3.1 sec

"Run it yourself: mmot render benchmark.mercury.json"
```

This is the single most powerful marketing asset Mercury-Motion can have. Render time is visceral. "40 minutes → 3 seconds" is not a feature — it is a category change.

### Weapon 2: The FilmTools Demo
A video (shot on ARRI or RED, in log) graded entirely with Mercury-Motion FilmTools — ACES pipeline, ARRI LogC4 IDT, SAM2 roto mask, depth-based rack focus, film grain.

Show the before (flat log) and after (finished grade). Show the `.mercury.json` config that produced it. Show that zero proprietary software was used.

**Target:** r/cinematography, r/colorists, YouTube filmmaking community. These audiences are passionate and vocal. One well-received post can drive thousands of GitHub stars.

### Weapon 3: "AI Generates a Video" Demo
Claude Code + Mercury-Motion: a user types a description of a video in plain English, Claude generates a `.mercury.json`, `mmot render` produces an MP4 in seconds. No API. No SaaS. No subscription. Runs locally.

**Target:** AI Twitter/X, AI newsletters, developer community. This hits the AI-native content creation wave without being generative AI — the AI writes the JSON, the renderer is deterministic.

### Weapon 4: The Origin Story Post
"Why I rewrote Remotion in Rust" — a long-form technical post explaining the headless Chrome problem, timing hacks, and why the JSON approach is architecturally superior. No bashing — just engineering honesty.

**Target:** Hacker News front page (aim for Show HN: Mercury-Motion). r/rust. r/programming. Technical audiences respect thorough engineering thinking.

### Weapon 5: The Name
Mercury, god of speed. Every mention of the name reinforces the brand promise without needing additional explanation. "Fast like the god it's named for" requires zero prior context.

---

## Channel Strategy

### GitHub
The primary distribution channel. Everything open, everything MIT.

- **README as marketing:** Benchmark numbers, feature list, architecture diagram, animated demo GIF in the first screen.
- **Releases:** Every release gets a detailed changelog with performance numbers.
- **Issues & discussions:** Respond fast. Speed in community management mirrors speed in rendering.
- **Star goal:** 1K in month 1, 10K by month 6.

### Hacker News
- **Launch:** Show HN post on release day. Technical, honest, benchmark-forward.
- **Ongoing:** Submit FilmTools launch as a separate Show HN. "Show HN: Free, open-source ACES color grading + ARRI log support + SAM2 roto, runs offline"
- **Tone:** No hype. Show the architecture. Invite criticism.

### Reddit

| Subreddit | Content |
|---|---|
| r/rust | Architecture post: "Why we built a video renderer in Rust" |
| r/programming | Benchmark post + Show Reddit |
| r/cinematography | FilmTools deep dive: ACES grading tutorial |
| r/colorists | SAM2 roto vs Magic Mask comparison |
| r/filmmakers | "Free DaVinci alternative for indie filmmakers" |
| r/videography | Film emulation: grain and halation deep dive |
| r/MachineLearning | SAM2 + Depth Anything integration architecture |

### YouTube
- **Format:** Technical tutorials, not vlogs.
- **FilmTools series:** "ARRI log to Rec.709 in Mercury-Motion", "SAM2 roto: rotoscoping without Premiere", "Film grain that actually looks like film"
- **Core engine series:** "Programmatic videos with JSON", "Mercury-Motion vs Remotion: render time comparison"

### X / Twitter
- Launch thread with benchmark GIF (timer ticking down from 40 minutes to 3 seconds)
- FilmTools grading screenshots (before/after)
- Short clips of the editor UI

### Bluesky
Stronger filmmaker and indie dev presence than X. Priority channel for FilmTools launch.

### Discord / Community
Launch a Mercury-Motion Discord on day one. Two channels from the start: `#core-dev` (JSON format, renderer, editor) and `#filmtools` (color science, film emulation, VFX). Different audiences, same tool.

---

## Content Calendar (First 6 Months)

| Month | Core Content | FilmTools Content |
|---|---|---|
| 1 | Launch: Show HN, benchmark page, README | FilmTools teaser: "ACES pipeline coming" |
| 2 | Architecture deep-dive blog post | FilmTools alpha: OCIO + camera log support |
| 3 | Browser/WASM launch | Film emulation deep-dive post |
| 4 | Template marketplace launch | SAM2 roto tutorial |
| 5 | AI + Mercury-Motion demo (Claude generates video) | Depth-based rack focus tutorial |
| 6 | v0.2 release (audio, precomps, Lottie) | FilmTools stable release |

---

## Positioning Against Competitors

### vs Remotion
**Don't attack. Demonstrate.**
The benchmark is the argument. Let developers run it themselves. The positioning is simple: Mercury-Motion is what Remotion would be if it started fresh in Rust with a JSON format instead of React. No hostility. Just honesty.

Messaging: *"Built for the same use case. Built differently."*

### vs DaVinci Resolve
**Position as complementary, not replacement.**
DaVinci Resolve Studio is a $300 professional suite. Mercury-Motion FilmTools is free, open, and handles the specific use case of color science + basic VFX + film emulation for the indie filmmaker who cannot justify Resolve Studio — or who wants their entire video pipeline (motion graphics + color) in one open-source tool.

Messaging: *"Professional color science. No subscription. No proprietary lock-in."*

### vs Creatomate / Shotstack
**Position on cost model.**
They charge per render or per month. Mercury-Motion renders locally for free, forever, with no per-render cost. For AI pipelines generating thousands of videos, this is a dramatic cost difference.

Messaging: *"Render 10,000 videos. Pay nothing."*

---

## FilmTools Specific Messaging

### The Anti-Slop Manifesto (for filmmaker audience)

Mercury-Motion FilmTools is built by cinema nerds for cinema nerds. The rules:

1. **Every operation is mathematically explainable.** No magic. No black box "AI enhance." If you apply ARRI LogC4, you can read the transfer function.
2. **AI assists. Never creates.** SAM2 draws a mask. You decide what to do with it. Depth Anything estimates depth. You decide the focus distance. The AI serves you.
3. **Offline. Always.** Your footage stays on your drive. Always.
4. **No subscription. No "Pro" tier for color.** ACES, LUTs, CDL, grain — all free, forever.
5. **We hate AI slop as much as you do.** That's why we built it this way.

---

## Metrics

### North Star Metric
**Monthly active renders** — the number of `.mercury.json` files rendered per month across all users. This captures both developer usage (batch pipelines) and filmmaker usage (single exports) in one number.

### Supporting Metrics
- GitHub stars (community trust signal)
- Discord members (community size)
- CLI downloads (actual usage)
- WASM app monthly active users (browser adoption)
- Template marketplace listings (ecosystem depth)

---

*This document is a living strategy. Update as channels and traction data evolve.*
