---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-030
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-007
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-016, DEC-017, DEC-018, DEC-024]    # buffer-no-alloc / red-proof / u16 repr / dnglab can't isolate demosaic
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-025, SPEC-031, SPEC-032]      # plane oracle it reads; MHC reuses this harness; fuzz

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: "STAGE-007's demosaic foundation — the camera-space RGB buffer convention + bilinear baseline + the bit-exact analytic oracle harness that SPEC-031's MHC kernel reuses unchanged."

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
  tokens_estimate: 45000000        # prediction (design): ~an [M] spec, cf. SPEC-028 52M / SPEC-029 41M grand
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-030: Demosaic foundation: the camera-space linear RGB

## Context

STAGE-007's first spec and its foundation. STAGE-006 made the Bayer **mosaic
plane** bit-exact (`plane::unpack_into`, verified vs `dnglab --raw-checksum`) and
parsed the CFA pattern into `Sensor.color_filter: Option<Cfa>`. A mosaic carries
one colour per photosite; to make an image every pixel needs all three channels.
This spec stands up the **demosaic path end to end** — the 3-channel camera-space
RGB buffer, a first interpolation kernel, the bit-exact analytic oracle, and the
`irr` surfacing — with the simplest kernel that is still a real demosaic:
**bilinear**. The quality kernel (clean-room Malvar-He-Cutler) swaps in at
**SPEC-031** against this same harness (`SPEC-031 depends_on: [SPEC-030]`).

Per the STAGE-007 frame's two recorded decisions: the oracle is **bit-exact on
analytic fixtures**, not a tolerance against a foreign demosaic — bilinear (and
later MHC) are fixed linear interpolations, so on a synthetic mosaic the output
RGB is exactly computable, and `dnglab` emits only a full colour render (DEC-024,
no isolated demosaic), so any real-image demosaic-only comparison is deferred to
STAGE-008's ColorChecker ΔE. And the output is **uncorrected camera-space linear
RGB** — no white balance, no colour matrix (STAGE-008) — so a demo image reads
greenish/flat; that is the honest STAGE-007 milestone.

## Goal

Produce a 3-channel, camera-space linear RGB image from a Bayer mosaic via a
**bilinear** demosaic driven by `Sensor.color_filter`, verified **bit-exact**
against hand-computed output on synthetic analytic fixtures (with a
perturbed-kernel red-proof), and surfaced by `irr`. Establish the RGB buffer
convention + oracle harness that SPEC-031's MHC kernel reuses unchanged.

## Design

Measured against the code on `claude/stage-007-specs` (2026-10-31) and the two
corpus files:

1. **The plane is a bare `&[u16]`, dims on `Sensor`.** There is no `Plane` struct:
   `plane::unpack_into(sensor, byte_order, file, dst: &mut [u16])` (`src/plane.rs:261`)
   writes `sensor.width * sensor.height` samples, and `develop_into(sensor, src,
   dst)` (`src/develop.rs:744`) takes the same `&[u16]`. The demosaic mirrors that
   shape exactly.
2. **The CFA phase is data, not a constant.** `Sensor.color_filter: Option<Cfa>`
   (`src/ifd.rs:598`) carries `repeat_dim: [2, 2]`, `pattern: [0, 1, 1, 2]` (RGGB;
   `0=R 1=G 2=B` via `plane_color: [0, 1, 2]`), `layout: 1`. Both corpus files are
   RGGB 2×2. Read the colour of pixel `(x, y)` from the pattern
   (`pattern[(y % rows) * cols + (x % cols)]`), so the kernel is phase-driven, not
   hard-coded to RGGB.
3. **Caller-owned output, no allocation (DEC-016).** Like `unpack_into` /
   `develop_into`, the demosaic writes a caller-owned `&mut [u16]`; the RGB buffer
   is `width * height * 3`, **interleaved** R, G, B. A wrong length is a typed
   `Error`, never a panic.
4. **Bilinear cannot overshoot.** An average of samples in `[0, 65535]` stays in
   range, so SPEC-030 needs no clamp. (MHC's gradient term CAN overshoot — SPEC-031
   clamps. Recorded so SPEC-031 does not inherit a false assumption.)
