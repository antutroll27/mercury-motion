# Mercury-Motion — Skills & Agents Master Reference
**Date:** 2026-03-23
**Status:** Living document — update as new skills are discovered or installed

Everything we need to build Mercury-Motion, organised by priority and area.

---

## Already Installed

| Skill | What it covers |
|---|---|
| `superpowers:*` suite | Planning, TDD, debugging, subagents, git worktrees, verification, code review |
| `actionbook/rust-skills` | Rust meta-cognition (38 skills), `/sync-crate-skills` for niche crates |
| `figma:implement-design` | Figma → production React code |
| `frontend-design` | Production-grade frontend UI quality |
| `document-skills:*` | PDF, DOCX, PPTX, XLSX, webapp-testing, etc. |
| `cloudflare:*` | Workers, Durable Objects, MCP, Agents SDK |

---

## Install Immediately — Highest ROI

### 1. `dchuk/claude-code-tauri-skills`
**GitHub:** https://github.com/dchuk/claude-code-tauri-skills
**Relevance: 10/10**

39 skills covering the entire Tauri v2 development lifecycle. The single most impactful find for Mercury-Motion's desktop editor.

| Category | Skills |
|---|---|
| IPC | `tauri-ipc`, `tauri-calling-rust`, `tauri-calling-frontend`, `tauri-frontend-events`, `tauri-frontend-js`, `tauri-frontend-rust` |
| Security | `tauri-permissions`, `tauri-capabilities`, `tauri-csp`, `tauri-runtime-authority`, `tauri-scope` |
| Distribution | `tauri-windows-distribution`, `tauri-macos-distribution`, `tauri-linux-packaging`, `tauri-code-signing`, `tauri-crabnebula` |
| Development | `tauri-project-setup`, `tauri-architecture`, `tauri-plugins`, `tauri-testing`, `tauri-debugging`, `tauri-binary-size` |
| UI | `tauri-window-customization`, `tauri-system-tray`, `tauri-splashscreen`, `tauri-sidecar` |
| CI | `tauri-pipeline-github` |

---

### 2. `awesome-skills/code-review-skill`
**GitHub:** https://github.com/awesome-skills/code-review-skill
**Relevance: 9/10**

9,500+ line code review skill. The Rust-specific rules (`reference/rust.md`) are exactly what Mercury-Motion needs:
- Every `clone()` must be justified
- `Arc<Mutex<T>>` → push toward single-ownership or `DashMap`
- `Cow<T>` preference to avoid allocations
- **Unsafe blocks:** mandatory `// SAFETY:` comments (critical for `skia-safe` + `ffmpeg-next`)
- Async: no blocking ops in async context, `tokio::sync::Mutex` for async guards, `join!`/`try_join!`
- Zero clippy warnings, `must_use` enforcement

---

### 3. `londey/claude-skill-rust`
**GitHub:** https://github.com/londey/claude-skill-rust
**Relevance: 8/10**

Auto-activates on `/rust`. Enforces the full Rust quality gate sequence:
`rustfmt → clippy (pedantic) → test → build → doc → cargo-deny → cargo-audit`

Includes `thiserror` (lib) / `anyhow` (app) split enforcement — matches Mercury-Motion's error strategy exactly.

---

### 4. `kaivyy/perseus`
**GitHub:** https://github.com/kaivyy/perseus
**Relevance: 8/10**

8 security deep-dive skills. Rust is a first-class supported language (Actix-web, Axum, Rocket, Warp). The Supply Chain skill covers CVEs across 8 package managers including Cargo — typosquatting, dependency confusion, transitive vulnerabilities. Critical for an open-source project that will have many contributors.

---

### 5. `trailofbits/skills`
**GitHub:** https://github.com/trailofbits/skills
**Relevance: 8/10**

Professional security skills from Trail of Bits:
- `zeroize-audit` — Rust secret zeroization (relevant if FilmTools handles any auth/API keys)
- `property-based-testing` — covers Rust (proptest, quickcheck)
- `constant-time-analysis` — cryptographic timing side-channels
- Differential code review, static analysis with CodeQL/Semgrep

---

### 6. `existential-birds/beagle`
**GitHub:** https://github.com/existential-birds/beagle
**Relevance: 8/10**

