---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-063
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-019
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-058, DEC-018, DEC-019, DEC-024, DEC-028, DEC-043, DEC-051, DEC-052, DEC-012, DEC-003]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-014, SPEC-017, SPEC-018, SPEC-064, SPEC-065, SPEC-066, SPEC-067]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-019's <capability>". Optional; null is acceptable.
value_link: "Infrastructure enabling STAGE-019's configurable mono develop: the options struct every other STAGE-019 option hangs off, with framing and orientation delivered."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 30000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-11-12
      notes: "Un-metered main-loop design in the long monorail-000 orchestrator session, which also framed STAGE-019. Its cost lands in STAGE-019's orchestration_cost at stage close, not here."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-063: DevelopOptions + develop_with_into: framing and orientation options with a byte-identical default

## Context

SPEC-063 is the first spec in STAGE-019 (PROJ-004 instrument layer C) and covers
monorail-000 / onyx-000 request 2 (`feedback/2026-11-11-monorail-000-requests.md`).
SPEC-064 (warp alpha), SPEC-065 (caller scratch) and SPEC-066 (CFA-as-mono) each add a
field to the options struct defined here, so they depend on it. SPEC-067 (blocked
transpose) does not.

Today `develop_into` (`src/develop.rs:802`) runs one fixed recipe:
1. `FixBadPixelsConstant`;
2. levels;
3. `WarpRectilinear` over the `ActiveArea`;
4. `DefaultCrop`;
5. `Orientation`.

A caller cannot ask for the uncropped active area, or for unrotated pixels. The requester
wants both:
- the active area for an instrument view that shows every recorded photosite (536,832 more
  on the Q2M);
- unrotated output, because their app rotates on the GPU, and our Orientation 5–8 gather
  costs them 774 ms against 617 ms (their spike).

`docs/future-plans.md` § PROJ-004 locked the two framing modes on 2026-11-02.

**DEC-058** (minted with this design) fixes the options shape, the "default is
`develop_into`" rule, validate-then-override, typed refusal for combinations that aren't
safe yet, and the `DEVELOP_VERSION` rule.

## Goal

Add `DevelopOptions` (framing `StandardCrop` or `FullActiveArea`, plus `apply_orientation`)
and `develop_with_into` / `output_dimensions_with`. `develop_into` and `output_dimensions`
delegate with `DevelopOptions::default()` and give byte-identical output on every held file.

## Design

### 1. Probe results (main at 3458854, 2026-11-12; recorded so build is transcription)

Held tier-B files, read through the public `Sensor`, with `develop_into` run on each in
release:

| File | Plane | ActiveArea (t,l,b,r) → size | DefaultCrop origin / size | Orient | Warp (OpcodeList3) | develop_into |
|---|---|---|---|---|---|---|
| Q2M L1021223 | 8424×5632 | (0,0,5632,8392) → 8392×5632 | (12,24) / 8368×5584 | 1 | **non-identity** | ok 8368×5584 |
| Q2M L1026016 | 8424×5632 | same | same | **6** | **non-identity** | ok 5584×8368 |
| Q2M L1026192 | 8424×5632 | same | same | 1 | **non-identity** | ok 8368×5584 |
| M Monochrom L1000622 | 5216×3472 | absent → 5216×3472 | (2,2) / 5212×3468 | 1 | none | ok |
| M Monochrom Typ 246 M2462362 | 5984×4000 | absent → 5984×4000 | (4,4) / 5976×3992 | 1 | none | ok |
| K-3 III K3III.DNG | 6304×4224 | (34,26,4194,6250) → 6224×4160 | (28,24) / 6192×4128 | 1 | none | ok |
| Canon 5D3 5G4A9394 (CFA) | 5920×3950 | (80,122,3950,5918) → 5796×3870 | (18,16) / 5760×3840 | 1 | none | ok (see below) |
| Nikon D300S _DSC4615 (CFA) | 4352×2868 | (0,0,2868,4320) → 4320×2868 | (15,10) / 4288×2848 | 1 | none | ok (see below) |
| Pixel 4a / 3a / 7 Pro (CFA) | | | | 1 | 4a: identity | **refused**: "opcode 9 is unrecognized and not marked optional" |
| K3III.PEF | | | | | | unpack refuses (compression 65535) |

