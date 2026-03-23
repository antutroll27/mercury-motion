# Mercury-Motion — Color Science Skill Reference
**Date:** 2026-03-23
**Skill location:** `~/.claude/skills/color-science/SKILL.md`
**Status:** Installed — auto-loads on any `mercury-filmtools/color/` work

---

## What the Skill Covers

| Section | Covers |
|---|---|
| **Log curves** | Full math for FLog2, SLog3, SLog2, LogC3 (all EIs), LogC4, Log3G10, CLog2/3, VLog, NLog |
| **Gamut matrices** | F-Gamut, S-Gamut3.Cine, REDWideGamut → AWG4 or ACEScg |
| **Camera matching** | `slog3_to_arri_logc4_look()`, `flog2_to_arri_logc4_look()` — single function calls |
| **ACES pipeline** | IDT matrices for every camera, CDL (Slope/Offset/Power/Sat) |
| **3D LUT** | `.cube` file parser + trilinear interpolation + GPU wgpu 3D texture upload + WGSL sampler |
| **OCIO 2.5** | C FFI integration strategy (native Rust for speed, OCIO for studio configs) |
| **`palette` crate** | Oklab gamut mapping, perceptual color math |
| **Display transforms** | Rec.709, sRGB, HDR10/PQ |
| **agx-emulsion** | Integration order (response curve → halation → Newson grain → gamma encode) |

---

## The Core Idea in Two Lines

```rust
let linear = pixel.map(flog2_to_linear);           // decode Fuji log
let awg4 = apply_matrix(F_GAMUT_TO_AWG4, linear);  // F-Gamut → ARRI Wide Gamut 4
```

Feed that into a LogC4 LUT and colorists can't tell it came from a Fuji.

Same pattern for Sony:

```rust
let linear = pixel.map(slog3_to_linear);
let awg4 = apply_matrix(SGAMUT3CINE_TO_AWG4, linear);
```

---

## Camera Log Quick Reference

| Camera | Log Format | Gamut | IRE at 18% grey | Stop range |
|---|---|---|---|---|
| Fujifilm X-H2S | FLog2 | F-Gamut | 38% | ~13 stops |
| Sony A7S III / FX3 | S-Log3 | S-Gamut3.Cine | 41% | ~15 stops |
| Sony FX6 / FX9 | S-Log3 | S-Gamut3 | 41% | ~15 stops |
| ARRI Alexa 35 | LogC4 | AWG4 | — | ~17 stops |
| ARRI Alexa Mini LF | LogC3 (EI800) | AWG3 | 40% | ~14 stops |
| RED KOMODO / V-RAPTOR | Log3G10 | REDWideGamutRGB | 25% | ~16 stops |
| Canon C70 / C300III | CLog2 / CLog3 | Cinema Gamut C | — | ~16 stops |
| Panasonic S5 / GH6 | V-Log | V-Gamut | 42% | ~14 stops |
| Nikon Z8 / Z9 | N-Log | N-Gamut | — | ~12 stops |
| Blackmagic Pocket 6K | BRAW (various) | BMD Film Gen5 | — | ~13 stops |

---

## The Full Grade Pipeline

```
Camera log footage
      │
  [1] Log decode → scene-linear          (FLog2/SLog3/LogC4/etc.)
      │
  [2] Gamut matrix → target gamut        (F-Gamut → AWG4, or → ACEScg for ACES)
      │
  [3] CDL (primary grade)                (Slope / Offset / Power / Saturation)
      │
  [4] Creative LUT (.cube)               (GPU 3D texture sample — ~0 cost)
      │
  [5] RRT + ODT                          (ACES tone map via OCIO, or baked LUT)
      │
  [6] agx-emulsion film response         (characteristic curve → DIR couplers)
      │
  [7] Halation                           (highlight red-channel blur)
      │
  [8] Newson grain                       (Poisson grain, wgpu compute shader)
      │
  [9] Display gamma encode               (Rec.709 / sRGB / PQ)
      │
  output frame
```

---

## Common Mistakes to Avoid

| Mistake | Fix |
|---|---|
| Grading before log decode | Always decode to scene-linear FIRST |
| Matrix multiply after gamma encode | Matrices work in SCENE-LINEAR only |
| Applying a Rec.709 LUT to log footage | Log decode → gamut convert → THEN LUT |
| Using sRGB luma for CDL saturation | Use Rec.709 luma: `0.2126R + 0.7152G + 0.0722B` |
| Mixing LogC3 and LogC4 constants | LogC4 is Alexa 35 only; older ARRI = LogC3 with EI |
| Wrong gamut matrix (S-Gamut3 vs S-Gamut3.Cine) | Same log curve, different primaries — don't mix |
| Powering negative values in CDL | Clamp to 0 before `powf()` — negative base = NaN |

---

## Why This Is Gap-Filling

Zero Claude Code skills exist anywhere for OCIO, color grading, LUT processing, or camera log math.
Mercury-Motion FilmTools is the first open-source Rust tool to implement this pipeline natively.

Dehancer: $99/yr. FilmConvert: $149+. Mercury-Motion: MIT, free, forever.

---

*Skill file: `C:\Users\Acer\.claude\skills\color-science\SKILL.md`*
*Update this summary if the skill is expanded (e.g. BRAW-specific IDTs, Blackmagic Film Gen5 support).*