92 skills across 11 language plugins. Key for Mercury-Motion:
- `beagle-rust` (7 skills): tokio, serde, Rust review via `/beagle-rust:review-rust`
- Release notes generation
- ADR (Architecture Decision Record) generation
- Test plan generation
- Multi-language review (Rust backend + TypeScript frontend in same session)

---

### 7. `dwalleck/catalyst`
**GitHub:** https://github.com/dwalleck/catalyst
**Relevance: 8/10**

Rust binary (~2ms startup) that auto-activates skills based on file context. Reads `skill-rules.json`, matches keywords/file path patterns/intent patterns/file content. As Mercury-Motion accumulates skills across Rust/Tauri/WASM/React, Catalyst eliminates manual `/skill` invocation. The right skill loads automatically when you open the right file.

---

## Install for Frontend Work

### 8. `masuP9/a11y-specialist-skills`
**GitHub:** https://github.com/masuP9/a11y-specialist-skills
**Relevance: 7/10**

4 skills for WCAG 2.2 compliance. Mercury-Motion's editor UI needs proper accessibility for professional users:
- `reviewing-a11y` — evaluates components against WCAG + WAI-ARIA
- `auditing-wcag` — systematic testing (axe-core + Playwright automated, keyboard/focus, manual)
- `planning-wcag-audit` — audit strategy
- `planning-a11y-improvement` — organisational maturity roadmap

---

### 9. `EastonShin/aria-apg-patterns`
**GitHub:** https://github.com/EastonShin/aria-apg-patterns
**Relevance: 7/10**

26 ARIA interaction patterns Mercury-Motion's editor will use:
- **Slider** → timeline scrubber, keyframe drag handles
- **Tree View** → layer/track list
- **Grid** → keyframe timeline grid
- **Dialog** → export modal, settings
- **Tabs** → editor / FilmTools page switch
- **Tooltip** → property hints

Complete key mappings, ARIA roles/states/properties, focus trap patterns.

---

### 10. `jezweb/claude-skills`
**GitHub:** https://github.com/jezweb/claude-skills
**Relevance: 6/10**

59 skills across 10 plugins. Relevant:
- `react-patterns` — React 19 performance/composition patterns
- `shadcn-ui` — component patterns (Mercury-Motion editor uses shadcn/ui)
- `design-review` — UI/UX review
- `project-docs` — architecture documentation generation

---

### 11. `daymade/claude-code-skills` (i18n-expert)
**GitHub:** https://github.com/daymade/claude-code-skills
**Relevance: 7/10** (if international release planned)

React/Next.js i18n: `t('namespace.key')` string extraction, pluralisation, Intl formatters, locale parity enforcement (en-US + zh-CN default), Python audit scripts for missing key detection. Plan for this from the start — retrofitting i18n is painful.

---

## Install for CI/CD

### 12. `akin-ozer/cc-devops-skills`
**GitHub:** https://github.com/akin-ozer/cc-devops-skills
**Relevance: 7/10**

31 skills across 5 categories. Key for Mercury-Motion:
- `github-actions-generator` + `github-actions-validator` — build Mercury-Motion CI matrix (Windows/macOS/Linux, Rust + WASM + frontend)
- `github-actions-validator` uses `actionlint` + `act` for local testing
- Bash + Makefile generator for build scripts

Note: No Rust/Cargo-specific templates — will need to prompt it to scaffold Rust-aware workflows.

---

### 13. `altinukshini/claude-code-pr-reviewer`
**GitHub:** https://github.com/altinukshini/claude-code-pr-reviewer
**Relevance: 7/10**

GitHub Action wrapping Claude Code. Triggers on `ai-review` label. Reviews:
- Architecture/SOLID principles
- OWASP Top 10 security
- Code quality + test coverage
- Posts severity-rated comments (P0–P3)

Language-agnostic — works on Mercury-Motion's Rust + TypeScript mixed PRs.

---

### 14. `steeef/claude-skill-github-actions`
**GitHub:** https://github.com/steeef/claude-skill-github-actions
**Relevance: 6/10**

GitHub Actions **troubleshooting** — fetches logs by commit SHA/branch, identifies error patterns, generates fix recommendations via `gh` CLI. For diagnosing failing Mercury-Motion CI runs.

