# Mercury-Motion FilmTools — Key Research Breakthroughs
**Date:** 2026-03-22
**Status:** Reference document — cool stuff worth knowing

These are the finds from deep research that fundamentally changed what we thought was possible for Mercury-Motion FilmTools. Each one solves a problem we thought would require either proprietary SDKs, C++ FFI nightmares, or compromises. They didn't.

---

## Breakthrough 1: Gyroflow Core — Pure Rust Stabilization

**GitHub:** https://github.com/gyroflow/gyroflow
**Author:** AdrianEddy
**Language:** Core library = pure Rust + wgpu
**License:** GPL v3

We thought video stabilization in Rust meant wrapping vid.stab (a C library via FFmpeg). It doesn't.

The **Gyroflow desktop application** — the most sophisticated open-source stabilization tool that exists — runs on a core library that is **pure Rust**. Embeddable. No Qt, no FFmpeg required in the core. It uses `wgpu` for GPU processing (the same backend Mercury-Motion already uses) and handles:

- Gyroscope-data-driven lens undistortion + stabilization
- Optical flow
- GPU-accelerated processing (Vulkan/Metal/DX12/OpenGL via wgpu)
- Keyframe smoothing
- Lens distortion correction and re-application
- 10–16 bit video support
- Lens profiles for GoPro HERO 6–13, Sony, DJI, Insta360, RunCam (community-extensible)

**Why gyro-assisted beats optical-flow-only:**
Optical flow guesses motion from pixels. Gyroscopic data *measures* motion directly from the camera's IMU. The result is stabilization that works correctly even during rapid motion, in dark scenes, or with repetitive textures where optical flow gets confused. This is why DJI and GoPro's in-camera stabilization feels so smooth — they have the gyro data.

