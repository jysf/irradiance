---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-093
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-006
  stage: STAGE-031
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5
  created_at: 2026-11-26

references:
  decisions: [DEC-084, DEC-083, DEC-079, DEC-058, DEC-059, DEC-060, DEC-043, DEC-044, DEC-049, DEC-068, DEC-070, DEC-016, DEC-002]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation, provenance-recorded-per-algorithm]
  related_specs: [SPEC-092, SPEC-091, SPEC-065, SPEC-064, SPEC-063]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-092]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-031's <capability>". Optional; null is acceptable.
value_link: "STAGE-031: the developed mono for export, with onyx's warp parity check intact"

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
  tokens_estimate: 60000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 90
      recorded_at: 2026-11-26
      notes: "Design in a side session started by the PROJ-006 orchestrator (reading, the corpus probe for strip overhead, the oracle digests and the opcode flags, DEC-084, this design, HANDOFF-204). Un-metered main-loop cycle; measured at the STAGE-031 close as orchestration cost."
    - cycle: build
      handoff: HANDOFF-204
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 55124174
      estimated_usd: 35.41
      duration_minutes: 75
      recorded_at: 2026-11-27
      notes: "Re-summed at ship (2026-11-27) over the whole transcript 3a1702f3-fb46-4b6d-808b-f0a4d0906600.jsonl (155 unique message.ids, all claude-opus-5-5, no subagents): input 329 + cache-creation 440,334 (1-hour) + cache-read 54,533,891 + output 149,620, priced at 5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x. The handback's floor was 50,795,351 / 32.95 USD, taken before the handback commit. CI run 45898355595 on 6b29c48 green, 25 checks, both determinism legs."
    - cycle: verify
      handoff: HANDOFF-205
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 24939839
      estimated_usd: 18.80
      duration_minutes: 240
      recorded_at: 2026-11-27
      notes: "Re-summed at ship (2026-11-27) over the whole transcript ef47f062-3d10-4ec0-8ce2-86a58d193f1b.jsonl (111 unique message.ids, claude-opus-5-5 plus synthetic entries with no usage, no subagents): input 226 + cache-creation 468,687 (1-hour) + cache-read 24,394,286 + output 76,640, priced as the build. The handback's floor was 21,924,077 / 17.05 USD, measured live. Wall time includes an app restart that killed the mutant run part way (resumed)."
  totals:
    tokens_total: 80064013
    estimated_usd: 54.21
    session_count: 3
shipped_at: 2026-11-27
---

# SPEC-093: The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling

## Context

Third and last spec of STAGE-031 (PROJ-006). onyx's export path needs the mix as a
developed image matching `output_dimensions_with`, as an option on `develop_with_into`,
so the mix goes through the same geometry and resampling the mono develop uses and
onyx's GPU-to-develop warp parity check (its DEC-062) carries over unchanged. onyx's
viewport draws SPEC-092's mixed plane at sensor geometry, with constant levels 0 and
65535, and warps, crops and rotates it on the GPU as it does a mono sensor plane.
`DevelopOptions` is `#[non_exhaustive]`, so a new field is additive.

From SPEC-092's ship (two oracle gaps its verify found, for this design to take up):