---

## Install for ML/ONNX Work (FilmTools AI)

### 15. `ortizeg/whet`
**GitHub:** https://github.com/ortizeg/whet
**Relevance: 6/10**

30 skills for CV/ML production: PyTorch Lightning, ONNX, TensorRT, OpenCV, FastAPI, Hugging Face, Docker. Six specialist agent personas including ML Engineer. The ONNX inference service archetype maps to Mercury-Motion's FilmTools AI pipeline (SAM2, Depth Anything, TransNetV2).

---

### 16. `huggingface/skills`
**GitHub:** https://github.com/huggingface/skills
**Relevance: 6/10**

- `transformers-js` — WebGPU + WASM backends for browser-side ML inference. Relevant if Mercury-Motion's browser build runs depth estimation or scene detection in-browser.
- `hugging-face-vision-trainer` — object detection + image classification model training. Useful if Mercury-Motion builds custom motion analysis models.

---

## Install for Docs & Architecture

### 17. `pranavred/claude-code-documentation-skill`
**GitHub:** https://github.com/pranavred/claude-code-documentation-skill
**Relevance: 6/10**

Code → Mermaid diagrams. Selects diagram type by content: sequence (Tauri IPC), class (renderer architecture), state (editor lifecycle), C4 (system architecture). Useful for keeping Mercury-Motion's architecture docs current.

---

### 18. `aspenkit/aspens`
**GitHub:** https://github.com/aspenkit/aspens
**Relevance: 7/10**

Scans repo, builds import graph (JS/TS), identifies hub files, auto-generates project-specific SKILL.md files (~35 lines each) via parallel agents. Git hook keeps context current. Point at Mercury-Motion's TypeScript frontend → get auto-generated skills for the editor codebase.

---

### 19. `levnikolaevich/claude-code-skills` (codebase-audit-suite)
**GitHub:** https://github.com/levnikolaevich/claude-code-skills
**Relevance: 7/10**

127 skills across 7 plugins. Key:
- `codebase-audit-suite` — security audit, build audit, code quality, test coverage, architecture audit (4-score model), performance audit
- `documentation-pipeline` — auto-detects backend/frontend/devops context, generates project docs

---

## WASM Skills

### 20. `derushio/wasm-skills`
**GitHub:** https://github.com/derushio/wasm-skills
**Relevance: 7/10**

Generic WASM patterns skill covers: Webpack dynamic imports, TypedArray memory management, Web Worker offloading, SSR avoidance, React state machines for WASM lifecycle. Directly applicable to Mercury-Motion's wasm-pack Web Worker architecture.

---

## Rust & FFmpeg (Agent 2 Finds)

### 21. `skill-rust-ffmpeg`
**Relevance: 9/10**

54,000-word Rust FFmpeg knowledge base — the most comprehensive Claude Code resource for audio/video processing in Rust. Covers:
- `ez-ffmpeg` — high-level ergonomic wrapper over `ffmpeg-next`
- `ffmpeg-next` — full FFI bindings, direct API access
- `ffmpeg-sys-next` — raw `libav*` FFI for low-level control
- Demuxing, decoding, encoding, muxing pipelines
- Hardware acceleration (VAAPI, NVENC, VideoToolbox)
- Audio filtering, format conversion, stream copying

Critical for Mercury-Motion's `--features ffmpeg` extended codec path (HEVC, ProRes, multi-track audio). The `/sync-crate-skills` command from `actionbook/rust-skills` should also auto-generate narrower ffmpeg-next skills from crate docs.

---

### 22. `onsails/cc` (rust-dev plugin)
**Relevance: 8/10**

Opinionated Rust development standards skill:
- **Edition 2024 enforcement** — all new crates must use `edition = "2024"` in Cargo.toml
- **FAIL FAST principle** — no silent failures; propagate errors up, never swallow them
- **Workspace templates** — Cargo workspace scaffolding with correct inter-crate `path` dependencies
- **`rust-builder` sub-agent** — dedicated agent that iterates `cargo build` until clean (no warnings, no clippy errors)

The FAIL FAST principle aligns directly with Mercury-Motion's `thiserror` (lib) / `anyhow` (app) error strategy. The workspace template covers the exact `crates/mercury-core`, `crates/mercury-filmtools`, `crates/mercury-cli` structure.