What follows from it:
- **Every Q2M frame has a real warp.** `kr0 = 0.999251106`, `kr1` from -0.032 to -0.061.
  So under §4 `FullActiveArea` is refused on all of them until SPEC-064. The
  `FullActiveArea` real-file oracle uses the three warp-free mono files instead (K-3 III,
  L1000622, Typ 246); each has a real border between the active area and the crop.
- **L1026016 is Orientation 6 and warped.** It is the real-file oracle for
  `apply_orientation: false` through the warp path.
- **Finding for SPEC-066.** The mono path does not refuse a CFA-declared sensor today: the
  Canon and Nikon files develop as monochrome. On the Canon, the scalar `black_level` is 2047
  while `black_levels` has 4 per-position values. This spec does not change that, and
  STAGE-019's design notes record it for SPEC-066.

**Pinned `develop_into` output** (MD5 of the output as little-endian `u16` bytes, from main
at 3458854). These are the byte-identity oracle for AC2:

| File | Output | MD5 |
|---|---|---|
| LEICA-Q2-MONO/L1021223.DNG | 8368×5584 | `feb4910588d189bd5bf89620b480fe6c` |
| LEICA-Q2-MONO/L1026016.DNG | 5584×8368 | `2bc53a6337b582a8080ac1206e62434b` |
| LEICA-Q2-MONO/L1026192.DNG | 8368×5584 | `f9a9c56c6255436ac7e764cb0a7c21fb` |
| LEICA-M-MONOCHROM/L1000622.DNG | 5212×3468 | `fddf0ea0dec6a66bf1fd1cb8c787cc79` |
| LEICA-M-MONOCHROM-TYP246/M2462362.DNG | 5976×3992 | `4aa013f64317412111c9d298300dcb28` |
| PENTAX-K3III-MONO/K3III.DNG | 6192×4128 | `82d2d51957922587ec55ee594b2a6b14` |
| CANON-EOS-5D-MARK-III/5G4A9394.DNG | 5760×3840 | `f533e4052026097e799d46bc8bc794bc` |
| NIKON-D300S/_DSC4615.DNG | 4288×2848 | `09faddddd24a11497cc072bcf3a42ce9` |

The three Pixel files are pinned as refusals with the text above.

### 2. Public API (`src/develop.rs`), exactly DEC-058 §1

- `DevelopOptions { pub framing: Framing, pub apply_orientation: bool }`:
  - `#[non_exhaustive]`;
  - derives `Debug, Clone, Copy, PartialEq` (not `Eq`, not `Hash`);
  - `impl Default` gives `StandardCrop` and `true`.
- `Framing { StandardCrop, FullActiveArea }`:
  - `#[non_exhaustive]`;
  - derives `Debug, Clone, Copy, PartialEq, Eq, Default`, with `#[default] StandardCrop`.
- `pub fn develop_with_into(sensor: &Sensor, src: &[u16], dst: &mut [u16], options: &DevelopOptions) -> Result<(), Error>`.
- `pub fn output_dimensions_with(sensor: &Sensor, options: &DevelopOptions) -> Result<(u32, u32), Error>`.
- `develop_into` becomes one line delegating with `&DevelopOptions::default()`. So does
  `output_dimensions`. Their doc comments say so and link the `_with` forms.
- Doc comments on every field and variant:
  - what it changes;
  - its default;
  - that options are **mono only** (the colour path takes none);
  - the §4 refusal.

### 3. Semantics: validate, then override (DEC-058 §3)

`resolve_geometry(sensor)` runs **unchanged**, so every hostile shape it refuses today is
still refused under every option. Then a new private `apply_options(geometry, options)`
rewrites the resolved `Geometry`:
- **`Framing::FullActiveArea`:** `crop_origin_x = crop_origin_y = 0`,
  `crop_width = active_width`, `crop_height = active_height`.
- **`apply_orientation: false`:** `orientation = 1`.

Every pass downstream already reads `Geometry`, so the three existing passes
(`crop_orient_normalize_into`, `normalize_active_area_into`,
`crop_and_orient_from_active_into`) need no change.

The output dimensions are `oriented_dimensions` of the rewritten geometry:
- `FullActiveArea` gives the active size, swapped for Orientation 5–8 when it is applied;
- `apply_orientation: false` gives the crop size, never swapped.

