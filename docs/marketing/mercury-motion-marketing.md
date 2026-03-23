# Mercury-Motion: Marketing Strategy & Brand Playbook

---

## Table of Contents

1. [Brand Identity](#1-brand-identity)
2. [Positioning Statements](#2-positioning-statements)
3. [Messaging Framework](#3-messaging-framework)
4. [Competitor Messaging](#4-competitor-messaging)
5. [Content Marketing Strategy](#5-content-marketing-strategy)
6. [Launch Strategy](#6-launch-strategy)
7. [Community Building](#7-community-building)
8. [SEO & Discoverability](#8-seo--discoverability)
9. [Partnership Opportunities](#9-partnership-opportunities)
10. [Metrics & KPIs](#10-metrics--kpis)

---

## 1. Brand Identity

### 1.1 Brand Story

Mercury — *Mercurius* to the Romans — was not merely fast. He was the only god permitted to cross every boundary: between the living and the dead, between the heavens and the earth, between mortals and Olympus. He was the messenger god, the god of commerce, the god of travelers. But above all, he was the god of speed — depicted with winged sandals, a winged helmet, and a caduceus that opened any door.

This is the founding myth of Mercury-Motion.

Video creation has long lived behind walls. The wall of a browser runtime. The wall of Node.js and a 400MB dependency tree. The wall of a headless Chrome process that renders frames one at a time, as slowly as a human would watch them. The wall of proprietary formats, cloud lock-in, and subscription paywalls.

Mercury-Motion crosses every one of those walls.

It is a single binary. It speaks JSON — the closest thing the digital world has to a universal language, the modern caduceus that every system understands. It runs anywhere Rust runs. It renders at speeds that feel, frankly, divine. Where Remotion takes minutes, Mercury-Motion takes seconds. Where other tools require a Node.js install, an npm tree, a headless browser, and a prayer, Mercury-Motion requires nothing but itself.

The name carries a promise: we will move at the speed of Mercury. We will cross boundaries — between languages, between platforms, between human intention and rendered output. We will be the messenger that turns a JSON description of a vision into a finished MP4, faster than you thought possible.

Mercury did not need roads. Neither does Mercury-Motion.

### 1.2 Brand Values

**Speed — without compromise**
Speed is not a feature. Speed is the point. Every architectural decision — Rust, GPU acceleration, zero external dependencies — exists in service of rendering video as fast as physically possible. We do not apologize for being obsessive about this. We benchmark everything. We publish the numbers. Speed is our religion.

**Simplicity — the elegance of constraint**
A `.mmot.json` file is a complete video. No project scaffolding. No `npm install`. No webpack config. No hydration. A file you can read, diff, commit, and hand to an AI — and it becomes a video. Simplicity is not the absence of power; it is the sign that power has been fully tamed.

**Openness — MIT, forever**
Not "open core." Not "free tier." Not "community edition." MIT. The license that trusts you. You can fork it, sell it, embed it, modify it, and never owe us anything. Openness is not a business model we reluctantly accept — it is a conviction. The best tools in the world are open tools. We intend to be one of them.

**Power — the full creative surface**
Simplicity without power is a toy. Mercury-Motion is not a toy. Layers. Keyframes. Easing curves. Compositions. Props and templates. Lottie import. After Effects interop. GPU-accelerated compositing. ElevenLabs voice synthesis integration. A native desktop editor with a timeline. We give you the full surface area of professional video creation — without hiding it behind abstractions that slow you down.

**Determinism — trust your output**
The same `.mmot.json` file, on any machine, on any day, produces the same MP4. Byte for byte. This is not a minor footnote — it is a foundational guarantee. You can version-control your videos. You can reproduce any output from any point in history. You can test your video pipeline the same way you test your code.

### 1.3 Tone of Voice

**Technical, but never condescending.**
Our audience writes code for a living. They do not need things explained slowly. They need things explained accurately. We use precise technical language because precision is a form of respect. We do not water down complexity — we make it approachable without making it vague.

**Confident, but never arrogant.**
We know Mercury-Motion is fast. We know it is a better tool for many workloads. We will say so clearly, with benchmarks to back it up. But we are not interested in sneering at competitors or the developers who use them. Remotion is a good project built by good people solving a real problem. We just solve it differently — and faster.

**Developer-first, always.**
We write copy that reads like good documentation: concrete, specific, example-driven. We prefer a code block over a metaphor. We prefer a benchmark number over an adjective like "blazing" or "lightning-fast." When we use metaphors (and we will, sparingly), they are earned by specificity elsewhere.

**Honest about trade-offs.**
Mercury-Motion is not the right tool for every situation. We will say this plainly. It is the right tool for developers who want speed, portability, and the power to automate video creation at scale without a browser runtime. We do not oversell. We explain. The audience can judge.

**Dry wit, earned.**
Developer audiences appreciate deadpan humor. We can be funny. But jokes are seasoning, not the meal. The meal is excellent tooling and honest communication.

### 1.4 Visual Identity Guidelines

#### Color Palette

Mercury-Motion's visual identity draws from two sources: the metallic shimmer of liquid mercury and the deep space of high-performance computing — dark terminals, glowing output, the aesthetic of something running very fast in the dark.

**Primary — Mercury Silver:** `#C0C0C0` → `#E8E8E8`
The element mercury is a liquid silver metal. Our primary neutral embraces this: a cool, bright silver that signals precision and modernity. Used for primary typography on dark backgrounds and key UI chrome.

**Accent — Velocity Orange:** `#FF6B2B`
A warm, high-energy orange that cuts through dark backgrounds like a benchmark result that embarrasses the competition. Used for CTAs, highlights, key numbers, and moments where we want to signal action or urgency.

**Background — Deep Void:** `#0D0D0F`
Almost-black. Darker than standard dark mode. The color of a terminal that is doing serious work. Primary background for the website, editor, and all branded materials.

**Secondary — Caduceus Gold:** `#D4A843`
A restrained gold, referencing Mercury's caduceus and the divine nature of the brand story. Used sparingly — callout boxes, premium indicators, the logo mark itself.

**Surface — Charcoal:** `#1A1A1F`
For cards, code blocks, editor panels — elevated surfaces on the deep void background.

**Success / Render Complete — Phosphor Green:** `#39FF14`
Used exclusively to signal successful renders, passing benchmarks, and completion states. A deliberate nod to terminal culture.

**Danger / Error — Rust Red:** `#B7410E`
Named for the language we're written in. Used for error states.

#### Typography

**Display / Headlines:** [Geist](https://vercel.com/font) or **Space Grotesk** — modern, geometric, technical. Slightly condensed. Works at large sizes for hero copy and benchmark callouts. The letterforms should feel engineered, not decorated.

**Body / Prose:** **Inter** — the default excellent choice for developer-facing content. Highly readable at small sizes, neutral, widely available.

**Monospace / Code:** **JetBrains Mono** — ligature-aware, highly readable, widely respected in the developer community. All JSON examples, CLI output, and code samples use this exclusively.

**Hierarchy principles:**
- Headlines are bold and large. We do not hedge with small type.
- Key numbers (30x, 100x, 30MB) are set large and in Velocity Orange.
- Code is always in a distinct block. Never inline without a monospace typeface.

#### Logo Concept Description

**The Mark:** A stylized caduceus — but minimal, geometric, modern. The two snakes become two forward-slash characters (`//`) intertwined — a programmer's comment marker, suggesting speed (in racing, slashes evoke motion trails) and code. The wings at the top are simplified to a single horizontal motion blur line, suggesting velocity rather than depicting it literally.

**The Wordmark:** "Mercury-Motion" set in Space Grotesk Bold. The hyphen is replaced by a custom glyph — a rightward-pointing motion trail, thin and tapered — reinforcing the brand's obsession with forward movement.

**Alternate mark:** For small contexts (favicon, npm badge, GitHub avatar), a single stylized `M` with a motion trail extending from its right leg. Recognizable at 16×16px.

**Usage rule:** The mark is always dark background, light mark — or light background, dark mark. Never Velocity Orange as a background color for the logo. Orange is an accent; it is not a field.

---

## 2. Positioning Statements

### 2.1 Primary Segment: Developer-Creators

**For** software engineers who create code walkthroughs, data visualizations, product demos, and technical content **who need** to produce polished, programmatic video without building a React-based video pipeline or waiting for a headless browser to crawl through frames, **Mercury-Motion is** a Rust-native, GPU-accelerated video creation engine **that** renders complete MP4s from a single JSON file in seconds rather than minutes, with zero Node.js, zero browser, and zero external dependencies — **unlike** Remotion, which requires a full JavaScript ecosystem and renders at 1×-3× realtime regardless of machine power.

### 2.2 Secondary Segment: Content & Marketing Teams at Scale

**For** content operations and marketing engineering teams who need to generate hundreds or thousands of personalized, templated videos at scale **who are frustrated by** the infrastructure cost, rendering time, and operational complexity of cloud-based video generation APIs and browser-based renderers, **Mercury-Motion is** a single-binary video rendering engine with a JSON template system **that** makes video generation a zero-dependency shell command capable of saturating GPU cores and producing thousands of renders per hour — **unlike** cloud SaaS video APIs, which bill per render, require internet access, and introduce latency and vendor risk into production pipelines.

### 2.3 Tertiary Segment: AI & Automation Pipelines

**For** AI engineers and automation architects who need to programmatically generate video as part of an agent pipeline, a code-generation workflow, or an AI-native product **who require** a video generation interface that is language-agnostic, machine-writable, and deterministic, **Mercury-Motion is** a JSON-first video format with a CLI renderer **that** accepts AI-generated scene descriptions and returns reproducible MP4s with no runtime dependencies or API calls — **unlike** every existing video creation tool, which assumes a human is in the loop and requires either a browser context or a proprietary API.

---

## 3. Messaging Framework

### 3.1 Core Headline

> **Describe your video in JSON. Render it in seconds. No browser. No Node. No waiting.**

This is the single sentence that defines Mercury-Motion. It is specific, it is falsifiable, and it makes exactly three promises that we can keep.

**Alternate headline (shorter):**
> **Programmatic video at the speed of Rust.**

**Alternate headline (benefit-first):**
> **Your 4-minute video render just became a 4-second one.**

### 3.2 Supporting Messages by Audience Segment

#### Developer-Creators

- "Write your animation like you write your tests: in plain text, version-controlled, reproducible."
- "No more `npm install` to make a video. One binary. One JSON file. One MP4."
- "Mercury-Motion renders a 60-second 1080p video in under 3 seconds on a modern GPU. Remotion takes 3 minutes."
- "Your `.mmot.json` file is git-diffable, AI-readable, and fully portable. Ship it anywhere."
- "The editor has a timeline. The renderer has no GUI requirements. Use whichever you need."

#### Content & Marketing Teams

- "Generate 10,000 personalized video variants without a cloud bill that scales with your ambitions."
- "Template your brand video in JSON. Swap the name, the logo, the data. Run `mmot render --props ./batch.json`. Done."
- "No render farm. No encoding queue. No cloud API. One server with a GPU. Infinite scale."
- "Mercury-Motion is a shell command. It fits into any CI/CD pipeline, any cron job, any Makefile."
- "Deterministic output means you can test your video templates in a staging environment and know exactly what production will look like."

#### AI & Automation Pipelines

- "JSON is the language every LLM already speaks. Mercury-Motion understands JSON natively."
- "Describe a video scene to GPT-4. Get back a `.mmot.json`. Run `mmot render`. Ship an MP4. No human required."
- "The format is designed to be machine-writable. Every property is typed, documented, and bounded."
- "Zero runtime dependencies means Mercury-Motion runs inside a Docker container, a Lambda function, an edge node, or a Raspberry Pi."
- "The render is deterministic: the same JSON always produces the same MP4. Your video pipeline can have unit tests."

### 3.3 Key Proof Points

| Claim | Proof Point | Benchmark Condition |
|---|---|---|
| 30–100x faster than Remotion | Renders a 60s 1080p/30fps video in ~3s (Mercury-Motion) vs. ~4–6 min (Remotion) | Single RTX 4070 GPU, equivalent scene complexity |
| Zero dependencies | Single `mmot` binary, ~30MB, no Node.js, no Chromium, no npm | Verified on clean Ubuntu 22.04 LTS install |
| Deterministic output | SHA-256 hash of output MP4 is identical across 100 runs on 3 different machines | Same `.mmot.json`, same `mmot` binary version |
| AI-friendly format | `.mmot.json` is valid JSON, fully schema-documented, LLM-completable | Validated via GPT-4o, Claude 3.5, Gemini 1.5 Pro |
| MIT licensed | LICENSE file in repository root | SPDX: MIT |

**Numbers to memorize and repeat:**
- **30–100x** — rendering speed advantage over Remotion
- **~30MB** — binary size (smaller than a single node_modules package in many projects)
- **0** — external runtime dependencies
- **1** — number of files needed to describe a complete video
- **∞** — number of programming languages that can generate a `.mmot.json`

### 3.4 Messages to Avoid

**Do not say:**
- "Blazing fast" — every tool says this. It means nothing. Use numbers.
- "Lightning-fast" — same problem. Say "30x faster." Say "3 seconds." Be specific.
- "Revolutionary" — you are a better video renderer, not a social movement. Stay grounded.
- "Simple" without context — everything claims to be simple. Show the five-line JSON example instead.
- "Remotion killer" — this positions us reactively, as a response to another product rather than a solution to a problem. We are building toward something, not away from something.
- "The future of video creation" — vague, unearned, and sounds like a pitch deck. Stick to what is true and verifiable.
- "No-code" — Mercury-Motion is code-first. JSON is a data format written by people who write code. Do not mislabel your audience.
- "Enterprise-grade" — this is a term used by tools that want to charge more. We are MIT-licensed and open. We do not use enterprise language.

**Avoid implied promises you cannot currently keep:**
- Do not claim After Effects feature parity until AE import is production-ready.
- Do not claim GPU acceleration is available on all platforms until it is tested and shipped.
- Do not quote benchmark numbers you have not run on reproducible hardware and documented publicly.

---

## 4. Competitor Messaging

### 4.1 Positioning Against Remotion

Remotion is the most direct competitor and the most relevant reference point for the developer audience. The messaging strategy is not to attack Remotion but to articulate a clear and honest technical differentiation. Remotion's team is talented; their tool has real users and real value. Our message is: "For the use cases where rendering speed, portability, and dependency elimination matter, Mercury-Motion is the right tool."

**The core differentiation:**

| Dimension | Remotion | Mercury-Motion |
|---|---|---|
| Runtime | Node.js + headless Chrome | None (single binary) |
| Render speed | ~1–3× realtime | 30–100× realtime |
| Binary size | 400MB+ (with Node and Chromium) | ~30MB |
| Video format | JSX/React components | `.mmot.json` |
| Language requirement | JavaScript/TypeScript | Any language (or none) |
| AI-writable | Indirect (LLM writes JSX) | Native (LLM writes JSON) |
| License | Remotion license (not fully OSI-free) | MIT |
| CI/CD friendliness | Requires Node environment | Any shell |

**What Remotion does better (and we should acknowledge this):**
- React component model is familiar to frontend developers
- Remotion Studio is a mature, polished editor
- Larger existing community and ecosystem
- Better suited for teams that already live in the JavaScript ecosystem

**Our honest positioning:**
"If you are a frontend developer building videos as React components and you are comfortable in the JavaScript ecosystem, Remotion is an excellent tool. If you are a developer who renders videos as part of a data pipeline, an AI workflow, a CI job, or a high-volume batch process — and rendering time is a real cost to you — Mercury-Motion will change your workflow."

**What to never say:**
Do not call Remotion's approach wrong. Do not characterize its license deceptively. Do not mock its performance without published benchmarks to back the claim.

### 4.2 Positioning Against Cloud SaaS Video APIs

(Targeting tools like Creatomate, JSON2Video, Shotstack, Synthesia, etc.)

**Their pitch:** "Upload your template, call our API, receive your video in the cloud."

**Their actual trade-offs:**
- Per-render pricing that scales painfully at volume
- Latency: API round-trip + queue time + render time + download time
- Internet dependency: no air-gapped deployments, no offline processing
- Vendor lock-in: their proprietary template format, their infrastructure
- Privacy: your video content traverses their servers
- No determinism: rendering behavior can change between API versions

**Our message:**
"Cloud video APIs trade your control for their convenience. At small scale, that trade is reasonable. At large scale — thousands of renders a day — you are paying per frame for something you could run locally in milliseconds. Mercury-Motion gives you the API surface of a cloud tool (JSON in, MP4 out) without the cloud. Run it on a $20/month VPS with a GPU. Own your pipeline."

**Key phrases:**
- "Zero API latency. The renderer is a shell call."
- "No per-render pricing. You own the binary."
- "No data leaves your machine unless you tell it to."
- "Your template is a JSON file in your git repository. Not a database row in their system."

### 4.3 Positioning Against After Effects

After Effects is not a programmatic tool. The audience segments that use AE and the audience segments that use Mercury-Motion overlap at the edges (motion designers who also code) but are fundamentally different.

**Do not position against AE directly.** Instead, position Mercury-Motion as the tool that *imports* AE work — taking motion designers' AE compositions and turning them into reproducible, automatable, programmatic video templates.

**The message:**
"Your motion designer delivers an After Effects comp. Mercury-Motion imports it, exposes its properties as JSON parameters, and lets your engineering team render 10,000 personalized variants overnight. No After Effects license required. No human clicking Export."

**After Effects integration is a partnership message, not a competition message.**

---

## 5. Content Marketing Strategy

### 5.1 Content Pillars

Mercury-Motion owns four content territories. Every piece of content should live in at least one of these pillars.

**Pillar 1: Speed & Performance**
The most defensible and most differentiated position. Benchmarks, profiling, GPU rendering internals, Rust performance techniques. This is where we establish technical credibility.

Content tone: rigorous, specific, show your work. Publish the benchmark code alongside the results. Let people reproduce it.

**Pillar 2: The JSON-First Video Philosophy**
The intellectual case for why describing video in a declarative, portable, text-based format is not just convenient but fundamentally superior for developer workflows. This pillar connects to version control, CI/CD, AI generation, and language-agnosticism.

Content tone: thoughtful, essay-like, slightly opinionated. This is where we develop a point of view, not just describe features.

**Pillar 3: Build Logs & Technical Deep Dives**
Behind-the-scenes content about building Mercury-Motion itself. How the GPU compositor works. How we designed the JSON schema. How we handle keyframe interpolation. How we benchmark against Remotion fairly.

Content tone: honest, educational, contributor-friendly. This builds trust and attracts open-source contributors.

**Pillar 4: Use Case Playbooks**
Concrete, step-by-step guides for specific use cases: "Generate a YouTube intro from your GitHub profile data," "Render 1,000 personalized sales videos in one CI job," "Build an AI video agent with GPT-4 and Mercury-Motion."

Content tone: practical, tutorial-style, result-oriented. Every post in this pillar ends with a working example in the repository.

### 5.2 Blog & Article Ideas

**Performance & Benchmarks**

1. **"We Benchmarked Mercury-Motion Against Remotion on 47 Different Machines. Here's What We Found."**
   Publish a reproducible benchmark suite. Run it on everything from a MacBook Air to an AWS g5.xlarge. Show the distribution, not just the best case. This is the post that gets shared.

2. **"Why We Wrote a Video Renderer in Rust (And What We Learned About GPU Compositing)"**
   A technical deep-dive into the rendering pipeline. Explains wgpu, frame compositing, the decision to avoid ffmpeg as a library. Establishes technical credibility with the Rust community.

3. **"The Hidden Cost of Headless Chrome: How Remotion's Architecture Limits Its Rendering Speed"**
   An honest, fair technical analysis of why browser-based rendering has inherent speed limits. No snark — just systems thinking. Shows we understand the problem space deeply.

**JSON-First Philosophy**

4. **"Your Video Should Be a Text File: The Case for Declarative Video Creation"**
   The manifesto post. Argues that a video described as JSON is version-controllable, diffable, AI-writable, and auditable in ways that binary formats and JSX components are not. This is the intellectual foundation for Mercury-Motion's design choices.

5. **"Git Diff for Your Animation: How to Version-Control Your Videos with Mercury-Motion"**
   Practical post showing what a `.mmot.json` diff looks like, how to use git blame to track animation changes, how to use branches for A/B testing video variants.

6. **"Why JSON is the Best Language for AI-Generated Video"**
   Explores why structured text (JSON) is a dramatically better interface between LLMs and video creation tools than code (JSX) or GUIs. Includes a working example with a prompt that generates a `.mmot.json`.

**Use Case Playbooks**

7. **"From GitHub Stats to MP4 in 60 Seconds: Automating Your Developer Year-in-Review Video"**
   A complete tutorial: fetch GitHub contribution data, template it into a `.mmot.json`, render with `mmot`, upload to YouTube. All code included.

8. **"How to Generate 10,000 Personalized Product Demo Videos with One Shell Script"**
   A guide for marketing engineering teams. Shows the template system, the `--props` flag, and a parallel rendering strategy with GNU parallel or a simple Go/Python orchestrator.

9. **"Building an AI Video Agent: GPT-4 Writes the Script, Mercury-Motion Renders the Video"**
   Step-by-step: design a system prompt that outputs `.mmot.json`, pipe it through Mercury-Motion, host the output. A complete, working AI video pipeline with zero cloud video APIs.

10. **"Using Mercury-Motion in GitHub Actions: Rendering Videos in CI/CD"**
    A practical GitHub Actions workflow that triggers on push, renders a video, and uploads it as a release asset. Shows the zero-dependency advantage concretely.

**Community & Opinion**

11. **"What We Got Wrong in the Mercury-Motion JSON Schema (v1 Retrospective)"**
    A candid post-v1 retrospective about design decisions in the JSON format that turned out to be mistakes. Shows intellectual honesty, attracts contributors, and signals that the project takes backward compatibility seriously.

12. **"The Programmatic Video Stack in 2026: Where We Are and Where We're Going"**
    A landscape analysis of the programmatic video ecosystem: Remotion, cloud APIs, AI video generators, Mercury-Motion. Honest about trade-offs. Positions Mercury-Motion as a thoughtful participant in a growing space.

### 5.3 YouTube & Video Content

(The irony is intentional: a video creation tool should have exceptional video marketing. All video content should itself be rendered using Mercury-Motion.)

**Show, Don't Tell: The Side-by-Side Demo**
The first and most important video: a side-by-side screen recording. Left panel: a Remotion project rendering. Right panel: the same scene as a `.mmot.json`, rendering with `mmot render`. The Remotion panel finishes in 4 minutes. The Mercury-Motion panel finishes in 7 seconds. No commentary needed. Let the render timer speak.

**"Building a Code Explainer Video in Mercury-Motion" — Tutorial Series**
A multi-part series showing the complete workflow: writing the JSON, using the editor, rendering, exporting. Every video in the series is itself rendered with Mercury-Motion — demonstrated by showing the `.mmot.json` file for the video you are watching in the end card.

**"Mercury-Motion Build Log" — Devlog Series**
A regular devlog series about building Mercury-Motion itself. What we shipped this week. What we are thinking about. What is hard. Developer audiences respond extremely well to authentic build-in-public content. This also serves as a community engagement mechanism.

**"30-Second Showcase" — Weekly Short-Form**
Every week, one Mercury-Motion community render gets a 30-second spotlight: what it is, how it was made, the key JSON snippet. Posted to YouTube Shorts, Twitter/X, and the Discord showcase channel.

**Conference Talk Recordings**
Submit talks to RustConf, Strange Loop, FOSDEM, and developer-focused content creator conferences. Title suggestions: "Rendering Video at the Speed of Rust," "JSON as a Video Format: Why and How," "GPU-Accelerated Compositing from Scratch in wgpu."

**Production Rule:** Every video published by Mercury-Motion must be rendered with Mercury-Motion. The source `.mmot.json` file for each video is published in a companion GitHub repository. This is both a demonstration of the tool and a library of real-world examples.

### 5.4 Twitter/X Strategy

The developer audience on Twitter/X responds to specificity, build-in-public authenticity, benchmarks, and hot takes about tools. The Mercury-Motion Twitter presence should be:

**Account handle:** `@mercurymotion` or `@mmot_dev`

**Posting cadence:** 1–2 original posts per day. Quality over quantity. Never post just to fill a calendar.

**Content types:**

*Benchmark posts (high engagement):*
"Rendered a 90-second 4K video this morning. Mercury-Motion: 11 seconds. Same scene in Remotion: 8 minutes 43 seconds. [screenshot of both terminals]"
These posts get argued with, shared, and discussed. Always include the repro instructions.

*JSON snippet posts:*
"This 23-line JSON file renders to a 10-second animated code explainer. [code block with syntax highlighting image] `mmot render intro.mmot.json`"
Show the minimal-JSON-to-impressive-output ratio. This is the aesthetic appeal of Mercury-Motion in a single tweet.

*Build-in-public posts:*
"Spent today redesigning the keyframe interpolation system. The old one was O(n²) in the number of keyframes for large compositions. The new one is O(log n). Shipping in v0.4."
Developers love this. It shows you understand your own internals.

*Hot takes (use sparingly, back with substance):*
"The reason most video creation tools are slow is that they were designed for humans to watch in real time. Mercury-Motion was designed for machines to render as fast as silicon allows. Those are different optimization targets."

*Community features:*
Retweet and quote-tweet community renders with genuine commentary. Credit contributors publicly and specifically.

*Reply strategy:*
Reply to every mention for the first 6 months. This is non-negotiable for building initial community. Reply to Remotion-adjacent discussions, programmatic video discussions, Rust ecosystem discussions — with genuine, useful contributions, not promotion.

**Threads (once per week):**
One longer thread per week on a technical topic. "How Mercury-Motion composites layers in the GPU [thread]." Threads get saved and shared more than individual posts.

### 5.5 GitHub Presence Strategy

The GitHub repository is a marketing asset. It is often the first place a developer encounters Mercury-Motion. It must be excellent.

**README Design:**
The README opens with the side-by-side benchmark numbers — not a description. First paragraph is the hook: "Mercury-Motion renders programmatic video 30–100x faster than browser-based tools. No Node.js. No Chrome. One JSON file. One binary."

Structure:
1. Benchmark hook (first thing they see)
2. Install (one line — the most important feature demo)
3. Five-line "Hello, World" `.mmot.json` example
4. Output: embedded GIF of the rendered result
5. Full feature list with links
6. Architecture overview (one diagram)
7. Contributing guide link
8. License

**The README must include a GIF.** A developer who lands on the GitHub page and sees a slick animated example is immediately oriented to what the tool does. The GIF should be generated by Mercury-Motion (obviously).

**Examples Directory:**
`/examples` should ship with at least 10 complete, runnable examples covering different use cases:
- `code-explainer/` — a code syntax highlight animation
- `data-viz/` — an animated bar chart
- `product-demo/` — a screen recording with overlay animations
- `social-card/` — a Twitter/OG card video
- `batch-render/` — example of rendering 100 variants with a shell script
- `ai-generated/` — an example JSON file generated by GPT-4o, with the prompt in the README

**Issue Labels:**
`good first issue`, `help wanted`, `docs`, `performance`, `json-schema`, `renderer`, `editor`, `integrations`. Clean labeling signals an organized project that welcomes contributors.

**Releases:**
Every release gets a GitHub Release with a rendered video changelog — a short MP4 (made with Mercury-Motion) showing what shipped. Novel, memorable, and a living demo of the tool.

---

## 6. Launch Strategy

### 6.1 Pre-Launch (8–12 Weeks Before)

**Week 1–2: Teaser Phase**
Register `mercury-motion.dev`. Set up the landing page with nothing but the headline, a short description, a terminal animation showing a render completing, and an email waitlist form. No features listed. No screenshots. Just the promise: "Programmatic video. No browser. No Node. Currently in development. Get early access."

Post a single tweet from the account: "We're building a programmatic video renderer in Rust. 30–100x faster than the browser-based alternative. MIT licensed. JSON format. No dependencies. Coming soon. [link to waitlist]"

Post to r/rust with a "Show HN"-style format: "I'm building a GPU-accelerated video renderer in Rust. Zero dependencies, JSON format, 30–100x faster than Remotion. Here's the architecture." This is a technical preview post, not a marketing post — share the wgpu approach, the compositor design, the benchmark methodology. Ask for feedback. Listen.

**Week 3–6: Build-in-Public Phase**
Begin posting weekly build logs. Each week: one blog post on `mercury-motion.dev/blog` and one Twitter thread. Topics:
- "Week 1: Designing the JSON schema for video"
- "Week 2: GPU compositing with wgpu — what I learned"
- "Week 3: The benchmark setup (and why it's fair)"
- "Week 4: The editor — Tauri + Vue, why not Electron"

Publish a draft version of the `.mmot.json` schema specification to GitHub and invite community feedback. Frame it as RFC-style: "This is what we're shipping in v1. Tell us what's wrong."

**Week 7–10: Early Access Phase**
Send invites to waitlist in batches. Require early access users to agree to share at least one render publicly. Create a `#showcase` Discord channel for these renders.

Begin collecting testimonials and benchmark reproductions from early users. Publish them (with permission) on the website.

Seed the examples library with community contributions. Offer a limited "Founding Contributors" role in Discord to anyone who submits a merged example PR before launch.

**Week 11–12: Pre-Launch Momentum**
Announce the launch date publicly. "Mercury-Motion v1.0 launches in two weeks."

Send every person on the waitlist a personalized video — generated with Mercury-Motion — welcoming them and telling them their waitlist position. This is not a gimmick: it is a demonstration that the tool can generate personalized video at scale.

Prepare all launch assets:
- Product Hunt submission (draft, not submitted yet)
- Hacker News "Show HN" post (written, not submitted)
- Reddit posts for r/rust, r/programming, r/videoproduction
- Press outreach to developer newsletters (TLDR, Bytes.dev, Changelog, Console.dev)

### 6.2 Launch Day Playbook

**Launch at 12:01 AM PST on a Tuesday** (optimal for Product Hunt; Tuesday–Thursday see highest engagement).

**Hour 0: Product Hunt**
Submit to Product Hunt. Tagline: "Programmatic video. No browser. No Node. 30–100x faster." Gallery images should be minimal and technical — terminal screenshots, JSON examples, benchmark numbers. The first comment (from the maker) should be the full story: why we built it, what makes it different, an honest assessment of what it does not yet do.

Activate every supporter who has agreed to upvote and comment. Do not ask for upvotes directly — ask people to "check it out and share their honest thoughts." Authentic comments with real feedback perform better than upvote floods.

**Hour 1: Hacker News**
Submit "Show HN: Mercury-Motion – Rust-native programmatic video renderer, 30–100x faster than Remotion."

The post body should be technical and honest:
- What it is (one paragraph)
- How it works (architecture overview with one diagram)
- Why it's faster (the key technical reason: GPU compositing, no browser overhead)
- How to try it right now (install command, five-line example)
- What it doesn't do yet (honesty builds trust on HN)
- The GitHub link

**Hour 2–3: Reddit**
Post to r/rust: "Show r/rust: I built a GPU-accelerated video renderer. Zero dependencies, MIT licensed. Feedback welcome." — Technical post. No marketing language.

Post to r/programming: "Mercury-Motion: Programmatic video in Rust, 30–100x faster than browser-based renderers." — Slightly broader framing.

Post to r/videoproduction: "We built an open-source programmatic video renderer with JSON format. For developers building automated video pipelines." — Audience-appropriate framing.

Post to r/rust and r/programming simultaneously is acceptable; cross-posting r/rust to r/programming is common for significant OSS launches.

**Hour 4: Developer Twitter**
Fire the prepared Twitter thread: "We shipped Mercury-Motion v1.0 today. [thread]" Walk through the benchmark, the JSON format, the binary size, the MIT license. End with the GitHub link and Product Hunt link.

**Ongoing through the day:**
- Reply to every comment on Product Hunt, HN, and Reddit personally. Do not use canned responses.
- Share notable comments (especially critical ones) publicly with honest engagement. "This person raised a fair point about X — here's our thinking" builds enormous credibility.
- Post a running "Launch Day" thread on Twitter updating benchmark reproduction attempts from the community.

**Newsletter outreach:**
The developer newsletter outreach should have been done 1–2 weeks prior (pre-launch). Target: TLDR Newsletter, Bytes.dev, The Changelog, Console.dev, Pointer.io, JavaScript Weekly (even though we are competing with JS — announce the alternative), Rust Weekly, This Week in Rust. Most newsletters need lead time. Do this early.

### 6.3 Post-Launch Momentum Plan

**Week 1 post-launch:**
- Publish a launch retrospective post: "Mercury-Motion Launch Day: Numbers, Feedback, and What's Next." Include HN points, GitHub stars, Discord joins, and a candid assessment of the feedback received.
- Address the top 3 criticisms from launch day feedback directly in the post.
- Ship a v1.0.1 patch addressing any critical bugs surfaced by the launch.

**Weeks 2–4:**
- "Most Requested Features" blog post based on GitHub issues and Discord feedback. This signals responsiveness.
- Begin the weekly render showcase series (Twitter Shorts + Discord).
- Ship one significant feature that was heavily requested at launch. The faster you close the gap between "launch promise" and "working feature," the faster trust compounds.

**Month 2:**
- Publish the first third-party benchmark (invite a known Rust community member or developer YouTuber to run the benchmark suite independently and publish their results).
- Launch the template gallery on the website. Seed with 20+ community templates.
- Announce the v1.1 roadmap publicly with a GitHub milestone.

**Month 3:**
- First conference talk submission accepted (RustConf, FOSDEM, or equivalent).
- Begin the comparison landing page strategy (see SEO section).
- Partner with one developer YouTuber for a sponsored tutorial (see Partnership section).

**Ongoing:**
- Monthly changelog video (rendered with Mercury-Motion).
- Weekly showcase.
- Biweekly blog post.
- Consistent reply to every GitHub issue and Discord message within 24 hours.

The primary post-launch metric is GitHub stars → Discord members → active weekly renders (community usage). The curve should not look like a Product Hunt spike that dies. It should look like a staircase: each launch-adjacent event (conference talk, major feature, press mention) lifts the floor.

---

## 7. Community Building

### 7.1 Discord Server Structure

The Mercury-Motion Discord is the primary community hub. It should be organized to serve both new users and active contributors without overwhelming either.

**Category: Getting Started**
- `#welcome` — automated welcome message with links to docs, examples, and the starter `.mmot.json`. Read-only.
- `#announcements` — official announcements only. Releases, blog posts, events. Slow mode: 1 message per day from maintainers only.
- `#introductions` — newcomers introduce themselves. Prompt: "What are you planning to build with Mercury-Motion?"

**Category: General**
- `#general` — open discussion, questions, off-topic
- `#help` — Q&A for users. Actively monitored. Template pinned: describe your `.mmot.json`, your command, and your error.
- `#show-and-tell` — share renders. Every post should include the source `.mmot.json` or a link to it.
- `#showcase` — curated renders selected by maintainers. High-quality bar. Being featured here is an honor.

**Category: Development**
- `#contributors` — for people actively contributing to Mercury-Motion. PRs, architecture discussions.
- `#json-schema-rfc` — ongoing discussion about the `.mmot.json` format evolution. Structured like an RFC process.
- `#renderer-internals` — deep technical discussion about the rendering pipeline, GPU compositing, benchmarks.
- `#editor` — discussion about the Tauri desktop editor specifically.

**Category: Integrations**
- `#lottie` — Lottie import/export discussion
- `#after-effects` — AE interop discussion
- `#ai-pipelines` — AI-generated video, LLM integration patterns
- `#social-presets` — platform presets (YouTube, TikTok, Instagram, etc.)

**Category: Meta**
- `#feedback` — product feedback, feature requests
- `#governance` — discussion about project direction, RFC process, release cadence. Transparent and public.

**Bot setup:**
- Auto-assign roles based on contribution level (GitHub PR merged → Contributor role)
- Weekly "Render of the Week" automated announcement
- `/benchmark` command that links to the public benchmark suite

### 7.2 Contributor Onboarding

The contributor experience is a product. Bad contributor experience is how open-source projects die.

**First contribution path:**
Every new contributor should be able to make a meaningful contribution within 30 minutes of cloning the repository. This means:
- `cargo build` works on the first try on all major platforms
- `cargo test` passes with no setup
- `CONTRIBUTING.md` is complete, accurate, and kind
- `good first issue` labels are maintained weekly — there should always be at least 10 tagged issues

**The `CONTRIBUTING.md` must include:**
- How to set up the development environment
- How to run the benchmark suite locally
- How the JSON schema is defined and validated
- How the renderer pipeline works (a one-page architecture overview)
- How to run the editor locally
- What "done" means for a PR (test coverage, docs, benchmark impact)
- How to propose a change to the `.mmot.json` format (RFC process)

**Recognition:**
- All contributors listed in `CONTRIBUTORS.md` (auto-generated from git log)
- First-time contributors get a personal "welcome" reply from a maintainer within 24 hours of their first PR
- Significant contributors get a Discord "Contributor" role and a mention in the monthly changelog
- "Founding Contributors" (merged PR before v1.0) get permanent recognition in the README

### 7.3 Ambassador / Advocate Program

**Who are Ambassadors?**
Community members who use Mercury-Motion publicly, create content about it, and help others. Not paid — but supported.

**What Ambassadors get:**
- `Ambassador` Discord role (visible, respected)
- Early access to new features (1–2 weeks before public release)
- Invitation to a private `#ambassador-council` Discord channel where roadmap decisions are discussed
- Co-authorship opportunities on blog posts
- Speaking introductions at events (a warm handoff, not a paid endorsement)

**How to become an Ambassador:**
Organic selection for the first 6 months — maintainers identify and invite people who are already contributing to the community. After 6 months, open an application process. Criteria: has published at least one public render, has helped at least 5 people in `#help`, is active in the community.

**Ambassador expectations:**
- Be honest about Mercury-Motion's trade-offs — ambassadors are not shills
- Be respectful of the community and of competitors
- Create at least one piece of public content per quarter (a render, a blog post, a tutorial)

### 7.4 Template Showcase & Gallery

The template gallery at `mercury-motion.dev/gallery` is a community-owned library of `.mmot.json` examples.

**Each gallery entry includes:**
- A rendered preview (MP4/GIF, rendered with Mercury-Motion, obviously)
- The source `.mmot.json` file
- Creator credit with a link to their GitHub/website
- A one-line description of what the template does
- Tags: `social`, `data-viz`, `code-explainer`, `product-demo`, `intro`, `outro`, `lower-third`, etc.
- A "Remix" button that opens the template in the online JSON editor

**Gallery governance:**
- Submitted via GitHub PR to a `gallery/` directory in the examples repository
- Maintainer review for quality and format compliance
- No commercial content in the gallery (no advertising for non-Mercury-Motion products)

**Featured templates:**
Maintainers select a "Template of the Week" featured prominently on the gallery homepage. Rotated every Monday. Announced on Discord and Twitter.

---

## 8. SEO & Discoverability

### 8.1 Target Keywords

**Primary keywords (high intent, moderate volume):**
- `programmatic video creation`
- `remotion alternative`
- `rust video renderer`
- `json video format`
- `automated video generation`
- `headless video renderer`

**Secondary keywords (moderate intent, broader):**
- `generate video from code`
- `video rendering CLI`
- `open source video creation`
- `video automation pipeline`
- `batch video generation`

**Long-tail keywords (lower volume, very high intent):**
- `render video without browser`
- `programmatic video without node.js`
- `gpu accelerated video rendering open source`
- `generate mp4 from json`
- `video creation single binary`
- `remotion too slow`
- `remotion rendering speed`
- `ffmpeg alternative video creation`

**Community/discovery keywords:**
- `rust video library`
- `wgpu video rendering`
- `tauri video editor`
- `mit license video renderer`

### 8.2 Landing Page Copy

**Hero Section:**

```
Mercury-Motion

Programmatic video at the speed of Rust.

Describe your video in a JSON file.
Run mmot render.
Get an MP4 in seconds, not minutes.

No browser. No Node.js. No dependencies.
One 30MB binary. MIT licensed.

[Get Early Access]  [View on GitHub]

"We rendered a 60-second 4K video in 4.2 seconds.
 The same scene took Remotion 5 minutes 18 seconds."
```

**Features Section:**

```
30–100x faster than browser-based renderers.
Benchmarked. Reproducible. Published.

One JSON file. One video.
The .mmot.json format is readable, diffable, and
writable by humans, scripts, and LLMs alike.

Zero dependencies.
The mmot binary is ~30MB. It requires nothing else.
No Node.js. No Chrome. No npm install. No prayers.

Deterministic.
The same JSON always produces the same MP4.
Version-control your videos. Test your templates.

MIT licensed. Forever.
Not open core. Not a free tier. MIT.
Use it, fork it, ship it. No strings.

GPU-accelerated.
Leverages your GPU for compositing and rendering.
Feed it more cores; watch the render time drop.
```

**How It Works Section:**

```
1. Write your video.
   A .mmot.json file describes your layers, keyframes,
   compositions, and props. It's plain JSON.
   Your editor understands it. Git understands it. GPT-4 understands it.

2. Run the renderer.
   $ mmot render my-video.mmot.json
   Rendering... ████████████████████ 100%
   Output: my-video.mp4 (rendered in 3.1s)

3. Get your MP4.
   A standard MP4. Works everywhere.
   H.264, H.265, VP9, AV1 — your choice.
```

**Social Proof Section:**

```
"Finally, a video renderer I can actually put in a CI pipeline."

"Watched the side-by-side benchmark three times.
 Still can't believe the render time difference."

"The JSON format is genius. I had GPT-4 generate a
 .mmot.json from a product description in 30 seconds."
```

**CTA Section:**

```
Start rendering in 60 seconds.

$ curl -fsSL https://mercury-motion.dev/install.sh | sh
$ echo '{"duration": 5, "fps": 30, "layers": [...]}' > hello.mmot.json
$ mmot render hello.mmot.json
→ hello.mp4

[Read the Docs]  [See Examples]  [Join Discord]
```

### 8.3 Comparison Pages Strategy

Dedicated comparison pages capture developers who are actively evaluating tools. These pages should be honest — presenting real trade-offs — because dishonest comparison pages are spotted immediately and destroy credibility.

**Target comparison pages:**

**`mercury-motion.dev/vs/remotion`**
The most important comparison page. Covers: architecture, rendering speed (with benchmarks), dependency footprint, format (JSON vs. JSX), license, editor, community size. Ends with an honest "When to choose Remotion" section — this counterintuitively builds trust.

**`mercury-motion.dev/vs/creatomate`**
SaaS vs. self-hosted. API latency vs. zero latency. Per-render pricing vs. own your binary. Privacy. Offline use.

**`mercury-motion.dev/vs/ffmpeg`**
FFmpeg is not a creative video tool but many developers use it for automated video tasks. Position Mercury-Motion as "FFmpeg for creative, layered, keyframed video" — higher-level, JSON-driven, but equally CLI-native.

**`mercury-motion.dev/vs/after-effects`**
Not really a comparison (different audiences) but captures SEO for "programmatic after effects alternative." Frame as: "AE for motion design; Mercury-Motion for automated rendering pipelines."

**Page structure for each comparison:**

1. One-paragraph honest summary of the competitor
2. Feature comparison table (neutral language, accurate)
3. Rendering speed comparison (with link to reproducible benchmark)
4. "Mercury-Motion is a better choice when..." (specific conditions)
5. "[Competitor] is a better choice when..." (honest, specific conditions)
6. Try Mercury-Motion section (install command)

**SEO requirements for comparison pages:**
- H1 includes both brand names
- Schema markup for comparison (FAQPage or Product schema)
- Internal links to relevant documentation sections
- Benchmark numbers in the meta description

---

## 9. Partnership Opportunities

### 9.1 Developer YouTubers

Developer YouTubers with audiences interested in tooling, Rust, or creative coding are high-priority partners. The ideal partner has:
- 50k–500k subscribers (micro to mid-tier — more accessible, higher engagement rate)
- Audience that overlaps with developer-creators
- History of honest tool reviews (not just sponsor content)
- Interest in Rust, systems programming, or creative coding

**Priority targets:**

*Tier 1 — Ideal fit:*
- **Theo (t3.gg)** — Large audience of TypeScript developers. Covering a Rust tool that challenges JS ecosystem assumptions is exactly his content.
- **ThePrimeagen** — Rust-positive, large developer audience, known for performance-focused takes. Perfect alignment.
- **No Boilerplate** — Rust-specific YouTube channel. Smaller audience but extremely targeted. High conversion likelihood.
- **Low Level TV** / **Low Level Learning** — Systems programming audience. Mercury-Motion's GPU rendering internals are content gold for this audience.

*Tier 2 — Strong fit:*
- **Fireship** — Short-form tech explainers. A 100-second "Mercury-Motion in 100 seconds" video would perform very well for both parties.
- **Traversy Media** — Tutorial-focused. A "Build a programmatic video with Mercury-Motion" tutorial would serve his audience.
- **Kevin Powell** — CSS/creative coding audience. Adjacent but potentially interested in the creative video creation angle.

**Partnership structure:**
For the first 6 months: offer free product access and a detailed technical briefing. No payment required. If they want to make content, support them with assets, benchmark data, and a technical contact.

After 6 months, once the product is stable and has demonstrated value: paid sponsorships for tutorial-style content. Budget: $2,000–$8,000 per sponsored video depending on audience size. Require that sponsored content clearly discloses the relationship and allows the creator full editorial control.

### 9.2 Newsletter Sponsorships

Developer newsletters are the most efficient paid channel for developer tools. They reach an audience that is actively seeking tools and is accustomed to paying attention.

**Target newsletters:**

| Newsletter | Audience | Typical CPM | Priority |
|---|---|---|---|
| TLDR Newsletter (Dev edition) | 750k+ developers | ~$50 CPM | High |
| Bytes.dev | React/JS developers | ~$40 CPM | Medium (cross-sell) |
| This Week in Rust | Rust developers | ~$30 CPM | High |
| The Changelog Weekly | OSS developers | ~$45 CPM | High |
| Console.dev | Developer tool aficionados | ~$60 CPM | High |
| Pointer.io | Engineering leaders | ~$55 CPM | Medium |
| TLDR AI | AI developers | ~$50 CPM | Medium |

**Newsletter ad copy guidelines:**
- Lead with the benchmark number. Don't bury the lead.
- Include the install command in the ad copy. Yes, in a newsletter ad. Developers want to see that it's simple before they click.
- Clearly state "MIT licensed, free forever" — this is a major anxiety reducer for tool adoption.
- Link directly to a landing page variant with newsletter-specific UTM tracking.

**Budget recommendation:**
Allocate 60% of paid marketing budget to newsletter sponsorships in the first 6 months. This is the highest-converting channel for OSS developer tools.

### 9.3 Integration Partners

Integration partnerships serve two purposes: extend Mercury-Motion's capabilities and access the partner's user base.

**LottieFiles**
Lottie is the standard format for web animations. A Mercury-Motion integration that accepts Lottie files as layers would be a significant feature for motion designers who want to automate their workflow. LottieFiles has a large designer community — a co-marketing post ("Bring Your Lottie Animations to Video with Mercury-Motion") would reach a new audience.

*Partnership form:* Technical integration + co-marketing blog post. No revenue share required.

**Figma**
Figma Plugin that exports a selected frame or component as a `.mmot.json` layer. This brings Mercury-Motion to every designer who has a developer working alongside them. The Figma plugin ecosystem is a significant distribution channel.

*Partnership form:* Build the plugin (or support community member building it). Submit to Figma Community. Reach out to Figma's partnerships team for potential developer spotlight.

**ElevenLabs**
ElevenLabs is the leading AI voice synthesis platform. An integration where a `.mmot.json` can reference an ElevenLabs voice ID and script, and Mercury-Motion will call the API and composite the audio automatically, would make Mercury-Motion dramatically more powerful for AI video pipelines.

*Partnership form:* Technical integration + co-marketing. Potential for mutual newsletter mentions and joint blog post ("Build an AI Narrator Video Pipeline with ElevenLabs + Mercury-Motion").

**Google Fonts**
Mercury-Motion should support Google Fonts natively — reference a font by name in the JSON and Mercury-Motion fetches it. This is a developer quality-of-life feature that also serves as a partnership signal.

*Partnership form:* Technical integration. No formal partnership required — Google Fonts is an open API. Mention in documentation.

**Unsplash / Pexels**
Stock photo/video integration: reference an Unsplash photo ID in your `.mmot.json` as an image layer source. Mercury-Motion fetches it automatically (with API key). This dramatically lowers the barrier to creating beautiful videos without assets.

*Partnership form:* Apply for API partnerships with both platforms. Co-marketing opportunity for both (their content, in motion, powered by Mercury-Motion).

**GitHub Actions Marketplace**
Publish an official `mercury-motion/render-action` GitHub Action. This is not a partnership — it is a distribution channel. Developers who discover Mercury-Motion through a GitHub Action in someone else's workflow are highly qualified leads.

---

## 10. Metrics & KPIs

### 10.1 Phase 1: Pre-Launch (Months –3 to 0)

**Goal:** Build awareness and a warm audience before launch.

| Metric | Target | Measurement |
|---|---|---|
| Waitlist signups | 2,000 emails | Email service provider |
| Twitter/X followers | 1,500 | Twitter Analytics |
| GitHub stars (preview repo) | 500 | GitHub |
| Discord early access members | 300 | Discord |
| Blog post unique visitors | 5,000/month | Plausible Analytics |
| Newsletter inclusions | 3 mentions in major newsletters | Manual tracking |

**What success looks like:** The waitlist is growing organically (not just from paid promotion). At least 50 people have reproduced the benchmark and posted about it publicly. The GitHub repo is being watched and starred by people we did not personally invite.

### 10.2 Phase 2: Launch (Week of Launch)

| Metric | Target | Notes |
|---|---|---|
| Product Hunt ranking | Top 5 on launch day | |
| Hacker News front page | Yes (>100 points) | |
| GitHub stars in 24 hours | 500+ | |
| GitHub stars in 7 days | 1,500+ | |
| Discord members (launch week) | 500+ | |
| npm/cargo downloads | 1,000+ installs | `cargo install mmot` |
| Press mentions | 5+ developer newsletters | |
| Reddit post upvotes | 500+ combined across subreddits | |

**What success looks like:** The Hacker News post stays on the front page for at least 4 hours. The GitHub star velocity in the first 48 hours exceeds 200 stars/day. At least one significant developer with a public audience posts about Mercury-Motion without being asked.

### 10.3 Phase 3: Growth (Months 1–6)

| Metric | Target | Cadence |
|---|---|---|
| GitHub stars | 5,000 | End of Month 3 |
| GitHub stars | 10,000 | End of Month 6 |
| Discord members | 2,000 | End of Month 3 |
| Discord DAU | 200 | End of Month 3 |
| Monthly binary downloads | 5,000 | End of Month 3 |
| Monthly active renders (community) | 10,000 | End of Month 6 |
| Gallery templates | 100 | End of Month 6 |
| External blog posts about Mercury-Motion | 20 | End of Month 6 |
| Open source contributors | 50 | End of Month 6 |
| YouTube/video tutorials (external) | 5 | End of Month 6 |

**What success looks like:** The project has momentum that does not require constant active promotion. New GitHub issues are being opened daily by users the maintainers have never heard of. The Discord has conversations happening that do not involve the maintainers. At least one company has publicly stated they are using Mercury-Motion in a production pipeline.

### 10.4 Phase 4: Scale (Months 6–18)

| Metric | Target | Notes |
|---|---|---|
| GitHub stars | 25,000+ | |
| Monthly active CLI users | 25,000+ | Estimated from download + usage telemetry (opt-in) |
| Companies using in production | 50+ (publicly known) | |
| Community-contributed templates | 500+ | |
| Contributors (any size PR merged) | 200+ | |
| Conference talks accepted | 5+ | RustConf, FOSDEM, etc. |
| Integration plugins shipped | 6+ | Figma, Lottie, ElevenLabs, etc. |
| Monthly organic search traffic | 50,000 visits | Plausible |
| Top 3 ranking for primary keywords | Yes | Google Search Console |

**What success looks like:** Mercury-Motion is the default answer in developer communities when someone asks "how do I generate video programmatically?" The project can sustain itself even if the founding maintainer takes a month off — the community keeps it alive.

### 10.5 North Star Metric

**Weekly active renders from distinct users.**

This single metric captures everything important: the tool has to be installed (binary downloads don't lie), it has to work (broken tools don't get used), and users have to have a reason to keep coming back (only useful tools get weekly use). All marketing and product decisions should be evaluated against their likely impact on this metric.

### 10.6 Metrics to Explicitly De-prioritize

**Website pageviews (vanity):** Traffic without usage is noise. A Hacker News post can drive 50,000 sessions in a day that convert to nothing.

**Twitter impressions:** Reach without engagement is irrelevant. Track replies, follows, and links clicked — not impressions.

**GitHub stars as a vanity metric:** Stars matter for social proof but not for sustainability. A project with 20,000 stars and 50 weekly users is less healthy than one with 2,000 stars and 5,000 weekly users.

**Press mentions without attribution:** A mention in a newsletter only matters if it drives signups or stars. Use UTM parameters on every link. Track conversion, not coverage.

---

*Mercury-Motion Marketing Strategy v1.0*
*For internal use and community transparency.*
*Mercury-Motion is MIT licensed. So is this document — use it freely.*