---

### 23. `rust-skill-creator`
**Relevance: 7/10**

Bootstraps new custom Rust Claude Code skills from scratch. Takes a crate name or topic, fetches docs, generates a structured skill file. Critical for building the custom skills Mercury-Motion needs that don't exist anywhere:

Use this to create: `/wgpu`, `/rust-profiling`, `/tauri-wasm`, `/film-emulation`, `/onnx-rust`

---

## CI/CD — Additional Finds

### 24. `cc-github-skills`
**Relevance: 7/10**

GitHub Actions matrix build authoring — specifically covers multi-platform matrix strategies (Windows/macOS/Linux × architecture × feature flags). More Rust-aware than `akin-ozer/cc-devops-skills`. Pair both: `akin-ozer` for generation, `cc-github-skills` for Rust-specific matrix patterns.

---

### 25. `claude-git-pr-skill`
**Relevance: 7/10**

PR review workflow with:
- **Approval gates** — blocks merge until checklist items are resolved
- **Inline suggestions** — generates specific code fix suggestions (not just comments)
- Semantic versioning enforcement on PR titles
- Changelog entry generation per PR

Complements `altinukshini/claude-code-pr-reviewer` (which runs as a GitHub Action). This one runs interactively during development.

---

## Changelog & Release Automation

### 26. Changelog Skills (Three Options)

| Skill | Approach | Best For |
|---|---|---|
| `sipaan/claude-changelog-skill` | Conventional Commits → CHANGELOG.md | Standard Keep-a-Changelog format |
| `showkkd133/changelog-gen-skill` | Git log analysis → categorised entries | Automated weekly changelog generation |
| `BenedictKing/codex-review` | PR review + changelog in one pass | Pre-merge quality gate with release notes |

Mercury-Motion recommendation: `sipaan` for conventional commits enforcement during development; `BenedictKing/codex-review` pre-merge. All three are lightweight installs.

---

## Performance & Specialised

### 27. `Claude-Code-Game-Studios`
**Relevance: 5/10**

Game engine development standards — partially applicable to Mercury-Motion's render loop:
- **`perf-profile` skill** — structured profiling workflow (instrument → profile → analyse → fix)
- **Zero-alloc hot path standards** — no allocations inside frame render loop
- Frame budget enforcement patterns

The zero-alloc standard is directly applicable to Mercury-Motion's render thread (every allocation inside `render_frame()` is a potential stutter).

---

### 28. `visual-programming-cc-skill`
**Relevance: 5/10**

Visual render validation loop — takes a screenshot of a rendered output and runs an automated visual diff against a golden reference. Relevant for Mercury-Motion's golden image test suite (pixel-exact comparison across platforms). Could replace manual golden image management.

---

## Confirmed Gaps — Custom Skills to Build

