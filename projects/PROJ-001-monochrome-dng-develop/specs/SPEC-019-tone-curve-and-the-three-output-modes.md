---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-019
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
  verify_verdict: null             # approved | punch-list | rejected — set when it leaves verify

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5        # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-052 (build), tier_map.build
  created_at: 2026-10-26

references:
  decisions: [DEC-018, DEC-002, DEC-016, DEC-005, DEC-021, DEC-017]
  constraints:
    - no-panics-on-untrusted-input
    - provenance-recorded-per-algorithm
    - library-not-application
    - no-copyleft-dependencies
    - oracle-must-be-shown-red
  related_specs: [SPEC-014, SPEC-015, SPEC-020]

# Blocking dependencies: specs that must SHIP before this one can start.
depends_on: []                     # develop_into (SPEC-014/017/018) and the oracle
                                   # harness (SPEC-020) are all already shipped.

claimed_by: null

value_link: "STAGE-003: turns the scene-linear plane into the display-referred image a person actually looks at — the tone curve and the three output representations a consumer encodes from."

cost:
  tokens_estimate: null
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      recorded_at: 2026-10-26
      notes: "main-loop, not separately metered (design); real-file probe + spec + handoff + build prompt"
    - cycle: build
      interface: claude-code
      tokens_total: 41313385
      duration_minutes: 25
      estimated_usd: 105.42
      recorded_at: 2026-10-26
      notes: "HANDOFF-052; summed from this session's Claude Code transcript (in+out+cache-create+cache-read, 165 usage records); USD at per-category Opus list rates (in $15/out $75/cache-write $18.75/cache-read $1.50 per Mtok); see handback"
    - cycle: verify
      interface: claude-code
      tokens_total: 16131777
      duration_minutes: 20
      estimated_usd: 41.43
      recorded_at: 2026-10-27
      notes: "HANDOFF-053; summed from the verify session's Claude Code transcript (in 200/out 119,153/cache-create 490,945/cache-read 15,521,479, 100 records, all claude-opus-4-8); USD at per-category Opus list rates; transcribed by orchestrator (verifier worktree on a different branch)"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-019: Tone curve and the three output modes

## Context

This is the **last spec in STAGE-003** ("Opcodes and output"). Its three
siblings have shipped: `SPEC-020` (the develop oracle harness), `SPEC-018`
(`WarpRectilinear`) and `SPEC-017` (`FixBadPixelsConstant`). After this spec,
STAGE-003's spec backlog is complete and STAGE-004 (crustyimg integration) can
consume this output.

`develop::develop_into` today produces a **scene-linear `u16`** image:
`[BlackLevel, WhiteLevel] → [0, u16::MAX]`, round-to-nearest, clamped
(`DEC-018`). Its own doc says so plainly (`src/develop.rs:682`):

> The tone-curve stage `SPEC-019` will add after this one is, for now, a no-op:
> this function's output IS the final image.

Two gaps follow, and they are this spec:

1. **No tone curve.** A scene-linear image shown directly is far too dark — the
   reference render `dnglab analyze --srgb` is **16-bit sRGB** (gamma-encoded,
   display-referred; `docs/oracle-contract.md:22`), and nothing in the library
   yet applies that transfer. STAGE-003's charter names "a tone curve
   (table-driven)" as a deliverable.

2. **One output representation.** `develop_into` writes `u16` and only `u16`.
   STAGE-003's charter names **three** output modes: "linear f32, u16, or
   8-bit."

### What curve — grounded in the real file, not assumed

Design-time probe (`exiftool -s -G1` on `LEICA-Q2-MONO/L1026192.DNG`,
2026-10-26) — the Q2M carries **no profile tone curve**:

```
[IFD0]   BaselineExposure          : 0
[SubIFD] PhotometricInterpretation : Linear Raw
[SubIFD] SamplesPerPixel           : 1
[SubIFD] BitsPerSample             : 14
[SubIFD] BlackLevel                : 512
[SubIFD] WhiteLevel                : 16383
```