- **`SPEC-092/FU-5`, a contract constant with no test is documentation only.**
  `MONO_MIX_FADE_START`, `MONO_MIX_ROW_FRACTION_BITS` and `MONO_MIX_FADE_WIDTH_BITS`
  described a kernel that reads private constants, and changing any of the public ones
  passed every test until the verify added `the_public_mixer_constants_are_the_kernels`.
  Pre-register one test that ties every public constant this spec adds or relies on
  (for example a mix field's default, a version constant, a scratch size) to the
  computation it describes, so changing either side alone goes red. **Taken up by AC9.**
- **`SPEC-092/FU-6`, a refusal the spec calls "last" needs its own order oracle.**
  SPEC-092's Design §6 put `MixRowOutOfRange` last, but no AC fixture combined it with an
  earlier fault, so building the row before the file checks passed (V5 and V5b). For
  every refusal this design orders (first, last, or between two others), pre-register a
  fixture that carries it together with each fault it must come after or before, on
  every entry point (`develop_with_into`, `output_dimensions_with`, `scratch_len`), and
  assert `dst` is untouched. **Taken up by AC8**, on four entries: the three named and
  `develop_with_scratch_into`, which takes the same options.

**Settled at design (2026-11-26, `DEC-084`, proposed).** The developed mix is the mono
develop's geometry applied to the mixed plane read as an already-normalised plane; the
mixer runs in 512-row strips into caller scratch; a new `DevelopPath::MonoMix` applies
the `GainMap`s and the first `WarpRectilinear` and nothing from `OpcodeList1`.

## Goal

Add an opt-in mix option to `DevelopOptions` so `develop_with_into`,
`develop_with_scratch_into`, `output_dimensions_with` and `scratch_len` develop a colour
Bayer DNG to a mixed monochrome image at the mono develop's geometry, bit-identical to
the mono develop of SPEC-092's mixed plane, with caller-owned scratch and no full-frame
intermediate when there is no warp, and byte-identical to today when the option is
absent.

## Inputs

- **Files to read:**
  - `decisions/DEC-084-...md` (this spec's decision: the rule, the opcode set, the
    options, the strips, the scratch, the order, the versions), `DEC-083` (the row and
    the kernel), `DEC-058` / `DEC-059` / `DEC-060` (options, `warp_alpha` and
    `FullActiveArea`, scratch), `DEC-043` (the colour crop scatter), `DEC-044` (bands),
    `DEC-049` (required processing).
  - The shipped SPEC-092 and SPEC-091 (`specs/done/`): their band form, refusal order,
    determinism fixture and mutant tables. This spec mirrors their shape.
  - `src/develop.rs`: `DevelopOptions` (271), `apply_options` (395),
    `output_dimensions_with` (442), `validate_warp_alpha` (458), `non_identity_warp` (472),
    `crop_source_coords` / `output_coords_from_crop` (495, 534), `develop_with_into`
    (1505), `scratch_len` (1546), `develop_with_scratch_into` (1583), `layout_len` /
    `needed_scratch` (1604, 1617), `develop_core` (1661, its numbered steps),
    `run_passes` (1877), `crop_and_orient_from_active_into` (1321).
  - `src/color.rs`: `PixelSlot` / `WholePlane` (603), the mixer (766 to 1091:
    `mix_weights`, `mix_row_of`, `store_mono`, `mono_mix_pixel`), `srgb_linear_plane_rows`
    (1164), `check_colour_required_processing` (1360), `CropScatter` (1543),
    `resolve_colour_gain_maps` (2295), `resolve_colour_levels` (2420), the
    `with_normalizer!` macro (196).
  - `src/demosaic.rs`: `check_supported_bayer` (553), `develop_rows_banded` (584, its
    `Sensor` clone and `green` allocation), `develop_color_fused` (765, its `RING`
    cache allocation at 806).
  - `src/opcode.rs`: `DevelopPath` (1056), `applied` (1134), `malformed_sets` (1147).
  - `tests/develop_scratch_alloc.rs` (the counting allocator), `tests/determinism.rs`
    (the SPEC-091 colour fixture and `mono_mix_hashes_match_committed`, 690 to 790),
    `tests/mono_mix.rs`, and the `Entry` tables in `tests/malformed_tags.rs`,
    `tests/misshapen_tags.rs`, `tests/unhonoured_tags.rs`, `tests/required_processing.rs`.
- **Corpus:** `$IRRADIANCE_CORPUS_DIR` = `~/PSeven/experiments/crustimg_redo_plus/images`;
  the five held colour files, as SPEC-092 lists them.

## Outputs

- **Files modified:**
  - `src/develop.rs`: the `mono_mix` field, its `Default`, its docs; the mix branch of
    `output_dimensions_with`, `scratch_len` and `develop_core` (Design §2 to §7); the
    module doc's "Options" and "Allocation" sections gain the developed mix.
  - `src/color.rs`: crate-private entry points for the mix's two check steps and its
    strip pass (Design §5), an `ActiveWindow` slot, and `CropScatter` built from an
    already-resolved, options-applied geometry. No public function is added here.
  - `src/demosaic.rs`: `develop_color_fused` takes its local height and its ring cache
    from the caller, and a `develop_rows_banded` form takes its green rows and ring from
    a caller slice with no `Sensor` clone; the existing callers keep their own
    allocations (Design §6).
  - `src/opcode.rs`: `DevelopPath::MonoMix` (Design §4).
  - `src/lib.rs`: `DEVELOP_VERSION`'s doc says what it fixes for the developed mix
    (Design §9); no value change. `MONO_MIX_VERSION`'s doc likewise.
  - `tests/developed_mix.rs` (new): the Tier-A and Tier-B ACs.
  - `tests/develop_scratch_alloc.rs`: the mix cases (AC10).
  - `tests/determinism.rs`: four developed-mix hashes on the SPEC-091 colour fixture
    (its bytes unchanged), and the red-proof extended (AC11).
  - The four `Entry` tables: two rows each, `develop_with_into` and
    `develop_with_scratch_into` with the mix, `DevelopPath::MonoMix`.
  - `fuzz/fuzz_targets/demosaic.rs` and `fuzz/fuzz_targets/develop.rs` (AC12).
  - `docs/determinism.md`: a row for the developed mix.
  - `CHANGELOG.md` `[Unreleased]` Added: the field and the `DevelopPath` variant.
- **Not modified:** `DEVELOP_VERSION` (stays `"8"`), `MONO_MIX_VERSION` (stays `"1"`),
  the `Error` enum (no new variant), every existing output, determinism hash and Tier-B
  pin, `docs/provenance-ledger.md` (no new algorithm: the mix is SPEC-092's, the
  geometry the mono develop's).
- **New public API:** the field `DevelopOptions::mono_mix: Option<[Rational; 3]>` and
  the variant `opcode::DevelopPath::MonoMix`.
- **New flags / options:** `mono_mix`, default `None` (today's mono develop exactly).
  `Some(weights)`: `DEC-083` §1's weights (non-negative rationals with positive
  denominators summing to exactly one), linear-sRGB `[R, G, B]`.

## Design

1. **The option.**
   ```rust
   pub struct DevelopOptions {           // #[non_exhaustive], Debug, Clone, Copy, PartialEq
       pub framing: Framing,
       pub apply_orientation: bool,
       pub warp_alpha: f64,
       pub black_levels: BlackLevels,
       /// SPEC-093, DEC-084. Default None.
       pub mono_mix: Option<[Rational; 3]>,
   }
   ```
   `Rational` (`ifd::Rational`) is `Copy + PartialEq + Eq`, so the derives hold. Weights
   only, the same argument as `mono_mix_*`. No struct wrapper: nothing else is planned
   (`DEC-083` has no fade parameter; presets are named weight sets). Interactions:
   - `framing`, `apply_orientation`, `warp_alpha`: exactly as for the mono develop,
     `DEC-059` §3's bounds rule under `FullActiveArea` included.
   - `black_levels`: **not read** under the mix. The mix's levels are the colour path's
     (`resolve_colour_levels`, `DEC-068` §2, `DEC-070`), which already index
     `BlackLevelRepeatDim`, as `mono_mix_plane_into`'s are. The field's doc says so.
   - The caching rule of `DEC-058` §5 becomes: key on (`DEVELOP_VERSION`, options), and
     also `MONO_MIX_VERSION` when `mono_mix` is set (§9).
2. **The rule (`DEC-084` item 1).** For every sensor, `src`, options with
   `mono_mix = Some(w)`: the developed mix equals
   `develop_with_into(&mono_of(sensor), &mixed, dst, &options_without_mix)`, where
   `mixed` is `mono_mix_plane_into(sensor, src, &w)`'s output and `mono_of(sensor)` is
   `sensor` with black `MONO_MIX_BLACK`, white `MONO_MIX_WHITE`, no `black_levels`, no
   `OpcodeList1` and no `OpcodeList2` (its `OpcodeList3`, geometry and orientation
   kept). The mono develop's `normalize(v, 0, 65535)` is the identity, so this says: the
   mixed value replaces stage 2, and the warp, the framing and the orientation run
   unchanged. This is the test oracle of AC1 and AC4 and the probe's.
3. **Refusals, in order; nothing written to `dst` or scratch before the passes.**
   `develop_with_into` and `develop_with_scratch_into` with the mix (`develop_core`'s
   steps, the mix's own marked ▸):

   | Rank | Check | Error |
   |---|---|---|
   | 0 | `warp_alpha` | `InvalidWarpAlpha` |
   | 1 ▸ | weights | `InvalidMixWeights { channel }` |
   | 2 | required processing on `DevelopPath::MonoMix` (▸ path) | `UnsupportedDngVersion`, `UnsupportedTag`, `MalformedTag`, `UnreadableOpcodeList`, `UnsupportedRequiredOpcode`, `UnsupportedMandatoryOpcode`, framing errors |
   | 3 | `src` length | `SourcePlaneWrongLength` |
   | 4 | geometry, then options | `InvalidActiveArea`, `InvalidDefaultCrop`, `UnsupportedOrientation` |
   | 5 ▸ | the colour levels, then the `GainMap`s | `InvalidLevels`, `InvalidBlackLevelPattern`; `MalformedOpcodeParams { id: 9 }`, `TooManyGainMaps` |
   | 6 | `dst` length | `DevelopBufferWrongLength` |
   | 7 | `FullActiveArea` only: `OpcodeList3` parse and bounds | parse errors, `UnsupportedWarpedFullActiveArea` |
   | — | (`OpcodeList1` is **not** parsed under the mix) | — |
   | 8 | `StandardCrop` only: `OpcodeList3` parse | parse errors |
   | 9 ▸ | CFA | `NotAColorFilterArray`, `UnsupportedCfaPattern` / `Layout` / `PlaneColor` |
   | 10 ▸ | colour calibration | `MissingColorCalibration` |
   | 11 ▸ | the row | `MixRowOutOfRange { channel }` |
   | 12 | scratch form only: scratch length | `DevelopScratchWrongLength` |
   | 13 | the passes: the warp applier | `UnsupportedWarpTangentialTerms` (scratch may be written; `dst` is not) |

   - The buffer errors are the develop entry's own (`SourcePlaneWrongLength`,
     `DevelopBufferWrongLength`), not the plane entries' `ColorSourceWrongLength` /
     `MonoBufferWrongLength`.
   - `output_dimensions_with` with the mix: ranks 0, 1, 4 only. `scratch_len` with the
     mix: ranks 0, 1, 4, then the `OpcodeList3` parse (as today), and **not**
     `OpcodeList1`. Neither reads the levels, the CFA or the calibration ("how much",
     not "whether", as `scratch_len` documents today): a mix on a mono file sizes fine
     and is refused by the develop (`NotAColorFilterArray`), before any write.
4. **Opcodes: `DevelopPath::MonoMix` (`DEC-084` item 3).** `applied(1) = Nothing`,
   `applied(2) = Every(GainMap)`, `applied(3) = First(WarpRectilinear)`;
   `malformed_sets() = Colour's` (levels and geometry, CFA, colour). Its doc: "the
   developed mix, `develop_with_into` with `DevelopOptions::mono_mix`". So a mandatory
   `FixBadPixelsConstant` is `UnsupportedRequiredOpcode { list: 1, id: 4 }` (the mono
   develop applies it, the mix cannot: the mono applier's 3×3 median mixes Bayer
   colours); a mandatory `WarpRectilinear` is applied (the colour entries refuse it);
   optional opcodes are skipped as always, except that the first `WarpRectilinear` is
   applied whatever its flags, as the mono develop does.
5. **Where the mix runs (`DEC-084` item 5).** After rank 12, with `T` and the layout of
   §7:
   - **No non-identity warp** (absent, identity, or `warp_alpha = 0`): run the strip
     pass over the framed rows, `[active_top + crop_origin_y, … + crop_height)` of the
     options-applied geometry, with a `CropScatter` built from that geometry (not from
     `resolve_geometry(sensor)` again, which would ignore the options) as the slot:
     each framed pixel's mix goes straight to its `dst` slot.
   - **A non-identity warp:** run the strip pass over the `ActiveArea` rows with an
     `ActiveWindow` slot (plane pixel `(x, y)` inside the `ActiveArea` → `A1` index
     `(y − top)·active_width + (x − left)`, else dropped) into `A1`; then the mono
     develop's `apply_warp_into(warp, active_width, active_height, A1, A2)` and
     `crop_and_orient_from_active_into(geometry, out_width, out_height, A2, dst)`,
     unchanged.
   - **The strip pass:** for each strip `[s, min(s + 512, end))` of the row range, call
     the band traversal on the caller's `T` slice with the mixer's row and the colour
     normalizer, store each emitted pixel's `mono_mix_pixel` at its slot. Strips are
     output-invariant (SPEC-091 AC11, SPEC-092 AC2), so the result is the whole plane's.
   - The levels/`GainMap`s (rank 5) and the CFA/calibration/row (ranks 9 to 11) are
     resolved once per develop, not per strip.
6. **The demosaic refactor, allocation only.** `develop_color_fused` today allocates its
   `RING·w` cache, and `develop_rows_banded` clones the `Sensor` (to change only its
   height) and allocates its `w·local_rows` green rows. For `develop_with_scratch_into`
   to stay allocation-free (`DEC-060` §2):
   - `develop_color_fused` takes the plane height to traverse and a caller `cache` slice
     (`RING·w`); `develop_color_with` allocates the cache as before.
   - A `develop_rows_banded` form takes one caller slice holding the green rows and the
     ring, and passes the local height instead of a cloned `Sensor`;
     `develop_rows_banded` keeps its signature and allocates that slice itself.
   - Every existing output is unchanged (the traversal reads the same values in the same
     order); the existing determinism hashes and band/threaded tests are the guard.
   - `RING` (8) stays where it is; the `T` formula (§7) reads it, and a unit test ties
     the two (AC9).
7. **Scratch (`DEC-084` item 6).** `S = 512` (a private constant beside `RING`'s user):
   `T = w·(min(h, S + 7) + RING)` samples, the largest band's green rows (a strip of `S`
   rows, three halo rows each side, one more for the even local top) plus the ring.
   - `scratch_len` with the mix: `T` without a non-identity warp; `A + max(A, T)` with
     one (`A = active_width·active_height`).
   - Layout, private: no warp `[T)`; warp `A1 = [0, A)`, then `[A, A + max(A, T))`
     holds `T` while the strips run and `A2 = [A, 2A)` afterwards.
   - On the held files (pre-registered, AC4): Canon 3,119,840; Nikon 2,293,504; Pixel 4a
     2,124,864 (its warp is the identity); Pixel 3a 24,385,536 and Pixel 7 Pro 22,701,936
     (each `2A`, the mono develop's own warp scratch), 2,124,864 and 2,045,814 at
     `warp_alpha = 0`.
   - `develop_with_into` keeps its one allocation of exactly `scratch_len` samples;
     under the mix that is never 0.
   - The design probe measured strips of 512 rows through the public band entry at
     0.99× to 1.06× of the whole-plane mix on the held files (256 rows: up to 1.12×);
     the probe's band calls also cloned and allocated, so the design's cost is lower.
8. **Determinism (`DEC-002`).** The mix is SPEC-092's exact integer kernel; the warp is
   the mono develop's, already hashed on both legs. `tests/determinism.rs` adds, on the
   SPEC-091 colour fixture (bytes unchanged), the developed mix under the default
   options and under an in-memory optional non-identity `WarpRectilinear` with
   Orientation 6, each for the luminance weights and the negative one-hot set: four
   FNV-1a hashes, committed on both CI legs, debug and release.
9. **Versions (`DEC-084` item 8).** No bump. The developed mix is fixed by
   (`DEVELOP_VERSION`, `MONO_MIX_VERSION`, options, weights included). onyx's recipe
   already records both versions and the resolved weights; with this spec it also
   records the options (framing, `apply_orientation`, `warp_alpha`) for the export, as
   it does for a mono export. A later change to the mixer's arithmetic bumps
   `MONO_MIX_VERSION`; a change to where the mix sits in the develop, the strip routing's
   output, or the opcode set bumps `DEVELOP_VERSION`. `DEVELOP_VERSION`'s and
   `MONO_MIX_VERSION`'s doc comments say this.
10. **Fuzz.** The `demosaic` target (colour sensors with a CFA and calibration) calls the
    four entries with fuzzer-chosen weights (valid and not), framing,
    `apply_orientation`, `warp_alpha` from `{1, 0, ½, NaN}`, `dst` one short or one long
    and scratch one short on half the inputs. The `develop` target's spare option bit 5
    sets `mono_mix` to the luminance weights, so its mono sensors reach the mix's
    refusals.
11. **No `irr` hook, no threading.** The developed mix is serial, like
    `develop_with_into`. An `irr` hook remains unneeded: AC4's digests pin real files.

## Acceptance Criteria

Pre-registered. Every check is exact; there is no tolerance in this spec.

- [x] **AC1, the rule on Tier-A fixtures (`DEC-084` item 1).** For every fixture × options
  × weights below, `develop_with_into` with the mix equals the §2 oracle (computed in the
  test from `mono_mix_plane_into` and a `develop_with_into` without the mix on
  `mono_of(sensor)`), and `develop_with_scratch_into` with exactly `scratch_len` samples
  equals both.
  - Fixtures (via `tests/support/dng_writer.rs`; levels that are not the identity,
    black above 0 and white below 65535, so normalisation matters):
    - (i) **odd**: 22 × 18, ActiveArea `[top 3, left 1, bottom 17, right 21]`,
      DefaultCrop origin `(1, 3)` size `17 × 11`, a 2×2 `BlackLevel` pattern whose four
      values differ;
    - (ii) (i) plus an optional non-identity radial `WarpRectilinear` whose lerps at
      α ∈ {½, 1} pass `DEC-059` §3 (scale in `[0, 1]`);
    - (iii) **tall**: 20 × 1100, ActiveArea `[top 3, left 1, bottom 1099, right 19]`
      (18 × 1096), DefaultCrop origin `(1, 5)` size `16 × 1090`, so the active-row strips
      are `[3, 515)`, `[515, 1027)`, `[1027, 1099)`, starting at odd rows, with a
      remainder;
    - (iv) (iii) with the warp of (ii);
    - (v) a `GainMap` fixture (non-unity gains in all four Bayer phases).
  - Options: Orientation 1 to 8 (in the sensor) × `StandardCrop` and `FullActiveArea`
    × `apply_orientation` true and false, at `warp_alpha` 1; plus `warp_alpha` 0 and ½
    on Orientation 6 and 3.
  - Weights: luminance `[2126, 7152, 722]/10000` and a set whose row has a negative
    entry (asserted).
  - Plus three hand-derived literal checks on fixture (i), Orientation 6, `StandardCrop`,
    no warp: named output pixels equal named `mixed` samples (coordinates worked out in
    the test, not through `crop_source_coords`).
- [x] **AC2, the option absent is today (Tier A).** `DevelopOptions::default().mono_mix`
  is `None`; on fixtures (i) and (ii) developed as mono, `develop_with_into` with options
  whose `mono_mix` is `None` equals `develop_into`; every existing determinism constant
  and Tier-B pin is unchanged (AC13).
- [x] **AC3, the strip count and `T`'s cap (Tier A).** Width-8 fixtures whose ActiveArea is
  the whole plane, under `FullActiveArea` with no warp, at heights 512 (one strip
  exactly), 513 (two strips, the second one row), 519 and 520 (where `min(h, S + 7)`
  stops growing): each developed mix equals the oracle, and `scratch_len` with the mix
  is `8·(min(h, 519) + 8)` = 4,160, 4,168, 4,216 and 4,216, written as literals.
- [x] **AC4, the held files (Tier B, all five).** With the luminance weights, for each
  options set the developed mix equals the §2 oracle computed in the test, **and** its
  FNV-1a digest (little-endian `u16`) equals this table (oracle values measured by the
  design probe, 2026-11-26; a mismatch is reported, never edited). `DEVELOP_VERSION ==
  "8"` and `MONO_MIX_VERSION == "1"` are asserted beside the table.

  | File | default | `warp_alpha` 0 | `FullActiveArea` | `apply_orientation` false |
  |---|---|---|---|---|
  | Canon 5D Mark III (5760×3840) | `0x7fa38a4df1ef70fa` | same | `0xc2f1f457c0b0ff19` (5796×3870) | same as default |
  | Pixel 3a (4016×3008) | `0x4139c125eb0480cc` | `0x8d0f238b1140d37c` | refused, see below | same as default |
  | Pixel 4a (4016×3008) | `0x3a0b22d387e1fa1b` | same | `0x46fa545dcdd1fdcc` (4032×3024) | same as default |
  | Pixel 7 Pro (3866×2908) | `0xaa092016652f3fa7` | `0x8071d9cb873f04b4` | `0x6e7c387db9c1b748` (3882×2924) | `0xa41a139d6b54ae07` |
  | Nikon D300S (4288×2848) | `0xc865b3ed76b53366` | same | `0xc258fdc0f7af9434` (4320×2868) | same as default |

  The Pixel 3a under `FullActiveArea` is `UnsupportedWarpedFullActiveArea` with
  `scale_max = 1.0015292540192604`, as its mono develop is. `scratch_len` with the mix
  equals §7's numbers on every file and options set above, and on the Pixel 3a and 7 Pro
  (default options) also equals the mono develop's `scratch_len` for `mono_of(sensor)`.
- [x] **AC5, `black_levels` is not read (Tier A).** On fixture (i) (its blacks differ by
  position), the developed mix under `BlackLevels::FirstValue` and `PerPosition` is
  identical and equals the oracle; the same fixture developed as mono differs between
  the two (asserted, so the fixture can tell).
- [x] **AC6, the opcodes (`DevelopPath::MonoMix`, Tier A).** (i) A mandatory
  `FixBadPixelsConstant` in `OpcodeList1`: the mix is
  `UnsupportedRequiredOpcode { list: 1, id: 4 }` on `develop_with_into` and
  `develop_with_scratch_into`, while the same file developed as mono succeeds. (ii) A
  mandatory non-identity `WarpRectilinear`: the mix develops and equals the oracle
  (whose `mixed` comes from an optional-flag copy, the flag not touching the plane),
  while `mono_mix_plane_into` on the mandatory file is
  `UnsupportedRequiredOpcode { list: 3, id: 1 }`. (iii) An optional
  `FixBadPixelsConstant` with a 7-byte parameter block: the mix develops and
  `scratch_len` succeeds, while the mono develop and mono `scratch_len` are
  `MalformedOpcodeParams`. (iv) Fixture (v): the `GainMap`s are applied once (AC1's
  equality, whose oracle drops `OpcodeList2`). (v) `check_required_processing` with
  `DevelopPath::MonoMix` is unit-tested against the same lists: list 1 refuses every
  mandatory opcode, list 2 accepts every `GainMap`, list 3 accepts the first
  `WarpRectilinear` only; its malformed sets equal `Colour`'s.
- [x] **AC7, the four `Entry` tables.** Each gains `develop_with_into` and
  `develop_with_scratch_into` with the mix (`DevelopPath::MonoMix`), and every row passes.
- [x] **AC8, the ranked refusals (`FU-6`, Tier A).** One representative fault per rank of
  §3, each a single change to a valid fixture (i) with the mix:

  | Rank | Fault |
  |---|---|
  | F0 | `warp_alpha = NaN` |
  | F1 | weights `[1/3, 1/3, 1/4]` (`InvalidMixWeights { channel: None }`) |
  | F2 | a mandatory `FixBadPixelsConstant` (`UnsupportedRequiredOpcode { list: 1, id: 4 }`) |
  | F3 | `src` one short |
  | F4 | `Orientation = 9` |
  | F5 | `BlackLevel` above `WhiteLevel` (`InvalidLevels`) |
  | F6 | `dst` one long |
  | F7 | `FullActiveArea` with a warp whose scale leaves `[0, 1]` |
  | F8 | `StandardCrop` with a malformed `WarpRectilinear` parameter block |
  | F9 | no CFA (`NotAColorFilterArray`, a mix on a mono file) |
  | F10 | no colour matrix (`MissingColorCalibration`) |
  | F11 | `AsShotNeutral [1/100000, 1, 1]` (`MixRowOutOfRange { channel: 0 }`) |
  | F12 | scratch one short (`develop_with_scratch_into` only) |
  | F13 | a warp with a non-zero tangential term (`UnsupportedWarpTangentialTerms`) |

  - **Every pair.** For every pair `Fi`, `Fj` with `i < j` that can coexist on one input,
    `develop_with_scratch_into` (all ranks) and `develop_with_into` (all but F12) return
    `Fi`'s error, and `dst` is exactly its sentinel fill. Pairs that cannot coexist are
    F7 with F8 (opposite framings) and any pair the build finds sharing one field; the
    handback lists each skipped pair with its reason. Two faults on one field run when
    one value carries both (F10 with F11: F10 removes `ColorMatrix1`, F11 sets the tiny
    neutral; F7 with F13: one warp out of bounds and tangential).
  - **Each fault alone** gives its own error on both develop entries, with `dst` at its
    sentinel.
  - **The sizing entries.** `output_dimensions_with` returns F0, F1 and F4's errors (and
    for each pair among them, the lower rank's) and `Ok` with the unmixed dimensions for
    every other fault alone; `scratch_len` returns F0, F1, F4 and F8's errors (pairs: the
    lower rank's) and `Ok` with §7's length for every other fault alone, F2 and an
    optional malformed `FixBadPixelsConstant` included.
- [x] **AC9, the contract constants (`FU-5`).** One test,
  `developed_mix_contract_constants_are_the_computation`, ties:
  (a) the field's default: `DevelopOptions::default().mono_mix == None`, and the default
  develop of fixture (i) as mono equals `develop_into`;
  (b) `MONO_MIX_BLACK` and `MONO_MIX_WHITE`: `mono_of` takes its levels from the two
  constants, and on fixture (i) the developed mix equals the oracle where the developed
  output has at least one pixel at 65535 and one in `(32768, 65535)` (asserted), so a
  changed constant alone moves the oracle and goes red;
  (c) `MONO_MIX_VERSION == "1"` and `DEVELOP_VERSION == "8"`, asserted beside AC4's
  digests and AC11's hashes with the comment that a new value needs a bump;
  (d) a `src/develop.rs` unit test: `scratch_len` with the mix equals these literals —
  fixture (iii) without a warp `20·(519 + 8) = 10,540`; an 8 × 8 fixture with a warp
  `64 + max(64, 8·(8 + 8)) = 192`; fixture (iv) (active `18 × 1096 = 19,728`)
  `19,728 + max(19,728, 10,540) = 39,456` — and the strip constant is 512 and the ring 8
  (each named in the test).
- [x] **AC10, allocation (counted, `tests/develop_scratch_alloc.rs`).**
  `develop_with_scratch_into` with the mix makes **zero** allocations on fixtures (i),
  (ii) and (iii) under `StandardCrop` and `FullActiveArea`, Orientation 1 and 6, and on
  one representative refusal of each rank F0 to F13; `develop_with_into` with the mix
  makes exactly one allocation of `2·scratch_len` bytes. Tier B: the Pixel 7 Pro with
  scratch, zero.
- [x] **AC11, determinism on both architectures.** `tests/determinism.rs` asserts the four
  committed hashes of §8 (recorded at build on macOS arm64), equality of each with the
  §2 oracle, and that the warped pair differs from the unwarped pair; the red-proof shows
  a one-bit sample change moves all four and a one-ulp change of a warp coefficient moves
  the warped two. The CI `determinism` job passes on both legs, debug and release.
- [x] **AC12, fuzzed and panic-free.** The `demosaic` and `develop` targets reach the four
  entries with the mix as §10 says; a short `cargo +nightly fuzz run` of each (the
  toolchain brief's smoke length) finds nothing; `just lint-ci` and `just lint-fuzz` pass.
- [x] **AC13, nothing else moves.** The full suite passes with `IRRADIANCE_CORPUS_DIR` set;
  every existing determinism constant and Tier-B pin is unchanged;
  `tests/error_kind.rs` is unchanged; `just semver-check` reports only the two additions.
- [x] **AC14, the oracle can go red.** Each pre-registered mutant below, applied by hand in
  build, fails at least one named test; the handback lists which. Mutants marked
  equivalent are argued, not run.

## Failing Tests

- **`tests/developed_mix.rs`** (new; Tier A via `tests/support/dng_writer.rs`, Tier B via
  the manifest):
  - `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`
  - `ac1_hand_derived_pixels_under_orientation_6`
  - `ac2_the_option_absent_is_develop_into`
  - `ac3_strip_count_and_the_cap_of_t`
  - `ac4_held_files_match_the_oracle_and_the_digests` (Tier B)
  - `ac5_black_levels_is_not_read_under_the_mix`
  - `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels`,
    `ac6_the_mix_applies_a_mandatory_warp`,
    `ac6_the_mix_never_parses_opcode_list_1`,
    `ac6_gain_maps_apply_once`
  - `ac8_every_ranked_pair_on_both_develop_entries`,
    `ac8_each_fault_alone`, `ac8_the_sizing_entries_check_only_what_they_need`
  - `developed_mix_contract_constants_are_the_computation` (AC9 a to c)
- **`src/develop.rs` unit tests:** `ac9_scratch_len_under_the_mix_is_the_strip_formula`.
- **`src/opcode.rs` unit tests:** `ac6_mono_mix_path_applies_gain_maps_and_the_first_warp`.
- **`tests/develop_scratch_alloc.rs`:** `ac10_the_developed_mix_allocates_nothing_with_scratch`,
  `ac10_the_developed_mix_allocates_once_without`.
- **`tests/determinism.rs`:** `developed_mix_hashes_match_committed` and the extended
  `colour_determinism_red_proof` (AC11).
- **The four `Entry` tables:** two rows each (AC7).

**Rule-clause map (Design §1 to §10; `DEC-084` items 1 to 8).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §1 field, default `None` | `ac2_…`, AC9 (a) | the default develop's bytes |
| §1 `black_levels` not read | `ac5_…` | a per-position black pattern |
| §1 framing, orientation, `warp_alpha` honoured | `ac1_…` | `FullActiveArea`, `apply_orientation` false, α ∈ {0, ½, 1} |
| §2 the rule, no warp | `ac1_…`, `ac1_hand_derived_…`, AC4 Canon/Nikon/Pixel 4a | the scatter's output slot per orientation |
| §2 the rule, warp | `ac1_…` fixtures (ii), (iv); AC4 Pixel 3a/7 Pro | the `A1` window, then the mono passes |
| §2 levels are the constants | AC9 (b) | values in `(32768, 65535]` |
| §3 ranks 0 to 13 | `ac8_every_ranked_pair_…`, `ac8_each_fault_alone` | variant and the `dst` sentinel |
| §3 sizing entries | `ac8_the_sizing_entries_…` | which faults they see |
| §4 opcode set | `ac6_…` (four), the `opcode.rs` unit test, AC7 | list 1 refused, list 3 applied |
| §5 strips tile, odd starts | `ac1_…` (iii), (iv); `ac3_…` | strip edges, the even local top, the remainder |
| §7 `T`'s cap at `S + 7` | `ac3_…` | heights 519 and 520 |
| §5 resolve once | (equivalent, see (u)) | — |
| §6 no allocation | `ac10_…` | the counted allocator |
| §6 existing outputs | AC13's hashes and pins | every existing entry |
| §7 `T`, `A + max(A, T)` | `ac9_scratch_len_…`, AC4's numbers | the literal lengths |
| §8 both architectures | `developed_mix_hashes_match_committed` | the committed hashes per CI leg |
| §9 versions | AC4, AC9 (c), AC11 | the versions beside each pinned value |

**Distinguishing fixtures (pre-registered mutants).** For each: the fixture on which the
right and wrong answers differ, and why the oracle is not computed by the mutated code.
The oracle of AC1/AC4 is `mono_mix_plane_into` (whole plane, `WholePlane` sink, its own
allocations) plus the mono develop of `mono_of(sensor)` (gather for no warp; the
`normalize_active_area` pass for the warp's input): it shares with the implementation
only the kernel, the band traversal's whole-plane twin, `apply_warp_into` and
`crop_and_orient_from_active_into`, none of which this spec changes except §6's
allocation-only refactor (pinned by the existing hashes).

- (a) `A1` indexed from the plane origin, not the `ActiveArea` origin: `ac1_…` fixture
  (ii) (active left 1, top 3).
- (b) `ActiveWindow` keeps a masked column (`x < left` or `x ≥ right`), wrapping into the
  next row: fixture (ii), active left 1 and right 21 of 22.
- (c) The scatter built from `resolve_geometry(sensor)` (options ignored): `ac1_…`
  under `FullActiveArea` and `apply_orientation` false.
- (d) The no-warp strips cover the active rows but the scatter's crop is the active area
  (framing lost): `ac1_…` `StandardCrop` with crop origin `(1, 3)`.
- (e) The last strip dropped, or its end off by one: `ac1_…` (iii) and `ac3_…` at 520
  rows.
- (f) A strip's band without its halo, or its local top not even: fixture (iii) under
  `FullActiveArea` and (iv), whose strips start at the odd rows 3, 515 and 1027;
  `ac3_…` at 513 rows.
- (g) The mix reads `black_levels` (the mono `resolve_levels_with`): `ac5_…`.
- (h) The `GainMap`s applied again at stage 2 in the passes: `ac6_gain_maps_apply_once`.
- (i) `DevelopPath::Mono` used for the mix: `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels`.
- (j) `DevelopPath::Colour` used: `ac6_the_mix_applies_a_mandatory_warp`.
- (k) `OpcodeList1` parsed under the mix (develop or `scratch_len`): `ac6_the_mix_never_parses_opcode_list_1`.
- (l) The warp skipped under the mix (the colour rule): `ac1_…` (ii), (iv); AC4 Pixel 3a
  and 7 Pro.
- (m) `warp_alpha` ignored under the mix: `ac1_…` at α = ½ and 0; AC4 Pixel 7 Pro
  `warp_alpha` 0.
- (n) Weights checked after required processing: `ac8_every_ranked_pair_…` (F1, F2).
- (o) The CFA / calibration / row checked before `dst` (ranks 9 to 11 moved up):
  pairs (F6, F9), (F6, F10), (F6, F11).
- (p) `dst` written before the row check, or scratch written before the scratch-length
  check: `ac8_…`'s sentinels on F11 and F12, alone and in every pair.
- (q) `T` without the halo (`S` for `S + 7`) or without the ring: `ac9_scratch_len_…`
  literals, and AC1 (iii) with exact scratch (the core's split fails).
- (r) `scratch_len` with the mix returns the mono layout (`0` / `2A`): AC4 Canon
  (3,119,840 ≠ 0) and `ac9_…`.
- (s) `A + A + T` (no reuse of `A2` for the strips): `ac9_…` fixture (iv), 49,996 ≠ 39,456.
- (t) `output_dimensions_with` / `scratch_len` skip the weights: `ac8_the_sizing_entries_…`
  with F1 alone.
- (v) `MONO_MIX_WHITE` changed (oracle and implementation disagree): AC9 (b).
- (w) The default routes through the mix, or `develop_into` gains the field: AC2 and
  every existing pin (AC13).
- (x) §6 refactor passes the full height to a band (no local height): the existing band
  and threaded tests and hashes, and AC1 (iii).
- **Equivalent, not run:** (u) the levels and row resolved per strip instead of once
  (same values, only slower); (y) the no-warp strips over all plane rows instead of the
  framed rows (extra rows are emitted and dropped by the scatter; the halo makes every
  kept pixel the same); (z) strips of 256 in both the core and `T` together (output
  unchanged by the strip invariance; only the AC9 literals move, which is (q)'s guard).

**Moved assertions.** No behaviour changes, but these assertions grow:
- The `ENTRIES` arrays of `tests/malformed_tags.rs` (16), `tests/misshapen_tags.rs`
  (16) and `tests/unhonoured_tags.rs` (15), and `tests/required_processing.rs`' table:
  two entries each (AC7). Their `match path` helpers (`malformed_tags.rs:150`,
  `misshapen_tags.rs:162`) need a `MonoMix` arm equal to `Colour`'s.
- `tests/develop_oracle.rs:190` and `tests/ljpeg_precision_binding.rs:92` match on a
  `DevelopPath`; check whether they enumerate paths and add `MonoMix` where they do.
- Any doc test or doc comment stating "Options are mono only; the colour path takes
  none" (`src/develop.rs` module doc and `DevelopOptions`' doc): reword to say the
  colour develop takes none, and the mix option develops a colour file to mono.
- `fuzz/fuzz_targets/develop.rs`'s option-byte table (its module doc) gains bit 5.

## Non-Goals

- New geometry or resampling: the mix reuses the existing framing, orientation and warp.
- Changing `develop_into` or `develop_with_into` output without the option, including
  CFA-as-mono (SPEC-066, DEC-061).
- A colour (RGB) developed output through `develop_with_into`.
- A CFA-aware `FixBadPixelsConstant` (STAGE-034, under onyx's rules; see onyx's reply below).
- Threaded variants, a public strip size, an `irr` hook, any new `Error` variant.
- Any change to SPEC-092's sensor-geometry entries (their refusal of a mandatory warp
  included).

## onyx's reply (2026-11-26) — the open questions, settled

onyx accepted the shape: mixing before geometry makes export equal the viewport by
construction, and about 10 MB of strip scratch suits iPad.

1. **Mandatory opcodes on colour Leica files: real, and moved to STAGE-034.** The same day,
   colour Q2, Q3, SL3, M10 and M11 files were probed. The Q2, Q3 and SL3 carry mandatory
   `FixBadPixelsConstant` (list 1) and `WarpRectilinear` (list 3) and are refused by every
   colour entry. The M10 and M11 are refused earlier, at unpack (SPEC-097). onyx's rules for
   the fix are recorded in STAGE-034's Design Notes. The sensor-geometry planes must not
   refuse a mandatory warp, because the viewport applies it. `FixBadPixelsConstant` runs on
   the CFA before demosaic, in the sensor-geometry planes and in this developed mix alike.
   **This spec does not change:** it ships with list 1 refused as designed, and the
   STAGE-034 spec later lifts that refusal on every entry together, so export stays equal
   to the viewport.
2. **The recipe and `black_levels`:** accepted. The colour path's levels suit onyx, and
   `mono-core` never sets `black_levels` together with `mono_mix`. Document that it is
   ignored (§1 and AC5 already do), and add no error variant. onyx's recipe records the
   export options next to the resolved weights, `DEVELOP_VERSION` and `MONO_MIX_VERSION`.
3. **Optional `WarpRectilinear`:** apply it. The mono and mixed exports must share
   geometry. onyx asked for confirmation that `mono_mix` composes with the rest of
   `DevelopOptions`, `warp_alpha` and `Framing` in particular. It does by design (§1:
   "exactly as for the mono develop"). AC4's reference outcomes and the tier-A equality
   cover `warp_alpha` 0, ½ and 1 and both framings, which matches onyx's export default
   (alpha 1 in the Standard crop, its DEC-064) and its parity check at alpha 1, ½ and 0
   (its DEC-073).

## Notes for the Implementer

- Keep the mix out of the mono passes: the mono `crop_orient_normalize_for` and
  `normalize_active_area_for` are not touched. The mix branch replaces them by the strip
  pass (§5) and then calls the same `apply_warp_into` and
  `crop_and_orient_from_active_into`.
- `develop_core` keeps one body: add the mix's checks at their ranks (a `match` on
  `options.mono_mix` at ranks 1, 2, 5, 7/8 for `OpcodeList1`, and 9 to 11) rather than a
  second copy of the step list.
- Build `CropScatter` from the options-applied `Geometry` (a constructor taking the
  plane width and a `&Geometry`); keep `CropScatter::new(sensor)` for the colour develop.
- Use checked splits for the scratch regions, as `split_scratch` does; never index.
- The `ActiveWindow` and `CropScatter` slots must be `PixelSlot`s so the strip pass is
  one generic function.
- The determinism fixture's bytes must not change; inject the warp into the parsed
  `Sensor`'s `opcode_list_3` in memory.
- Record AC4's digests from the probe's table, not from your implementation; if your
  output differs, the implementation is wrong or the design is, and either way it is
  reported.
- Do not bump either version. Take the `Error` enum from `origin/main` at build start
  and add nothing to it.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Read the code for every fact the design states about it, and probe every number
   a red-proof depends on, before the handoff. Three details in the design were wrong
   against the code (F10 "removes ColorMatrix1", the develop fuzz target's "spare bit 5",
   and a one-ulp warp red-proof that moves no `u16` pixel); each cost the build a probe
   loop, and none reached the code. And search for a counter-example before calling a
   fault pair impossible: (F8, F13) can share one `WarpRectilinear` block.

2. **Does any template, constraint, or decision need updating?**
   — No DEC: DEC-084 holds as accepted. Three template changes, made at the STAGE-031
   close in `projects/_templates/spec.md`: the Rule-clause map now names a public
   contract constant, an order word and an "impossible" fault pair as clauses that need
   their own row (`every-rule-clause-maps-to-a-reachable-test`); the Distinguishing
   fixture paragraph says check per point and compare a perturbation with the output's
   quantum (`a-designed-mutant-must-be-able-to-fail-on-its-fixture`); and a new "Claims
   about existing code" paragraph asks for the `file:line` behind every such fact
   (`read-the-kernel-before-deciding-the-approach`, codified at N=3). The unit tests
   written after the code are a new `watch` lesson,
   `red-first-covers-only-the-integration-suite`, at N=2.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. The `demosaic` fuzz CI job is the hygiene patch already scheduled at
   the PROJ-005 close; with four specs' entries now behind it, it should land before the
   PROJ-006 release. STAGE-034 (framed in PR 357) is next.

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — Before, onyx could mix a colour Bayer DNG to mono only at sensor geometry
   (SPEC-092), so its export had to rebuild the crop, orientation, warp and resampling
   itself and could not reuse its GPU-to-develop warp parity check. Now setting
   `DevelopOptions::mono_mix` makes `develop_with_into`, `develop_with_scratch_into`,
   `output_dimensions_with` and `scratch_len` give the developed mono at the mono
   develop's geometry, bit-identical to the mono develop of the mixed plane (19 held
   digests and every `scratch_len` as registered, four hashes equal on aarch64 and
   x86_64), with no allocation in the scratch form and byte-identical output to today
   when the option is absent.

## Follow-ups

Raised in HANDOFF-204 (build, `B`) and HANDOFF-205 (verify, `V-N`); the build did not
number its findings, so they are numbered here. The verify's tests are in commit
`6abe11c`, squashed into `f6c52da`.

| id | finding | disposition |
|---|---|---|
| `FU-1` | B deviation 6, V ruling 6: only `tests/developed_mix.rs` was seen failing against the stub; the unit, allocation and determinism tests and the Entry rows were written after the code | `signal: red-first-covers-only-the-integration-suite` — new, `watch` at N=2 (SPEC-092/FU-3 is the first); every such test was shown red by a mutant, so no defect followed |
| `FU-2` | B, V-4: the `demosaic` fuzz target, which now reaches the developed mix, has no CI smoke job and no `just` recipe | `signal: fuzz-targets-without-a-ci-smoke-job` — N=3 to N=4; still the hygiene patch scheduled at the PROJ-005 close |
| `FU-3` | B: AC11's red-proof asked for one ulp of a warp coefficient, which moves no `u16` pixel; `2^-26` is the smallest power-of-two step that moves both warped hashes | `signal: a-designed-mutant-must-be-able-to-fail-on-its-fixture` — third recurrence since codification; the template's Distinguishing fixture paragraph now asks to compare a perturbation with the output's quantum |
| `FU-4` | B reflection: F10 "removes ColorMatrix1" and "spare bit 5" were stated as facts the code contradicts | `signal: read-the-kernel-before-deciding-the-approach` — N=2 to N=3, codified at the STAGE-031 close ("Claims about existing code" in the spec template) |
| `FU-5` | V-1: the design's (F8, F13) skip was wrong; one malformed `WarpRectilinear` block can carry a tangential term | `fixed` — verify, `tests/developed_mix.rs:418` `ac8_one_warp_block_both_malformed_and_tangential_is_f8`; the code was already right. The lesson tightens the Rule-clause map (`every-rule-clause-maps-to-a-reachable-test`) |
| `FU-6` | V-2: onyx's parity grid was not fully crossed (`warp_alpha` 0 and 1/2 ran only with `apply_orientation` true) | `fixed` — verify, `tests/developed_mix.rs:123` `ac1_parity_grid_composes_every_option` |
| `FU-7` | V-3 [XS]: `ac4_held_files_match_the_oracle_and_the_digests` continues past a held file missing from the corpus, so a partial corpus passes it vacuously | `signal: named-tests-can-pass-vacuously` — evidence added; the check that signal accepted (set means must run, as `tests/colour_sdk.rs` and `tests/dng_write_corpus.rs` do) closes it, at its project close. `just test`'s corpus-status lines show the gap meanwhile |
| `FU-8` | V-5 (optional): `develop_into`'s warp has no cross-architecture hash of its own; the developed-mix warped pair is the only one | `closed` — the mono develop and the developed mix share one warp, and the warped pair in `tests/determinism.rs` goes red on a warp change (V9). The two can only diverge through a change that first fails AC1's equality (`ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`), which is the trigger to add a mono hash |