### 4. The one refusal (DEC-058 §4)

`FullActiveArea` together with a **non-identity** `WarpRectilinear` (after today's
`is_identity` filter) returns:

```rust
Error::UnsupportedWarpedFullActiveArea { active_width: u32, active_height: u32 }
```

- It is classified `ErrorKind::Unsupported`.
- `Display` names both values and says the warp's edge samples would be visible. Don't
  write spec IDs into user-facing text.

**Order inside `develop_with_into`:**
1. `check_required_processing`;
2. `src` length;
3. `resolve_geometry`, then `apply_options`;
4. levels;
5. `dst` length, against the options dimensions;
6. **only when `framing == FullActiveArea`:** parse `OpcodeList3` and refuse a non-identity
   warp;
7. then today's `FixBadPixelsConstant` copy, and the rest unchanged.

So the refusal comes **before** the plane copy and before any write to `dst`. Under
`StandardCrop` the parse order and the error precedence are exactly today's. Under
`FullActiveArea` a malformed `OpcodeList3` is reported before a malformed `OpcodeList1`;
document that.

`output_dimensions_with` does **not** look at opcode lists (nor does `output_dimensions`
today). It returns the active size even for a combination `develop_with_into` will refuse.
Its doc says so.

### 5. Errors (`src/lib.rs`)

- One new variant, `UnsupportedWarpedFullActiveArea`, as above. Add it to `Error::kind()`
  with no wildcard (DEC-051), and add its row to `tests/error_kind.rs`.
- No other new variant. Wrong buffer lengths reuse `DevelopBufferWrongLength` /
  `SourcePlaneWrongLength`, with `expected` computed from the options dimensions.

### 6. Fuzz (`fuzz/fuzz_targets/develop.rs`)

After the existing `develop_into` call, take one option byte from the input (for example the
last byte, documented in the target header):
- bit 0 selects `FullActiveArea`;
- bit 1 clears `apply_orientation`.

Run `output_dimensions_with` / `develop_with_into` under the same `MAX_PIXELS` cap. Add
seeds to `examples/fuzz-seeds.rs`: a fixture with `ActiveArea` and a crop, and one with a
warp, each ending in each option byte. No seed comes from a corpus file (DEC-003).

### 7. Docs

- **CHANGELOG `[Unreleased]`:** `DevelopOptions`, `Framing`, `develop_with_into`,
  `output_dimensions_with` and the new `Error` variant, under Added. Note that
  `develop_into` output is unchanged and `DEVELOP_VERSION` stays `"4"`.
- **Provenance ledger:** one row for `src/develop.rs` options. It is DNG 1.7 Chapter 4
  (`ActiveArea`, `DefaultCrop`) and TIFF/Exif `Orientation`, as for SPEC-014; no new
  algorithm.
- **Module doc of `src/develop.rs`:** a short "Options (SPEC-063, DEC-058)" paragraph.
- `DEVELOP_VERSION` is **not** bumped (DEC-058 §5). AC2 proves the default output is
  unchanged.

## Inputs

- **Files to read:**
  - `src/develop.rs` (all of it; `resolve_geometry` 139, the three passes 559–700,
    `develop_into` 802);
  - `src/lib.rs` (`Error`, `Error::kind`, `DEVELOP_VERSION`);
  - `src/warp.rs` (`is_identity`);
  - `src/opcode.rs` (`parse_warp_rectilinear`, `check_required_processing`).
- **Tests to mirror:**
  - `tests/develop.rs` (`minimal_sensor`, `corpus_sensor`, the orientation tests, the
    warp-branch fixtures 1097–1240);
  - `tests/support/opcode.rs` (`single_warp_rectilinear_list`,
    `single_fix_bad_pixels_constant_list`);
  - `tests/support/md5.rs`;
  - `tests/error_kind.rs`.
- **Decisions:** DEC-058 (this spec), DEC-018, DEC-019, DEC-024, DEC-028, DEC-043, DEC-051,
  DEC-052, DEC-012, DEC-003.

## Outputs