5. **`irr develop` is the surfacing precedent** (`src/bin/irr.rs:284`): it does
   `unpack_into` → `output_dimensions` → `develop_into` and prints stats. The new
   `irr demosaic` mirrors it (`unpack_into` → `demosaic_into`) and writes a **P6
   PPM** (maxval 65535, big-endian) so the result is viewable. No levels/WB are
   applied — the honest raw demosaic.

## Inputs
- **Read:** `src/plane.rs:261` (`unpack_into` — the mosaic source + buffer-length
  convention), `src/develop.rs:744` (`develop_into` — the `(sensor, &[u16], &mut
  [u16])` shape to mirror), `src/ifd.rs:598` (`Cfa` — `repeat_dim` / `pattern` /
  `plane_color`), `src/bin/irr.rs:284` (`cmd_develop` — the surfacing precedent).
- **Reuse:** `tests/support/tiff.rs` (`cfa_sensor` Build DSL — synthetic CFA files,
  no corpus), `tests/support/perturb.rs` (red-proof mutation of a crate copy,
  DEC-017), `tests/support/pnm.rs` (PNM parse, to read the demo output back).
- **External:** none for the kernel (bilinear is textbook). `dnglab` is NOT an
  oracle here (DEC-024).

## Outputs
- **`src/demosaic.rs` (new):** `pub fn demosaic_into(sensor: &Sensor, src: &[u16],
  dst: &mut [u16]) -> Result<(), Error>` — `src.len() == width*height`,
  `dst.len() == width*height*3` interleaved RGB; typed `Error` (no panic) on wrong
  lengths, on `color_filter == None` (a mono plane has nothing to demosaic), and on
  an unsupported CFA (`repeat_dim != [2,2]` / non-RGGB-family pattern — see
  Non-Goals).
- **Border policy — REFLECT, phase-preserving.** Off-image indices mirror about the
  edge pixel (`x<0 → -x`, `x≥w → 2w-2-x`; same for `y`), which preserves even/odd
  parity and therefore the Bayer phase at edges and corners. Documented on
  `demosaic_into`; the fixtures verify edge and corner pixels.
- **Bilinear kernel — exact integer arithmetic, round half up.** In `u32`, the
  known-colour channel is the sample itself; a missing channel is the average of
  its same-colour neighbours with `+half` before the divide: two neighbours
  `(a+b+1)/2`, four `(a+b+c+d+2)/4`. The four RGGB cases are enumerated in Notes.
  No overflow (`4·65535+2 < u32::MAX`); result ≤ 65535.
- **`Error` variants (new, names final at build):** e.g. `NotAColorFilterArray`
  (mono / absent CFA), `UnsupportedCfaPattern { .. }`, `RgbBufferWrongLength
  { expected, actual }`.
- **`src/bin/irr.rs`:** new `irr demosaic <file>` → `unpack_into` → `demosaic_into`
  → **P6 PPM** on stdout (maxval 65535, big-endian RGB), plus the stats line
  `cmd_develop` prints; wired into `USAGE`.
- **`tests/demosaic_oracle.rs` (new):** the analytic-fixture bit-exact oracle +
  red-proof (see Failing Tests). **Tier-A only** — synthetic, no corpus, no tools,
  so CI runs the whole thing.
- **Provenance:** bilinear is textbook (no external source); the **MHC**
  provenance-ledger entry lands with SPEC-031. A one-line "SPEC-030 = bilinear"
  note suffices here.

## Acceptance Criteria
- [ ] `demosaic_into` turns a synthetic RGGB mosaic into interleaved RGB whose
  every pixel equals the hand-computed bilinear value **exactly** (interior, edges,
  corners), for at least two hand-built fixtures
- [ ] The phase is read from `Sensor.color_filter`: the SAME code demosaics a BGGR
  (or GRBG) fixture correctly, proving RGGB is not hard-coded
- [ ] The oracle goes **red** on a perturbed kernel (a dropped neighbour or a
  changed weight): mutant output ≠ honest output, and the honest control stays
  green (DEC-017 — assert the mutation changed the OUTPUT, per `plane_oracle.rs`)
- [ ] `demosaic_into` returns a typed `Error` (no panic) on wrong `src`/`dst`
  length, on `color_filter == None`, and on an unsupported CFA pattern