**Companion crate:** `telemetry-parser` ([crates.io/crates/telemetry-parser](https://crates.io/crates/telemetry-parser)) — also by AdrianEddy. Parses camera motion metadata embedded in video files: Sony, GoPro GPMF, Insta360, DJI, Betaflight. This is what feeds Gyroflow its gyro data.

**What this means for FilmTools:** We get the most sophisticated open-source stabilization library available, and it's native Rust, and it already uses our GPU backend. We don't write a stabilizer — we embed Gyroflow Core.

---

## Breakthrough 2: `gpu-video` — BRAW and R3D Decoded in Rust

**GitHub:** https://github.com/AdrianEddy/gpu-video
**Author:** AdrianEddy (same person as Gyroflow)
**Language:** Rust

We thought decoding Blackmagic RAW (BRAW) and RED RAW (R3D) required proprietary SDKs. You had to sign agreements with Blackmagic Design and RED Digital Cinema. Most open-source tools simply don't support these formats.

AdrianEddy built **GPU-accelerated video decoding AND encoding** in Rust that handles BRAW and R3D natively — no proprietary SDK required.

Combined with `rawler` (Canon/Nikon/Sony/Fuji stills RAW) and `dng-rs` (Cinema DNG), this closes the last major professional camera RAW format gaps:

| Format | Tool | Notes |
|---|---|---|
| Canon CR2/CR3, Nikon NEF, Sony ARW, Fuji RAF | `rawler` | LGPL, alpha but active |
| Cinema DNG | `dng-rs` | Pure Rust, from AXIOM open cinema camera |
| Blackmagic RAW (BRAW) | `gpu-video` | GPU-accelerated, pure Rust |
| RED RAW (R3D) | `gpu-video` | GPU-accelerated, pure Rust |

**What this means for FilmTools:** FilmTools can ingest footage directly from the professional cameras filmmakers actually use — ARRI, RED, Blackmagic — without requiring users to first export to a different format. This is a significant workflow advantage over tools that only accept ProRes/H.264.

---

## Breakthrough 3: `agx-emulsion` — Spectral Film from Manufacturer Datasheets

**GitHub:** https://github.com/andreavolpato/agx-emulsion
**Author:** Andrea Volpato
**What it is:** The most physically rigorous open-source film simulation project in existence

We planned to simulate film with S-curves and noise. That's what most tools do. `agx-emulsion` goes much deeper.

It uses **published datasheets from Kodak and Fujifilm** — actual spectral sensitivity measurements, dye transmittance data, emulsion characteristic curves — to build a complete simulation of the chemical photographic process:

1. Virtual negative exposure (scene light hitting spectral-sensitive emulsion layers)
2. Chemical dye density modeling (how different dyes form in development)
3. DIR couplers — inter-layer chemical interactions (this is the magic of "film color")
4. Print/paper exposure simulation
5. Development process simulation
6. Grain simulation using actual crystal data from manufacturer specs
7. Halation (Gaussian blur per-channel for light scatter through base)

**What DIR couplers are (and why they matter):** Developer Inhibitor Release couplers are a chemical mechanism in multi-layer film that causes the development of one layer to *inhibit* adjacent layers. This is why film has that characteristic color rendering where shadows go slightly greenish, highlights have that warm roll-off — it's inter-layer chemistry, not a color grade. `agx-emulsion` simulates this. No other open-source tool does.

**For specific film stocks:** Kodak Vision 3 250D, Kodak Vision 3 500T, Fuji Eterna 500, and others — using their actual published spectral data.

**What this means for FilmTools:** Our film emulation engine won't approximate film. It will simulate the chemistry. The reference implementation is `agx-emulsion`. We port the algorithms to Rust. Dehancer ($99/yr) and FilmConvert ($149+) are likely approximating what `agx-emulsion` calculates from first principles.

---

## Breakthrough 4: Newson et al. (ACM ToG 2023) — Publication-Grade Film Grain

**CPU version:** https://github.com/alasdairnewson/film_grain_rendering
**GPU/CUDA version:** https://github.com/alasdairnewson/film_grain_rendering_gpu
**Paper:** https://dl.acm.org/doi/10.1145/3592127 (ACM Transactions on Graphics)

Most film grain implementations use Gaussian noise or Perlin noise with a post-process filter. They look okay at a glance but fall apart under scrutiny — especially when you slow down or zoom in.

Newson et al. published a **physically accurate stochastic grain model** in one of the most prestigious graphics journals:

- Grains are modelled as individual circular silver halide crystals with log-normal size distributions
- Placement is a Poisson process in image space (not random per-pixel — actual grain clustering)
- Each grain has transmittance characteristics matching real emulsion physics
- The density and size of grains varies with exposure level (fine grain in highlights, coarser in shadows) — exactly as real film behaves
- Resolution-independent rendering — zooming in reveals more grain, not pixellation
- Monte Carlo sampling with configurable grain radius, filter sigma, sample count

**The GPU version** is CUDA-based. We port it to **wgpu** for Mercury-Motion, making it run on any GPU (Vulkan/Metal/DX12) without requiring CUDA. This is one of the most interesting Rust + wgpu implementation tasks in the project.

**What this means for FilmTools:** When a filmmaker applies "Kodak Vision 3 250D grain" in Mercury-Motion FilmTools, they're getting a mathematically rigorous simulation of that specific film stock's grain structure — not a texture overlay. This is what separates a serious tool from a preset pack.

---

## Breakthrough 5: OCIO 2.5 — Vulkan GPU Acceleration

**GitHub:** https://github.com/AcademySoftwareFoundation/OpenColorIO
**Released:** September 2025 (VFX Reference Platform 2026)
**License:** BSD 3-Clause

OpenColorIO has always been the industry standard for color management — DaVinci Resolve, Nuke, Blender, Houdini, Arnold all use it. But its GPU support was limited to OpenGL, Metal, and DirectX.

**OCIO 2.5 adds Vulkan GPU support.**

This matters enormously for Mercury-Motion because:
- Mercury-Motion's renderer uses `wgpu` with a Vulkan backend
- OCIO 2.5's GPU path also uses Vulkan
- They speak the same GPU language — enabling zero-copy color transforms in the same command buffer as rendering

**What ships in OCIO 2.5:**
- Vulkan GPU renderer (new)
- Built-in ACES 2.0 configs — `cg-config-v4.0.0_aces-v2.0` and `studio-config-v4.0.0_aces-v2.0` ship with the release
- New hue curve transform type
- Config merging (preview feature)
- The ACES 2.0 pipeline is now under Academy Software Foundation (joined ASWF August 2025)

**The OCIO integration gap:** No production-ready Rust bindings exist yet. `opencolorio-sys` is a raw FFI stub. The path forward: implement common transforms natively using the `palette` crate (camera log curves and ACES matrix operations are just math), and use OCIO via C FFI for the full config-driven pipeline.

---

## Bonus: Other Significant Finds

### `palette` — The Rust Color Math Foundation
[crates.io/crates/palette](https://crates.io/crates/palette) — MIT
Type-safe color space conversions in Rust: CIE Lab/LCh, sRGB, linear RGB, Oklab, Jzazbz, ACEScg, gamut mapping. This is the foundation everything else is built on when implementing color science natively in Rust.

### `dng-rs` — Cinema DNG in Pure Rust
[apertus-open-source-cinema/dng-rs](https://github.com/apertus-open-source-cinema/dng-rs)
From the **AXIOM** open cinema camera project — an open-source cinema camera that shoots Cinema DNG. Zero-copy DNG read/write in pure Rust. This is DNG as it was meant to be used — by people building actual cameras.

### ZoeDepth — Metric Depth (Not Just Relative)
[isl-org/ZoeDepth](https://github.com/isl-org/ZoeDepth)
MiDaS estimates *relative* depth (which pixel is in front of which). ZoeDepth adds a metric binning module for **absolute depth in meters**. For realistic depth-of-field simulation, knowing the actual distance matters — you can set a focus distance of 3 meters and have the physics be correct, not just visually plausible.

### agx-emulsion Free Print LUTs (Reference Starting Points)
- Juan Melara: Fuji 3510, Kodak 2383, Kodak 2393 — free print film emulation LUTs based on measured data
- Cullen Kelly: Kodak 2383 + Fuji 3510 (ACES and DaVinci Wide Gamut compatible)
These aren't the final product — they're starting points and validation targets for our spectral simulation.

---

## Why This Stack Is Unprecedented

Before this research, we expected FilmTools to require:
- ARRI's proprietary SDK for RAW decoding → **solved by gpu-video**
- RED's proprietary SDK (requires license agreement) → **solved by gpu-video**
- C++ wrappers for stabilization → **solved by Gyroflow Core (pure Rust)**
- Commercial references for grain simulation → **solved by Newson et al. (open, publication-grade)**
- Approximate film emulation curves → **solved by agx-emulsion (actual manufacturer data)**
- OpenGL-only OCIO GPU transforms → **solved by OCIO 2.5 Vulkan**

The result: a FilmTools stack that is predominantly **native Rust**, runs on **any GPU via Vulkan**, uses **actual measured physical data** for film simulation, and requires **zero proprietary SDKs**.

This is genuinely unprecedented for an open-source tool. DaVinci Resolve is closed. Dehancer is closed. FilmConvert is closed. Nuke is $5,000/yr. Mercury-Motion FilmTools will be MIT-licensed and free forever.

---

*Document created: 2026-03-22. Update as new finds emerge.*