- **Files modified:**
  - `src/develop.rs`: the options types, `develop_with_into`, `output_dimensions_with`,
    `apply_options`, and the delegation;
  - `src/lib.rs`: the new `Error` variant, its `Display` and its `kind()` row;
  - `fuzz/fuzz_targets/develop.rs` and `examples/fuzz-seeds.rs`;
  - `CHANGELOG.md`;
  - `docs/provenance-ledger.md`;
  - `tests/error_kind.rs`.
- **Files created:** `tests/develop_options.rs`.
- **New public items:** `develop::DevelopOptions`, `develop::Framing`,
  `develop::develop_with_into`, `develop::output_dimensions_with`,
  `Error::UnsupportedWarpedFullActiveArea`.
- **New options and their defaults:**
  - `framing`: `StandardCrop` (default) or `FullActiveArea`;
  - `apply_orientation`: `true` (default) or `false`.

## Acceptance Criteria

- [ ] **AC1. Defaults.** `DevelopOptions::default()` has `framing == StandardCrop` and
  `apply_orientation == true`. `Framing::default() == StandardCrop`.
- [ ] **AC2. Byte identity (tier B, the load-bearing oracle).** On each of the eight files
  in §1, both `develop_into` and `develop_with_into(&DevelopOptions::default())` produce
  output whose MD5 equals the pinned value. Both refuse the three Pixel files with the same
  `Display` text, and `DEVELOP_VERSION == "4"`. The test must report a missing file by name
  (the corpus-skip convention), never pass silently.
- [ ] **AC3. FullActiveArea dimensions.**
  - `output_dimensions_with` returns the active size on tier-A fixtures (with and without
    an `ActiveArea` tag; Orientation 1 and 6, swapped for 6).
  - On tier B: K-3 III 6224×4160, L1000622 5216×3472, Typ 246 5984×4000. On the Q2M it
    returns 8392×5632, even though develop refuses there (§4).
- [ ] **AC4. FullActiveArea pixels, by two relations, not by a re-implementation.** On
  tier-A fixtures and on the three warp-free tier-B files, `FullActiveArea` output equals:
  - (a) `StandardCrop` output of the same sensor with `default_crop_origin = (0, 0)` and
    `default_crop_size` = the active size;
  - (b) contains `StandardCrop` output of the unmodified sensor at
    `(crop_origin_x, crop_origin_y)`.

  At least one tier-A fixture carries `FixBadPixelsConstant`, so the copy path runs under
  `FullActiveArea`.
- [ ] **AC5. The refusal.**
  - **Synthetic.** A sensor with a non-identity warp (and also `FixBadPixelsConstant`)
    under `FullActiveArea` returns `UnsupportedWarpedFullActiveArea` with the right
    dimensions, `kind() == Unsupported`, and `dst` unchanged (a sentinel fill).
  - **Real file.** The same refusal on Q2M L1021223.
  - **Identity warp.** A sensor with an identity warp (`kr = [1, 0, 0, 0]`) under
    `FullActiveArea` develops.
  - **`StandardCrop`.** The non-identity sensor develops exactly as `develop_into` does.
- [ ] **AC6. `apply_orientation: false`.**
  - **All eight orientations.** For each of Orientation 1 to 8, on a non-square tier-A crop
    with distinct pixel values: output equals the default output of the same sensor with
    `orientation = Some(1)`, and its dimensions are the unswapped crop size.
  - **Warp path, real file.** On Q2M L1026016 (Orientation 6, warped), `apply_orientation:
    false` equals `develop_into` of the sensor with `orientation = Some(1)`, at 8368×5584.
- [ ] **AC7. Combination.** `FullActiveArea` with `apply_orientation: false` on an
  Orientation 6 tier-A fixture gives the active size, unswapped. Its pixels equal AC4(a)'s
  sensor with `orientation = Some(1)`.
- [ ] **AC8. Validation is not skipped (DEC-058 §3).**
  - Under `FullActiveArea`, a `DefaultCrop` that does not fit is still `InvalidDefaultCrop`.
  - Under `apply_orientation: false`, `Orientation = 9` is still `UnsupportedOrientation`.
  - Both hold for `develop_with_into` and for `output_dimensions_with`.
- [ ] **AC9. Buffer lengths.** A `dst` sized for the default dimensions but passed with
  `FullActiveArea`, or with `apply_orientation: false` on Orientation 6 when the crop is
  non-square, is `DevelopBufferWrongLength`. Its `expected` is the options product. `dst`
  is not written.
