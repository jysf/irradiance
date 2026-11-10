---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-049
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/lib.rs
  - src/opcode.rs
  - src/develop.rs
  - src/color.rs
  - Cargo.toml
  - tests/
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-054-required-processing-validation-and-sensor-api-stability.md

tags:
  - opcodes
  - dng
  - required-processing
  - sensor-api
  - non-exhaustive
  - semver
  - stage-010
  - spec-054
---

# DEC-049: Validate required opcode processing on every develop path, and seal the `Sensor` view

## Decision

Two coupled pre-0.2.0 correctness/API fixes for review finding **#3 (P1)**. The
opcode-validation part is the load-bearing one; the `Sensor` part is the API change
the field addition forces.

### 1. Required-processing validation (the P1 fix)

- **Keep the `OpcodeList2` bytes.** Add `opcode_list_2: Option<Vec<u8>>` to `Sensor`,
  read from tag `51009` exactly as `opcode_list_1` / `opcode_list_3` are read, with
  the same `DEC-012` interpretation-class tolerance (a structurally-unreadable tag is
  `None` and recorded in `malformed_tags`, never fatal). `opcode_lists: [bool; 3]`
  stays as the presence summary.
- **Every public develop entry validates every present opcode list** — the mono path
  (`develop_into`) **and** the colour path (`develop_color_into`), plus their `f32`
  and threaded variants, since those share the fused kernel and must not offer a
  validation-free door. For each opcode in a present list `1`/`2`/`3`:
  - **optional** (DNG `Flags` bit 0 set) → may be skipped (spec-permitted);
  - **mandatory** (bit 0 unset) → it must be *exactly* the instance this path will
    apply; otherwise the call returns a new typed
    `Error::UnsupportedRequiredOpcode { list: u8, id: u32 }`.
- **The "applied" set is path- and list-specific:**
  - mono path applies the **first** `FixBadPixelsConstant` in `OpcodeList1` and the
    **first** `WarpRectilinear` in `OpcodeList3`;
  - the colour path applies **nothing** (`src/color.rs:255`);
  - **`OpcodeList2` is applied by no path.**
  So a mandatory opcode in `OpcodeList2`, a mandatory opcode on the colour path, a
  recognized-but-wrong-list mandatory opcode (e.g. a mandatory `WarpRectilinear` in
  `OpcodeList2`), and a **second** mandatory instance beyond the one the applier
  consumes (the `find_map` first-match-only gap the review named) all become the
  typed error. Optional instances of any of these are still skipped.
- **No `DEVELOP_VERSION` bump.** This only turns some former-`Ok` (silently wrong)
  results into `Err`; it never changes the bytes of a develop that still succeeds, so
  the determinism contract is untouched (same reasoning as DEC-047 for group B).

### 2. `Sensor` is a sealed parser-output view

- `Sensor` is produced by the parser (`Container`) and **read** by consumers; it is
  not a type consumers construct. Make it `#[non_exhaustive]`, which both answers the
  triage's open maintainer decision #3 (consumers do **not** build `Sensor` directly)
  and makes the `opcode_list_2` field addition — and every future hardening field —
  non-breaking.
- `#[non_exhaustive]` forbids struct-literal / `..rest` construction **outside the
  defining crate**, which breaks the crate's own integration tests (`tests/`, a
  separate crate) that build `Sensor { .. }` fixtures (7 sites across
  `tests/develop_oracle.rs`, `tests/colour_metadata_oracle.rs`,
  `tests/color_oracle.rs`). Migrate those fixtures onto an **in-crate, dev-only
  constructor/builder** gated behind a cargo feature (mirrors the `fuzzing` feature,
  `DEC-036`): `just test` runs `--all-features` so the fixtures compile, while the
  default / published surface gains nothing. The builder's defaults also shrink the
  fixtures. C must touch all 7 fixtures anyway to add `opcode_list_2`, so the seal +
  migration happens **once**, here, rather than a second time later.
- The **remaining** public parser-output types flagged semver-urgent by the staff
  review (`Cfa`, `WarpRect`, and the `Opcode` enum — reconciling `src/opcode.rs`'s
  "`#[non_exhaustive]`-free by choice" note) are sealed as **one sweep in SPEC-042
  (group F, the release spec)**, where they already live in the stage plan. Sealing
  the field-growing struct (`Sensor`) in C and the rest in F keeps each a single
  reviewable change and seals last-touched types last.

## Why