- [ ] A 1×1, a 2×2, and an odd-dimension (3×5) mosaic each demosaic without panic,
  with the reflect border policy applied as documented
- [ ] `irr demosaic <corpus Bayer DNG>` emits a valid P6 PPM of `width×height` that
  a standard PNM reader opens — the first multi-channel `irr` output
- [ ] The monochrome path and every STAGE-006 oracle stay green; `src/demosaic.rs`
  adds no panic-capable call on the public API (the five panic-free lints hold)
- [ ] The oracle is Tier-A (synthetic, no corpus, no tools) so CI runs it in full

## Failing Tests
Written now; build makes them pass. All in **`tests/demosaic_oracle.rs`** unless noted.
- `"rggb_interior_matches_hand_computed_bilinear"` — a hand-built RGGB mosaic
  (values chosen so interior averages are exact); asserts the full interleaved RGB
  equals a hand-written expected table.
- `"edges_and_corners_follow_the_reflect_policy"` — asserts the border pixels equal
  the reflect-policy hand computation (a second fixture whose averages hit halves,
  pinning round-half-up).
- `"phase_is_driven_by_the_cfa_pattern"` — a BGGR fixture; asserts correct RGB,
  proving the `pattern` (not a hard-coded RGGB) drives colour assignment.
- `"a_perturbed_kernel_turns_the_oracle_red"` + `"the_honest_control_is_green"` —
  the red-proof (a mutated `demosaic` per DEC-017, or an injected neighbour swap):
  asserts mutant RGB ≠ honest RGB, and control == expected.
- `"mono_or_absent_cfa_is_a_typed_error"`, `"wrong_buffer_lengths_are_typed_errors"`,
  `"degenerate_dims_do_not_panic"` (1×1, 2×2, 3×5) — the no-panic contract.
- `"irr_demosaic_emits_a_valid_p6_ppm"` — parses the emitted header + pixel count
  (reuse `tests/support/pnm.rs`).

## Non-Goals
- **Malvar-He-Cutler** — SPEC-031 (this spec is the bilinear baseline + harness).
- **White balance, colour matrix, illuminant, display encode, ColorChecker ΔE** —
  STAGE-008.
- **Levels / black-subtract, crop, orientation.** `demosaic_into` is a
  level-agnostic pure interpolation on the mosaic it is given; the `irr` demo feeds
  the RAW unpacked mosaic (honestly un-normalised). Composing demosaic with
  levels/crop/WB into a `develop_color_into` is STAGE-008.
- **Non-2×2 / X-Trans CFA** — a typed `Error`; the Fuji 6×6 (DHT) is a later stage.
- **Real-image perceptual comparison vs `dnglab`** — deferred to STAGE-008's ΔE
  (DEC-024: dnglab can't isolate demosaic).

## Notes for the Implementer
- **The four RGGB bilinear cases** (0-indexed, row 0 = `R G R G…`, row 1 = `G B G
  B…`), every average `+half` then integer-divide, computed in `u32`:
  - **R site** (even row, even col): R = self; G = (left+right+up+down+2)/4;
    B = (4 diagonals + 2)/4
  - **B site** (odd row, odd col): B = self; G = (4 orthogonal + 2)/4;
    R = (4 diagonals + 2)/4
  - **G on an R row** (even row, odd col): G = self; R = (left+right+1)/2;
    B = (up+down+1)/2
  - **G on a B row** (odd row, even col): G = self; R = (up+down+1)/2;
    B = (left+right+1)/2
  Derive the site from `sensor.color_filter.pattern`, not from a hard-coded RGGB,
  so the BGGR fixture passes with the same code.
- Pick the interior fixture's values so every average is integer (no half), then a
  second fixture that DOES hit halves to pin the round-half-up rule.
- Red-proof: heed `tests/plane_oracle.rs`'s lesson — assert the mutation changed
  the OUTPUT (mutant ≠ honest), not merely that it compiled; a dropped neighbour or
  a `+2`→`+1` change is a genuine (non-no-op) perturbation.
- `irr demosaic`: mirror `cmd_develop`'s unwrap-policy note (caller-owned buffers,
  DEC-016). P6 PPM header is `"P6\n{w} {h}\n65535\n"`, then big-endian `u16` RGB.

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