- [ ] **AC10. Error surface.**
  - `UnsupportedWarpedFullActiveArea` is in `Error::kind()` with no wildcard.
  - It has a row in `tests/error_kind.rs`.
  - A test pins that its `Display` contains both dimensions.
- [ ] **AC11. API hygiene.**
  - `DevelopOptions` and `Framing` are `#[non_exhaustive]`, with every field and variant
    documented.
  - An external test constructs options only via `Default` plus field assignment.
  - `cargo doc` has no broken links.
  - CHANGELOG `[Unreleased]` names every addition.
- [ ] **AC12. Fuzz.** The `develop` target exercises `develop_with_into` under all four
  option combinations; the new seeds are written by `examples/fuzz-seeds.rs`; a 60 s run is
  clean.
- [ ] **AC13. No regression.**
  - Every existing test passes with no assertion changed (other than `error_kind`'s rows
    and counts).
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.
  - The `src/` diff adds no allocation on any path, and none under `FullActiveArea` before
    the refusal.

## Failing Tests

- **`tests/develop_options.rs`** (new):
  - `default_options_are_standard_crop_and_oriented`: AC1.
  - `develop_into_output_is_unchanged_on_held_files`: AC2, tier B, the MD5 table from §1.
  - `full_active_area_dimensions`: AC3, tier A and tier B.
  - `full_active_area_equals_standard_crop_over_the_whole_active_area`: AC4(a).
  - `full_active_area_contains_the_standard_crop`: AC4(b).
  - `full_active_area_with_a_warp_is_refused_before_any_write`: AC5, synthetic + Q2M.
  - `identity_warp_does_not_trigger_the_refusal`: AC5.
  - `unoriented_output_equals_orientation_one_for_all_eight`: AC6.
  - `unoriented_q2m_orientation_six_through_the_warp_path`: AC6, tier B.
  - `full_active_area_unoriented_combination`: AC7.
  - `options_do_not_skip_tag_validation`: AC8.
  - `dst_length_is_checked_against_the_options_dimensions`: AC9.
  - `refusal_display_names_the_active_size`: AC10.
- **`tests/error_kind.rs`**: the new row (AC10).

**Red-proof expectations** (verify mutates these; each must turn a named test red):
- Default `FullActiveArea` → AC1, AC2.
- `apply_options` skipped → AC3, AC4, AC6.
- The crop override set to the crop instead of the active area → AC4.
- The refusal removed → AC5.
- The refusal moved after the `FixBadPixelsConstant` copy or after a `dst` write → AC5's
  sentinel.
- `apply_orientation` ignored → AC6.
- `resolve_geometry` bypassed under an option → AC8.
- `dst` checked against default dimensions → AC9.

## Non-Goals

- **`warp_alpha`, `WarpRect::lerp`, and relaxing the §4 refusal.** SPEC-064.
- **Caller-owned scratch / removing internal allocations.** SPEC-065.
- **CFA-as-mono, per-position levels, or refusing CFA on the mono path.** SPEC-066. The §1
  finding is recorded for it.
- **The blocked transpose.** SPEC-067.
- **Options on the colour path, or `irr develop` CLI flags.** No named caller; the CLI can
  follow when one asks.
- **A `DEVELOP_VERSION` bump, a release, or any change to default output.**

## Notes for the Implementer

- **Pins come from before your change.** The §1 MD5s were taken on main at 3458854. If
  AC2 goes red, your change altered default output. Fix the change, never the pin.
- **Prefer the geometry rewrite (§3)** over new branches in the three passes. If a pass
  seems to need an option-specific branch, stop and say why in the handback.
- **The relations in AC4 and AC6 use only the public API.** Assign `Sensor` fields on a
  value from `Sensor::test_fixture()` or from `container.sensor()`. Do not re-derive
  normalisation in the test.
- **Tier-B tests** take `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place, never copy them (DEC-003). Each Q2M develop is ~47 MP, so keep
  the number of full develops per test modest.
- **`just test` already passes `--all-features`;** add no flag.
- **Commit-message bodies** are plain prose with no code tokens. `Cargo.lock` is
  gitignored. Never force-push.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