No `ProfileToneCurve`, no `LinearizationTable`, no `TransferFunction`, no
`RenderingIntent`, no `ColorMatrix`. `PhotometricInterpretation: Linear Raw`
with `SamplesPerPixel: 1` means the sensor plane *is* the image
(`docs/measured-q2m-dng.md`). The only transfer between the scene-linear plane
and the display-referred sRGB reference is therefore the **sRGB OETF** (the
output colour space's own transfer function, IEC 61966-2-1). Honouring a
per-file `ProfileToneCurve`/`LinearizationTable` is deliberately out of scope
(there is none to honour, and it is the next wave — STAGE-003 scope excludes
"LUT / film response curves").

### The architectural choice this spec makes

**`develop_into` is not reopened. `DEC-018` stays.** The develop core keeps
producing scene-linear `u16`; the tone curve and the three modes are a
**separate, composable encode layer** (new `src/tone.rs`) that reads the
scene-linear develop result and writes one of three caller-owned output buffers
(the same caller-owned-buffer shape `DEC-016` chose for `unpack_into` /
`develop_into`). This keeps the existing develop/levels/warp oracles stable (no
decoded-pixel change to a shipped function — AGENTS.md §8's "a fix that changes
decoded pixel output is never a patch" does not fire, because nothing shipped
changes) and matches the library's ethos: the develop produces the image, the
caller chooses the representation it encodes.

## Goal

Add a table-driven sRGB tone curve and three output encoders — scene-linear
`f32`, display-referred 16-bit sRGB `u16`, and display-referred 8-bit sRGB `u8`
— that turn `develop_into`'s scene-linear `u16` result into the representation a
consumer displays, deterministically and byte-identically across platforms, with
the tone math proven correct analytically against the sRGB specification (not
against another decoder) and proven to go red on a wrong table.

## Inputs

What the implementer will read or consume.

- **Files to read:**
  - `src/develop.rs` — the scene-linear `u16` output these encoders consume
    (`develop_into`, `output_dimensions`); its module doc's "Output
    representation — `DEC-018`" and the `:682` tone-curve-is-a-no-op note.
  - `src/lib.rs` — the `Error` enum (add one length-mismatch variant) and the
    five crate-level panic-free `deny` lints these functions live under.
  - `examples/fuzz-seeds.rs` + the `fuzz-seeds` recipe in `app.just` — the
    **established pattern** for generating committed data via an example + a
    `just` recipe. The tone table generator mirrors it.
  - `decisions/DEC-018-*.md` (why develop output is scene-linear `u16`),
    `decisions/DEC-016-*.md` (caller-owned buffers), `decisions/DEC-021-*.md`
    and `decisions/DEC-017-*.md` (the mutated-copy / asymmetric red-proof
    patterns to mirror).
- **External APIs:** none. No new dependency (`no-copyleft-dependencies`; the
  sRGB OETF is arithmetic).
- **Related code paths:** `src/`, `tests/`, `examples/`, `app.just`,
  `docs/provenance-ledger.md`.

## Outputs

What the implementer will produce.

- **Files created:**
  - `src/tone.rs` — the public encode layer (three functions below) and the
    sRGB lookup logic. Carries the provenance comment (IEC 61966-2-1).
  - `src/tone_table.rs` — the committed, **generated** const lookup tables
    (`pub(crate) const SRGB_ENCODE_8: [u8; 65536]` and
    `pub(crate) const SRGB_ENCODE_16: [u16; 65536]`), with a header comment
    saying it is generated and how to regenerate it. This is data, not
    hand-written logic.
  - `examples/gen-tone-table.rs` — regenerates `src/tone_table.rs` from the
    sRGB OETF (the one place `powf` is allowed to run — at generation time, on
    the maintainer's machine, producing committed bytes that are then
    byte-identical for everyone).
  - `tests/tone_oracle.rs` — the analytic tone oracle and its red-proof.
- **Files modified:**
  - `src/lib.rs` — `pub mod tone;` and one new `Error` variant
    (`OutputBufferWrongLength { expected, actual }` or reuse an existing
    length-mismatch shape — implementer's call, but it must be a *typed* error,
    never a panic).
  - `app.just` — a `gen-tone-table` recipe (mirrors `fuzz-seeds`), and its line
    added to §6 of `AGENTS.md`'s command block in the **same change**
    (acceptance criterion 8 of `SPEC-001`: a recipe that gains a command gains a
    line there).
  - `src/bin/irr.rs` — `irr develop` gains a `--format linear-f32|srgb16|srgb8`
    option (default `srgb8`) so each mode has a **reader** (AGENTS.md §11
    "ship the reader with the field"); this is the dev/oracle binary, not a
    product surface.
  - `docs/provenance-ledger.md` — a row for the sRGB OETF (see Acceptance
    Criteria).
- **New functions (exact signatures):**
  - `pub fn to_linear_f32_into(linear: &[u16], dst: &mut [f32]) -> Result<(), Error>`
    — scene-linear, `dst[i] = linear[i] as f32 / 65535.0`. Tone curve **not**
    applied. `[0.0, 1.0]`.
  - `pub fn encode_srgb16_into(linear: &[u16], dst: &mut [u16]) -> Result<(), Error>`
    — display-referred 16-bit sRGB via `SRGB_ENCODE_16`. Directly comparable to
    `dnglab analyze --srgb`'s 16-bit sRGB PNM.
  - `pub fn encode_srgb8_into(linear: &[u16], dst: &mut [u8]) -> Result<(), Error>`
    — display-referred 8-bit sRGB via `SRGB_ENCODE_8`. `[0, 255]`.
  - Each returns `Err` iff `dst.len() != linear.len()`; otherwise writes every
    element and returns `Ok(())`.
- **New flags / options:** `irr develop --format <linear-f32|srgb16|srgb8>`,
  default **`srgb8`**.
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Each maps to at least one test in `## Failing Tests`.

- [ ] **AC1 — scene-linear f32.** `to_linear_f32_into` maps `0 → 0.0`,
      `65535 → 1.0`, and `v → v/65535.0` exactly for a swept set of inputs; the
      tone curve is **not** applied. Length mismatch returns the typed error,
      not a panic.
- [ ] **AC2 — 16-bit sRGB.** `encode_srgb16_into` applies `SRGB_ENCODE_16`:
      `0 → 0`, `65535 → 65535`, monotonic non-decreasing across the full
      domain, and matches the analytic sRGB OETF (AC5). Length mismatch →
      typed error.
- [ ] **AC3 — 8-bit sRGB.** `encode_srgb8_into` applies `SRGB_ENCODE_8`:
      `0 → 0`, `65535 → 255`, monotonic non-decreasing, matches the analytic
      OETF (AC5). Length mismatch → typed error.
- [ ] **AC4 — no runtime `powf`, table-driven.** The **library** crate
      (`src/`, excluding `examples/`) contains no `powf`/`exp`/`ln` or other
      transcendental on the encode path; the sRGB mapping is a committed const
      table plus integer lookup, and `to_linear_f32` is a single IEEE division.
      (Mechanically: a test / grep asserting the encode path calls no `powf`,
      and the tables are `const`.)
- [ ] **AC5 — analytic tone oracle (independent of rawler).** Every entry of
      both committed tables matches the sRGB OETF (IEC 61966-2-1) recomputed in
      `f64` to within **±1 code value**, with the two segment endpoints and the
      `0.0031308` linear/power breakpoint checked exactly at the rounding rule.
      The oracle derives its own expected values arithmetically — it reads
      nothing from `dnglab`/`rawler` (`DEC-004`'s circularity warning; this is
      STAGE-003's "analytic, non-rawler check" success criterion).
- [ ] **AC6 — the oracle goes red (`oracle-must-be-shown-red`).** A deliberate
      fault — one committed table entry perturbed by a large delta (≫ the ±1
      tolerance) — turns AC5 **red**, with the **unmutated** table run first as
      the negative control (green). Mirrors `DEC-017`/`DEC-021`: the red-proof
      operates on a mutated **copy**, so the shipped table carries no fault
      switch, and reverts byte-identical.
- [ ] **AC7 — determinism / cross-platform byte-identity.** Because the tables
      are committed const data and lookups are integer (and `to_linear_f32` is
      one IEEE-deterministic division), the three encoders' output is
      byte-identical on macOS and Linux. A test pins the exact anchors
      (`0`, `65535`, the breakpoint sample) so CI (Linux) and a local macOS run
      both assert the same bytes; `just gen-tone-table` reproduces the committed
      tables with no diff.
- [ ] **AC8 — provenance row.** `docs/provenance-ledger.md` gains a row:
      module `src/tone.rs` (+ `src/tone_table.rs`), algorithm "sRGB OETF
      (opto-electronic transfer function)", source "IEC 61966-2-1", source
      licence "published specification", provenance class **1**, note "nothing
      read from dnglab/rawler; table generated by `examples/gen-tone-table.rs`".
- [ ] **AC9 — the reader ships with the field.** `irr develop --format`
      exercises all three modes; a test (or the `irr` smoke path) reads each.
      No output mode exists that nothing reads (AGENTS.md §11).
- [ ] **AC10 — gates green.** `just lint-ci`, `just lint-no-allow`,
      `just test --all-features`, `just deny` + `just deny-fuzz`, and `just msrv`
      pass, and CI is **observed** green on the shipping SHA (not asserted from a
      laptop — PATCH-001's lesson).

## Failing Tests

Written during **design**, BEFORE handoff. The implementer's job in **build**
is to make these pass. ⚠ A zero-match `cargo test <name>` exits 0 — confirm each
test actually *ran* (per-target, summed), the trap `SPEC-015/AC9` and `SPEC-016`
both call out.

- **`tests/tone_oracle.rs`** (new integration test; the analytic oracle + its
  red-proof)
  - `"srgb16_table_matches_the_analytic_oetf_within_one_code_value"` — AC5:
    recompute the sRGB OETF in `f64` for all 65536 indices, assert each
    `SRGB_ENCODE_16` entry is within ±1 of the rounded analytic value.
  - `"srgb8_table_matches_the_analytic_oetf_within_one_code_value"` — AC5: same
    for `SRGB_ENCODE_8`.
  - `"srgb_tables_pin_the_exact_anchors"` — AC2/AC3/AC7: `SRGB_ENCODE_16[0]==0`,
    `SRGB_ENCODE_16[65535]==65535`, `SRGB_ENCODE_8[0]==0`,
    `SRGB_ENCODE_8[65535]==255`, and the `0.0031308` breakpoint sample equals
    the analytic linear-segment value.
  - `"srgb_tables_are_monotonic_non_decreasing"` — AC2/AC3: both tables never
    decrease across the domain.
  - `"analytic_tone_oracle_goes_red_on_a_mutated_table"` — AC6: build a **copy**
    of a table with one entry shifted by +40, assert the ±1 oracle rejects the
    copy (red), and that the unmutated table passes (negative control). Reverts
    byte-identical — no fault switch ships.
- **`src/tone.rs` unit tests** (`#[cfg(test)] mod tests`)
  - `"to_linear_f32_maps_endpoints_and_is_pure_rescale"` — AC1:
    `0 → 0.0`, `65535 → 1.0`, swept `v → v/65535.0`.
  - `"encoders_reject_a_wrong_length_destination"` — AC1/AC2/AC3: each encoder
    returns the typed length-mismatch `Error` (never panics) when
    `dst.len() != linear.len()`.
  - `"encoders_write_every_element_on_a_correct_length_buffer"` — AC1/AC2/AC3:
    full-buffer round trip, no element left unwritten.
  - `"srgb8_matches_srgb16_downscaled_at_anchors"` — AC3: the 8-bit mode is the
    8-bit rounding of the same curve (consistency between the two tables).
- **`examples/gen-tone-table.rs` regeneration check** — AC7: a test (or a CI
  step in `just gen-tone-table`) regenerates the tables and asserts **no diff**
  against the committed `src/tone_table.rs`.

> **Analytic-oracle note.** The ±1 tolerance is in the *test*, not the shipped
> table: cross-platform `powf` can differ in the last bit, so an exact
> re-derivation would be a platform-dependent test. The committed table is the
> single byte-identical source of truth (AC7); the oracle proves it is *correct*
> (±1 of the spec) and the red-proof proves a *wrong* table is caught. This is
> the same split `DEC-023`/`SPEC-020` used for the perceptual metric.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to happen,
they file a new spec in STAGE-003's (or a later stage's) backlog, not expand
this one.

- **Honouring a per-file `ProfileToneCurve` / `LinearizationTable`.** The Q2M
  carries none (probe above); profile/film curves are the next wave
  (STAGE-003 explicitly excludes "LUT / film response curves").
- **Colour, white balance, `ColorMatrix`, demosaic.** Monochrome; PROJ-002.
- **Auto-tone, histogram-driven adaptivity, exposure compensation.** A
  determinism hazard and a STAGE-003 non-goal.
- **Changing `develop_into`'s scene-linear `u16` contract (`DEC-018`).** The
  encode layer is additive; the develop core is untouched.
- **End-to-end SSIMULACRA2 scoring of a real render vs `dnglab --srgb`.** That
  is `SPEC-020`'s harness, and `DEC-024`/`SPEC-021` already record that the
  `--srgb` reference is *uncorrected* for warp, so a correct warp scores *worse*
  — the perceptual end-to-end score is a known scope-limited check, not this
  spec's. This spec validates the **tone math analytically** (AC5), which is
  exactly the independent check STAGE-003 asks for.
- **The `develop_version` field.** It lands in STAGE-004; this stage's output
  is version 1 by convention (`DEC-002`), but the field itself is not this
  spec's.

## Notes for the Implementer

- **A new DEC is expected at build (`DEC-025`): "the STAGE-003 tone curve is the
  sRGB OETF; display-referred output is sRGB-encoded, scene-linear output skips
  the curve."** Record the load-bearing choice (sRGB OETF as the default curve
  because the file carries no profile curve; three modes with f32 = scene-linear
  and u16/u8 = display-referred; `develop_into` left scene-linear per `DEC-018`)
  with honest confidence. Cite the probe above as evidence.
- **The sRGB OETF (the one formula to transcribe), L ∈ [0,1] → V ∈ [0,1]:**
  `V = 12.92·L` for `L ≤ 0.0031308`, else `V = 1.055·L^(1/2.4) − 0.055`. Table
  index `i` is the scene-linear `u16`; `L = i / 65535.0`. Scale `V` to the
  output range (`×255` or `×65535`), **round to nearest** (consistent with
  `DEC-018`'s rounding rule), clamp into range.
- **Determinism is the whole point of the table.** Generate it once with `powf`
  in `examples/gen-tone-table.rs`; the library never calls `powf`. If the 16-bit
  table as a source literal is unwieldy, `include_bytes!` of a committed
  little-endian `.bin` decoded with `u16::from_le_bytes` is an acceptable
  equivalent (compile-time embed is not runtime I/O, so `library-not-application`
  holds) — provided it is committed, regenerable via `just gen-tone-table`, and
  validated by AC5/AC7. The generated-const form mirrors `examples/fuzz-seeds.rs`
  and is the default.
- **Panic-free.** These are library paths: no `unwrap`/`expect`/indexing that
  can go out of range/`panic!`/arithmetic that can overflow. Use `.get()`,
  `iter().zip()`, `try_into()`. The five `deny` lints apply.
- **Reuse, don't reinvent, the red-proof shape.** `DEC-017` (plane oracle
  rebuilds a mutated copy) and `DEC-021` (develop oracle's asymmetric mutation)
  are the models — operate on a copy, keep the shipped table clean.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word:
   `design` | `build` | `verify` | `ship` | `escaped` | `none`.
   — <one word>

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after.
   — <answer | none>
