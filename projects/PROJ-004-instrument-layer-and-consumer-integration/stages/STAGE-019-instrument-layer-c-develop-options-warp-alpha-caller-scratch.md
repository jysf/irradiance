---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-019                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-12
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Lets an instrument-grade consumer control the mono develop instead of taking one fixed recipe: framing, how much of the lens warp to apply, whether to orient, who owns the scratch memory, and whether a CFA-declared sensor is developed as monochrome, with every default byte-identical to develop_into.
  delivers:
    - "A #[non_exhaustive] DevelopOptions with a Default that reproduces develop_into byte for byte, and develop_with_into taking it: framing StandardCrop or FullActiveArea, apply_orientation"
    - "warp_alpha in [0, 1] over WarpRect::lerp, with a typed refusal (never an out-of-bounds read or a silent edge clamp) where the lerped radial scale would leave FullActiveArea"
    - "A caller-owned scratch variant (scratch_len plus a &mut [u16]) so develop allocates nothing, the FixBadPixelsConstant plane copy included"
    - "A CFA-as-mono option that develops a CFA-declared plane on the mono path with per-position black and white levels"
    - "A blocked Orientation 5-8 transpose, byte-identical, with a recorded before/after timing"
  explicitly_does_not:
    - "Change any default output: develop_into and every dnglab or analytic oracle stay byte-identical"
    - "The colour path (develop_color_into and friends): the named caller is monochrome; colour parity is a later, separate question"
    - "The displacement-field sampler from future-plans Stage C: no named caller"
    - "Optical black (parked Stage B, onyx DEC-010) or MaskedAreas (item 8, low priority, STAGE-014 or later)"
    - "Parallel unpack (request 1): its own stage after this one"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-019: Instrument layer C: develop options, warp alpha, caller scratch, CFA-as-mono, blocked orientation

## What This Stage Is

