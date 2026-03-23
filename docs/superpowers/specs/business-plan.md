# Mercury-Motion — Business Plan
**Version:** 0.1.0-draft
**Date:** 2026-03-22
**Status:** Draft

---

## The Opportunity

The programmatic video market is broken in two places simultaneously:

**For developers:** Remotion is the only serious tool. It is slow (headless Chrome), fragile (timing hacks), requires a full React/Node/npm toolchain, and is source-available — not open source. Teams of 4+ pay per seat. The architectural problems are unfixable without a rewrite.

**For filmmakers:** DaVinci Resolve is the only free professional-grade color tool. It is closed source, proprietary, database-driven, and does not run in the browser. No open-source tool offers professional color science (ACES, camera log formats, OCIO), film emulation, and AI-assisted VFX in a single offline-first package.

Mercury-Motion solves both. Not with incremental improvements — with an architectural clean break.

---

## Products

### Mercury-Motion Core
A Rust-native, JSON-driven programmatic video creation engine and desktop editor.

- **`.mercury.json` format** — human-readable, git-diffable, AI-generatable video projects
- **Native renderer** — Skia + wgpu, 30–100x faster than Remotion
- **Desktop editor** — Tauri 2.0, native GPU preview, full timeline
- **Browser app** — same React frontend, WASM renderer, zero server required
- **CLI tool** (`mmot`) — single binary, zero dependencies

### Mercury-Motion FilmTools
A professional filmmaker toolkit built into a separate page of the same app.

- **Color Science** — full ACES pipeline, OCIO, all major camera log formats (ARRI, RED, Sony, Canon, Panasonic, Fuji, Blackmagic), LUT engine, CDL
- **Film Emulation** — grain, halation, bloom, gate weave — pure algorithmic physics math
- **Smart Restoration** — stabilization, temporal NR, deflicker, frame interpolation, upscaling
- **VFX Assist** — SAM2 masking/roto, depth-based rack focus, chroma key
- **Audio** — RNNoise denoising, Whisper transcription, multi-camera sync

Zero generative AI. Everything runs offline, locally, forever.

---

## Market Segments

### Primary: Developer-Creators (~8M globally)
Software engineers producing programmatic videos — data visualizations, code explainers, product demos, social media content, automated reports. They want Remotion's *idea* without Remotion's browser overhead.

**Conversion trigger:** Speed. A 40-minute Remotion render becomes 12 seconds. This is not a feature comparison — it is a category shift.

### Secondary: Content Teams Using Templates (~50M globally)
Marketing, growth, and content teams that fill developer-built templates with data to produce batches of personalized videos. They want local, offline, no SaaS subscription.

**Conversion trigger:** Cost. Creatomate and Shotstack charge per render or per month. Mercury-Motion is free, local, forever.

### Tertiary: AI/Automation Pipelines (fastest growing)
Agents, pipelines, and scripts generating videos programmatically at scale. The `.mercury.json` format is trivially LLM-generatable. The CLI is ideal for batch processing.

**Conversion trigger:** Format. JSON is the universal language of AI agents. Remotion requires writing React.

### FilmTools Primary: Indie Filmmakers & Colorists (~2M globally)
Cinematographers, colorists, and post-production professionals who need professional color science tools but cannot justify $300/year for DaVinci Resolve Studio — or refuse to be locked into a proprietary ecosystem.

**Conversion trigger:** Access. ACES, OCIO, ARRI LogC4, and SAM2 roto in a single free, open-source, offline tool. Nothing else offers this.

### FilmTools Secondary: Cinema-Literate Content Creators
YouTube filmmakers, documentary makers, and narrative content creators who care about image quality — grain, halation, proper log-to-rec709 transforms — but currently rely on expensive plugins (FilmConvert, Dehancer) or manual LUT workflows.

**Conversion trigger:** Quality + cost. Film emulation that is mathematically correct (not a preset filter), free, and explainable.

---

## Revenue Model

### Phase 1–2: Pure Open Source (MIT, Free)
Mercury-Motion Core and FilmTools are fully free, MIT-licensed. No license tiers. No "source available for teams." Revenue comes from community growth, not product gates.

**Goal:** Become the default open-source programmatic video tool within 18 months. GitHub stars, npm/cargo downloads, community adoption are the metrics.

### Phase 3: Hosted Services (Optional, Never Required)
The core tool stays free forever. Optional paid services for teams and power users:

| Service | Model | Audience |
|---|---|---|
| **Cloud Render API** | Per-minute or subscription | Teams running batch pipelines at scale |
| **Template Marketplace** | Revenue share (80/20 creator) | Template creators and buyers |
| **Priority Support** | Annual subscription | Studios and agencies |
| **Managed Self-Hosting** | Annual subscription | Enterprises with data sovereignty needs |