No community skill exists for these. Mercury-Motion should build them using `superpowers:writing-skills` + `rust-skill-creator` (skill #23 above).

**Gap priority scores (from ecosystem-wide search):**

| Custom Skill | Gap Severity | What it covers | Priority |
|---|---|---|---|
| `/color-science` | **10/10 — zero skills anywhere** | OCIO 2.5 C FFI patterns, ACES pipeline, log curve math, LUT application in wgpu, `palette` crate color space conversions | **Critical** |
| `/wgpu` | **9/10 — zero skills anywhere** | WGSL shader authoring, render pipeline setup, compute shaders, wgpu-rs API patterns, buffer management, texture sampling | **Critical** |
| `/rust-profiling` | **8/10** | criterion benchmark workflow, cargo-flamegraph, perf annotation, identifying hot paths in render loop | **High** |
| `/rust-docs` | **8/10** | `cargo doc` best practices, doc-test patterns, `#[doc(hidden)]` usage, publishing docs to docs.rs | **High** |
| `/tauri-wasm` | **7/10** | wasm-pack output consumed by Tauri, JS bindings via wasm-bindgen, memory management across IPC/WASM boundary | **High** |
| `/film-emulation` | **7/10** | agx-emulsion algorithm porting guide, Newson grain implementation in wgpu compute shaders, OCIO FFI patterns | **High** |
| `/cargo-audit-deny` | **7/10** | `cargo-audit` + `cargo-deny` workflow, advisory database, license policy enforcement | **Medium** |
| `/react-timeline-ui` | **7/10** | Timeline editor patterns (virtual scrolling, drag-to-scrub, keyframe handles), React 19 + Canvas2D interop | **Medium** |
| `/multi-crate-release` | **7/10** | Releasing a Cargo workspace with interdependent crates in the correct order, version bumping, GitHub Releases | **Medium** |
| `/audio-dsp` | **6/10** | Real-time audio processing in Rust — CPAL (cross-platform audio), Symphonia (decoding), dasp (DSP primitives), RNNoise integration | **Medium** |
| `/onnx-rust` | **6/10** | ONNX Runtime Rust bindings (`ort` crate), model loading, inference batching, WASM-compatible model selection | **Medium** |
| `/cargo-release` | **6/10** | Semantic versioning, changelog generation, cargo-release workflow, cross-platform binary publishing | **Medium** |

---

## Skills Aggregators to Monitor

| Resource | What to watch for |
|---|---|
| [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) | General workflow tools, new Rust entries |
| [travisvn/awesome-claude-skills](https://github.com/travisvn/awesome-claude-skills) | Frontend/React skills |
| [VoltAgent/awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills) | Broad coverage, 500+ skills |
| [jeremylongshore/claude-code-plugins-plus-skills](https://github.com/jeremylongshore/claude-code-plugins-plus-skills) | 1,367 skills — most comprehensive |
| [agentskills.in](https://agentskills.in) | Marketplace, searchable |
| [buildwithclaude.com/plugins](https://buildwithclaude.com/plugins) | Plugin registry |

---

## Skills Installation Priority Order

```
Phase 1 — Before writing any code:
  dchuk/claude-code-tauri-skills          (Tauri IPC + distribution)
  londey/claude-skill-rust                (Rust quality gates)
  awesome-skills/code-review-skill        (Deep Rust review)
  onsails/cc (rust-dev)                   (Edition 2024, FAIL FAST, workspace scaffold)
  dwalleck/catalyst                       (Auto-activate skills)

Phase 2 — During core engine development:
  actionbook/rust-skills + /sync-crate-skills  (already installed; run sync)
  skill-rust-ffmpeg                       (54K-word FFmpeg/Rust knowledge base)
  existential-birds/beagle (beagle-rust)
  kaivyy/perseus                          (supply chain security)
  trailofbits/skills                      (property-based testing, zeroize)
  rust-skill-creator                      (bootstrap custom /wgpu, /film-emulation skills)

Phase 3 — During editor development:
  derushio/wasm-skills                    (WASM patterns)
  jezweb/claude-skills                    (React 19 patterns)
  masuP9/a11y-specialist-skills           (WCAG audit)
  EastonShin/aria-apg-patterns            (ARIA for editor components)
  aspenkit/aspens                         (auto-generate frontend skills)

Phase 4 — Before first release:
  akin-ozer/cc-devops-skills              (GitHub Actions CI generation)
  cc-github-skills                        (Rust multi-platform matrix patterns)
  altinukshini/claude-code-pr-reviewer    (PR review GitHub Action)
  claude-git-pr-skill                     (Interactive PR review + approval gates)
  sipaan/claude-changelog-skill           (Conventional Commits → CHANGELOG.md)
  pranavred/claude-code-documentation-skill (architecture diagrams)

Phase 5 — FilmTools AI pipeline:
  ortizeg/whet                            (ONNX inference)
  huggingface/skills                      (transformers-js WebGPU)

Build custom skills (use rust-skill-creator to bootstrap):
  /color-science  ← HIGHEST PRIORITY: zero skills exist anywhere for OCIO/ACES
  /wgpu           ← WGSL shaders, render pipeline, compute shaders
  /film-emulation ← agx-emulsion port, Newson grain in wgpu
  /rust-profiling ← criterion, flamegraph, hot path analysis
  /tauri-wasm     ← wasm-pack + Tauri IPC boundary
  /audio-dsp      ← CPAL, Symphonia, RNNoise
  /multi-crate-release ← workspace versioning + GitHub Releases
```

---

*Update this document as skills are installed, deprecated, or superseded.*