`Sensor.opcode_lists` is presence-only and only lists 1 and 3 keep bytes, so
`OpcodeList2`'s payload is dropped on read. Worse, `parse_opcode_list`
(`src/opcode.rs:299`) **discards `Flags` for every recognized opcode** — it keeps the
optional bit only on `Opcode::Unknown` — and the colour path calls it not at all. So
a *recognized* mandatory opcode in the wrong list, or any mandatory opcode on the
colour path, parses clean and is silently dropped: the file develops to a
plausible-looking wrong image and `Ok(())`. This is the
`parser-accepts-what-pipeline-cannot-honor` signal (now N≥2 with the `Hi/Vi` item).

**Measured safety of a validation-only fix (corpus probe 2026-11-10, `exiftool -b`
→ per-opcode `id`/`flags`):** CANON-5D3 and NIKON-D300S (the Bayer/colour files) carry
**no opcode lists at all**; the Leica Q2-Mono carries a mandatory `FixBadPixelsConstant`
in list 1 and a mandatory `WarpRectilinear` in list 3 — **both applied on the mono
path**; the other mono files carry none; **no corpus file carries `OpcodeList2`.** So
rejecting unapplied-required opcodes leaves the entire shipped corpus `Ok` and
bit-exact, and the whole of group C is hostile/unsupported-input hardening — matching
the triage's "none of the P1s touch the corpus."

## Rejected alternatives

- **Reshape `Opcode` to carry `flags` on every variant** and validate off the typed
  parse — rejected: a small flags-aware scan over the raw opcode framing (the same
  `id/version/flags/size` reader `parse_opcode_list` already uses) is lighter, needs
  no enum-shape change, and naturally sees list position and duplicate instances
  (which the typed `find_map` path cannot).
- **Actually apply the opcodes on the colour path** instead of rejecting — rejected
  for 0.2.0: no corpus file needs it, and a correct colour-path opcode pipeline is a
  large piece of work; the review asks for a *typed error*, not new honouring. A
  future spec may add application.
- **Defer all `#[non_exhaustive]` to F** (add `opcode_list_2` as a plain field in C) —
  rejected: C must edit all 7 `Sensor` fixtures regardless, so sealing in C migrates
  them once; deferring would touch them in C (add field) and again in F (literal →
  builder).
- **A `#[doc(hidden)] pub` constructor** for the fixtures — rejected in favour of the
  feature-gated one: no even-nominally-public construction path, and it reuses the
  established `fuzzing`-feature precedent.

## Mechanics

- `src/ifd.rs`: read tag `51009` into `opcode_list_2` beside lists 1/3; keep
  `opcode_lists` presence. `Sensor` gains `#[non_exhaustive]`.
- `src/opcode.rs` (or a small preflight helper E can later absorb): a `pub(crate)`
  flags-aware support scan taking the three list byte-slices and the path's applied
  set, returning `Ok(())` or `Error::UnsupportedRequiredOpcode { list, id }`. It must
  still surface the existing `Error::UnsupportedMandatoryOpcode` / `Error::Truncated`
  / `Error::MalformedOpcodeParams` shapes for genuinely malformed framing.
- `src/lib.rs`: add the `UnsupportedRequiredOpcode { list, id }` variant (the `Error`
  enum is already `#[non_exhaustive]`, `src/lib.rs:143`) and its `Display`.
- `src/develop.rs` / `src/color.rs`: call the scan at the top of each public develop
  entry, before any allocation or decode; §12 behavioural verification means a test
  exercises **each** entry, not just one.
- `Cargo.toml`: a dev-only feature gating the in-crate `Sensor` fixture constructor;
  migrate the 7 `tests/` fixtures onto it.
- Provenance ledger row for the validation logic (DNG 1.7.0.0 Ch.7 `Flags`
  semantics, this build's own scan); CHANGELOG `[Unreleased]` entry.

## Scope and non-goals

- **In:** keep `OpcodeList2` bytes; required-processing validation on mono + colour
  (and variants); `Error::UnsupportedRequiredOpcode`; seal `Sensor` + migrate its
  fixtures; red-proofs; fuzz coverage of the scan; docs.
- **Out:** honouring any new opcode or the colour-path opcode pipeline; sealing
  `Cfa` / `WarpRect` / `Opcode` (SPEC-042); any change to a corpus develop's output;
  a `DEVELOP_VERSION` bump; the parser guard (SPEC-052), the Compression-7 shape
  binding (SPEC-053), or the licence/package work (SPEC-055).