### Phase 4: FilmTools Pro (Optional Add-on)
AI model weights for advanced FilmTools features (Real-ESRGAN upscaling, Depth Anything DOF) are open source but large (~500MB+). A FilmTools Pro bundle — pre-packaged, auto-updating model weights with a one-click setup — could be a modest paid product ($49 one-time or $5/month) while the algorithmic core (color science, film emulation, stabilization) stays free.

**The hard rule:** If you can render it with the free binary, you can always render it with the free binary. Paid features are convenience and AI model packaging, never color science or fundamental capabilities.

---

## Competitive Landscape

### Direct Competitors

| Tool | Speed | License | Browser | Filmmaker Tools | Price |
|---|---|---|---|---|---|
| **Mercury-Motion** | ★★★★★ | MIT | ✓ (WASM) | ★★★★★ | Free |
| Remotion | ★★ | Source-avail. | ✓ (native) | ✗ | Free / $150+/yr |
| Motion Canvas | ★★★ | MIT | Partial | ✗ | Free |
| Revideo | ★★★ | MIT | Partial | ✗ | Free |
| DaVinci Resolve | ★★★★ | Proprietary | ✗ | ★★★★ | Free / $300 Studio |
| Creatomate | ★★★ | SaaS | ✓ | ✗ | $49–$299/mo |
| Shotstack | ★★★ | SaaS | ✓ | ✗ | $49–$349/mo |

### Why Competitors Cannot Respond

**Remotion** cannot match Mercury-Motion's speed or determinism — it would require removing React and Chrome, which are the entire product.

**DaVinci Resolve** cannot go open source or browser-first — Blackmagic Design's business model depends on the proprietary Studio tier and hardware sales.

**Creatomate/Shotstack** cannot go offline-first — they are SaaS businesses by design. Their infrastructure cost is their revenue mechanism.

---

## Go-To-Market Strategy

### Phase 1: Developer Community (Months 1–6)
**Channels:** Hacker News, r/programming, r/rust, X/Twitter, Dev.to

**Weapon:** The benchmark. A single page showing a side-by-side render time comparison — same video, same machine, Mercury-Motion vs Remotion. "40 minutes → 12 seconds." This is visceral and shareable.

**Supporting content:**
- "Why I rewrote Remotion in Rust" blog post
- Open-source everything from day one
- JSON format documentation with interactive examples

### Phase 2: Filmmaker Community (Months 4–9)
**Channels:** Reddit (r/cinematography, r/colorists, r/filmmakers), YouTube, Bluesky, DaVinci Resolve forums

**Weapon:** FilmTools. A free, open-source, offline alternative to $300 DaVinci Resolve Studio's Magic Mask + color tools. The ARRI LogC4 → ACES pipeline, free, in a tool that also makes motion graphics.

**Supporting content:**
- "ACES grading for free, in Mercury-Motion FilmTools" tutorial
- Comparison: DaVinci Magic Mask vs Mercury-Motion SAM2 roto — side by side
- Film emulation deep-dive: "Why our grain algorithm is different from a LUT"

### Phase 3: AI/Automation Wave (Months 6–12)
**Channels:** AI Twitter/X, AI newsletters, LangChain/CrewAI/Claude communities

**Weapon:** The JSON format + CLI. "Your AI agent can now generate videos. Describe a video in plain English, get a `.mercury.json`, render it locally in 3 seconds. No API. No SaaS. No per-render fee."

**Supporting content:**
- Claude Code + Mercury-Motion demo: AI generates a data visualization video from a CSV
- `remotion-dev/skills`-equivalent skill for Mercury-Motion's JSON format
- Template gallery with AI-generatable examples

### Launch Milestone Goals

| Milestone | Target | Timeline |
|---|---|---|
| GitHub stars | 1,000 | Month 1 |
| GitHub stars | 10,000 | Month 6 |
| Discord/community members | 5,000 | Month 6 |
| Templates in marketplace | 100 | Month 9 |
| Cloud render API beta | - | Month 12 |

---

## Moat Summary

Mercury-Motion's competitive moat is **architectural**, not feature-based. It cannot be erased by a competitor adding a feature — it would require them to rebuild from scratch:

1. **Speed** — 30–100x faster than Remotion is an architectural consequence of not using Chrome
2. **Determinism** — guaranteed pixel-identical frames across machines, runs, and time
3. **Format** — `.mercury.json` is git-diffable, AI-generatable, scriptable — Remotion's React/TSX is none of these
4. **License** — MIT forever; Remotion's source-available model is a permanent disadvantage with teams
5. **FilmTools** — no other open-source tool combines programmatic video creation with professional color science, film emulation, and AI-assisted VFX in a single offline application
6. **The name** — Mercury, the fastest Roman god. The brand promise is in the name.

---

*This document is a living plan. Update as strategy evolves.*