The mono develop becomes configurable. Today `develop_into` runs one fixed recipe: bad
pixels, levels, warp, `DefaultCrop`, orientation (`src/develop.rs`), with internal
allocations whenever a warp is present. After this stage a caller passes a
`DevelopOptions` to `develop_with_into` and chooses:
- the framing, `StandardCrop` (`DefaultCrop`, today's) or `FullActiveArea` (`ActiveArea`);
- how much of the lens correction to apply, `warp_alpha` in `[0, 1]` over
  `WarpRect::lerp`;
- whether to apply `Orientation`;
- whether a CFA-declared sensor is developed as monochrome, with per-position levels;
- and, through a scratch variant, who owns the working memory.

`Default` reproduces `develop_into` exactly, and `develop_into` keeps its signature. The
Orientation 5–8 path also gets a cache-friendly blocked loop with the same output.

This is PROJ-004 instrument-layer Stage C (`docs/future-plans.md` § PROJ-004, geometry),
merged with monorail-000 / onyx-000 requests 2, 3, 6 and 4
(`feedback/2026-11-11-monorail-000-requests.md`).

## Why Now

It is second in the order agreed with the requester on 2026-11-12 (Stage A, then Stage C,
then parallel unpack), and Stage A (STAGE-018) shipped the same day. It has a named caller
whose spike measured the costs: two active-area scratch `Vec`s on every Q2M develop (every
real Q2M frame carries a mandatory `WarpRectilinear`), and 774 ms vs 617 ms for
Orientation 6 vs 1 on the same plane. Everything it touches already exists and is
oracle-covered, so the defaults can be pinned byte-identical before anything changes.

## Success Criteria

- **Defaults are byte-identical.** `develop_with_into(&DevelopOptions::default())`
  equals `develop_into` on every tier-A fixture and every held tier-B file, and every
  existing develop oracle (dnglab, analytic, SPEC-017/018/019) stays green unchanged.
- **Each option has an oracle and a red-proof.**
  - `FullActiveArea` output is the `ActiveArea` (8392×5632 on the Q2M, recovering
    536,832 photosites over `StandardCrop`), checked by dimensions and by pixel identity
    with the levels-only plane where no warp applies.
  - `warp_alpha = 0` equals the warp-free path; `warp_alpha = 1` equals today's warp. A
    bounds oracle over every held frame for alpha in {0, .25, .5, .75, 1}, red-proofed with a
    synthetic over-unity polynomial that must be refused, typed.
  - `apply_orientation = false` equals the oriented output un-rotated.
  - CFA-as-mono: a synthetic CFA-declared uniform plane equals the mono path; per-position
    levels are honoured; a red-proof with positions swapped goes red.
- **The scratch variant allocates nothing.** A counting-allocator test (the
  `tests/previews_alloc.rs` pattern) shows zero allocations on a warped, bad-pixel-opcode
  fixture; wrong-length scratch is a typed error before any write.
- **The blocked transpose is byte-identical** for Orientation 1–8 on fixtures and held
  files, and a before/after timing on the Q2M is recorded (method fixed at design; no new
  benchmark dependency unless its licence passes `deny`).
- **No panics, no new dependency, typed errors** in `ErrorKind` (DEC-051); new public types
  `#[non_exhaustive]` (DEC-052); fuzz seeds cover the options.

## Scope

### In scope
- `DevelopOptions`, `Framing`, `develop_with_into`; `develop_into` delegating to it.
- `WarpRect::lerp(alpha)` and `warp_alpha`, with the `f(r) <= 1` validity rule measured on
  the corpus (future-plans "Pin at design time" 1).
- `develop::scratch_len(sensor, &options)` and a scratch variant, covering the two
  active-area buffers, the `FixBadPixelsConstant` full-plane `src.to_vec()` and the
  neighbours `Vec` in `apply_fix_bad_pixels_constant`.
- A CFA-as-mono option and the `BlackLevelRepeatDim`-to-position mapping it needs.
- A blocked loop in `crop_orient_normalize_into` and a recorded timing.
- CHANGELOG, provenance rows, fuzz seeds.

### Explicitly out of scope
- The colour develop path, the displacement-field sampler, optical black, `MaskedAreas`,
  parallel unpack (see `value_contribution.explicitly_does_not`).
- Tangential warp terms (`kt`): still refused, as today.
- Any change to default output, `develop_into`'s signature, or `DEVELOP_VERSION` unless a
  spec shows a default output changes (none should).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Order: the options struct lands first; the other four hang off it. The blocked transpose
does not need it and can run in parallel with any of them.

- [x] SPEC-063 (shipped 2026-11-12) [M→M] — DevelopOptions + develop_with_into: #[non_exhaustive] options with a byte-identical Default, Framing StandardCrop / FullActiveArea, apply_orientation; develop_into delegates; a DEC for the options shape and the version rule.
- [x] SPEC-064 (shipped 2026-11-12) [M→M] — warp_alpha over WarpRect::lerp: measure the backward radial scale across held frames, refuse typed where FullActiveArea plus the lerped warp would sample outside the active area, bounds oracle at five alphas, over-unity red-proof. Depends on the options spec.
- [x] SPEC-065 (shipped 2026-11-13) [M→M] (re-sized from S at design: shared core, allocation-free opcode walkers, two test binaries) — Caller-owned develop scratch: scratch_len plus a scratch variant covering the warp buffers, the FixBadPixelsConstant plane copy and the neighbours Vec; zero-allocation test; typed wrong-length error. Depends on the options spec.
- [x] SPEC-066 (shipped 2026-11-13) [M→M] (re-sized from S at design: two normalize sites, odd-origin phase fixtures, tier-B Canon relation) — CFA-as-mono option: `DevelopOptions::black_levels` (`FirstValue` default / `PerPosition`), BlackLevel indexed by BlackLevelRepeatDim from the ActiveArea origin, white per sample (DEC-061); uniform-plane analytic oracle and swapped-position red-proof. Depends on the options spec.
- [ ] SPEC-067 (frame) [S] — Blocked Orientation 5-8 transpose in crop_orient_normalize_into: byte-identical for all eight orientations, before/after Q2M timing with a method fixed at design.

- [ ] (unframed) [XS] — SPEC-065 and SPEC-066 verify follow-ups. SPEC-065: a median fixture whose neighbours arrive unsorted, so "neighbours unsorted" is caught by an `apply_fix_bad_pixels_constant` test directly; a develop-level malformed-`OpcodeList1` row in `tests/develop_scratch_alloc.rs`'s error-path list. SPEC-066: AC5's "the warp ran" guard does not prove the warp ran (`FirstValue` differs from `PerPosition` with no warp too); AC7's `develop_err` does not check the scratch sentinel.
- [ ] (unframed, outside STAGE-019's requests) [S] — Level-tag follow-ups found by the SPEC-066 design (DEC-061 §5): a RATIONAL non-integral `BlackLevel` (Pixel 7 Pro) is costed as malformed and resolves to black 0 on every path, mono and colour; and the colour path's `level_at` indexes `black_levels` by a fixed 2x2 slot, ignoring `BlackLevelRepeatDim` (wrong for a `[1, 2]` or `[2, 1]` pattern; right for every held file).

**Count:** 4 shipped / 3 active / 0 pending (one spec, SPEC-067, plus two unframed follow-up lines. SPEC-063 25.96M tok / $18.62; SPEC-064 32.22M / $23.71; SPEC-065 41.89M / $29.94; SPEC-066 shipped 2026-11-13, DEC-061, verify approved post-merge, 26.73M / $19.51)

## Design Notes

Facts and open questions to pin at design time:
- **Where allocation happens today.** `develop_into` (`src/develop.rs` ~802): `src.to_vec()`
  when `OpcodeList1` has `FixBadPixelsConstant`; two `vec![0u16; active_len]` when a
  non-identity warp applies; `Vec::with_capacity(8)` inside
  `apply_fix_bad_pixels_constant` (~504). The scratch spec must remove all of them on its
  path, or say which remain and why.
  **Measured by the SPEC-065 design (DEC-060):** on a Q2M frame `develop_into` makes 6
  allocations / 283,943,504 bytes. Two of the small ones are the `Vec<Opcode>` that
  `parse_opcode_list` builds inside `parse_fix_bad_pixels_constant` and
  `parse_warp_rectilinear`, which these notes missed. Those lookups become
  allocation-free walkers. With the plane copy reused for the warp output, the scratch is
  P + A = 94,707,712 samples on the Q2M, and `develop_into` drops to 1 allocation of
  189.4 MB.
- **The warp clamps to edge** (DEC-024, `src/warp.rs`): an out-of-extent source sample
  repeats the nearest edge pixel. Under `StandardCrop` the clamped region is cropped away;
  under `FullActiveArea` it would be visible. The warp-alpha spec decides whether
  `FullActiveArea` with any `f > 1` region is refused (the future-plans locked rule) or
  only when clamped pixels land inside the output. Measure `max f(r)` per held frame first;
  the old "503.8 px" corner pull is unrecorded and must not be assumed.
- **Lerp shape.** `f_α = (1 − α) + α·f`, so `kr0` lerps toward 1 and `kr1..kr3` scale by
  α; `cx`, `cy` unchanged. `f <= 1` everywhere implies `f_α <= 1` for every α.
- **CFA routing.** `develop_into` has no CFA branch; the mono path reads the scalar
  `black_level`. Confirm with one test whether `check_required_processing(…, Mono)` or
  `resolve_geometry` refuses a CFA sensor today before designing the override.
  **Answered by the SPEC-063 probe (2026-11-12): neither refuses.** The held Canon 5D3 and
  Nikon D300S CFA DNGs develop on the mono path today, with the scalar `black_level` (the
  Canon's is 2047 while `black_levels` carries 4 per-position values). SPEC-066 must
  decide whether the default keeps accepting them (byte-identical, pinned by SPEC-063 AC2)
  or starts refusing (a behaviour change and a `DEVELOP_VERSION` question).
  **Decided by the SPEC-066 design (DEC-061):** the default keeps accepting them,
  byte-identical (`BlackLevels::FirstValue`); `PerPosition` is opt-in. Every held
  `LinearRaw` file declares a 1x1 pattern, so both modes agree on them; the Canon
  changes 5,602,332 of 22,430,520 active samples by at most 6. White is per sample in DNG,
  so there is no per-position white.
- **Source-scanning tests.** `tests/develop_oracle.rs` (and `tests/develop.rs`)
  fault-inject by reading `src/develop.rs` text for named function bodies
  (`crop_orient_normalize_into`, …). Restructuring those functions breaks them. Found by
  the SPEC-066 build; name them in any handoff that touches `develop.rs` (SPEC-067's
  transpose lives in `crop_orient_normalize_into`).
- **Every held Q2M frame carries a non-identity warp.** SPEC-063 refuses `FullActiveArea`
  with a real warp (DEC-058 §4), so on the Q2M that framing waits for SPEC-064.
  **Measured by the SPEC-064 design (DEC-059):** all three Q2M frames keep the backward
  radial scale in `[0.8998, 0.99925]` at every α, so no sample leaves the active area and
  `FullActiveArea` is admitted. The corner pull is 506.1 px on L1021223. The Pixel 3a is a
  held over-unity lens (max 1.0015) and is the red-proof's polynomial.
- **Options and the colour path** (`src/color.rs` mirrors the geometry, DEC-043). Keep the
  options mono-only and say so in the doc comment; do not half-wire colour.
- **Benchmarking.** The repo has no bench harness. The transpose spec picks either an
  `#[ignore]`d timing test or a criterion dev-dependency (licence check), and records
  numbers in the spec, not as a CI gate.

## Dependencies

### Depends on
- STAGE-013 (shipped): the test-only DNG writer, for CFA-declared and warped fixtures.
- DEC-018 (u16 output, clamped levels), DEC-019 (DefaultCrop relative to ActiveArea),
  DEC-024 (warp kernel and clamp), DEC-043 (geometry order), DEC-051, DEC-052.

### Enables
- monorail-000 / onyx-000: framing, partial warp, no-orientation develop into
  caller-owned memory, and converted-camera support.
- The parallel-unpack stage (request 1) comes next in the agreed order.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?**
  - <note>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
