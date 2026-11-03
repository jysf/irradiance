---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-083
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-03
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-034

project:
  id: PROJ-002
  stage: STAGE-008
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/spec-034-verify-6466d9
  pr: 87                           # verify PR (docs-only) against main
  completed_at: 2026-11-03          # YYYY-MM-DD
  notes: "✅ APPROVED at 2fbc03f1c21b4ae30d470e1a6c11c98daa603da5. Desktop Claude Code session — /cost is unavailable here (the known recurring metering gap, memory `desktop-cost-tool-unavailable-handbacks-null`); tokens_total left null, NOT invented. Recover the real total from this verify session's transcript at ship (scratchpad session UUID 1815e3c4-005e-401b-a35b-dc96bf72a60f)."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-083: Verify — Dual-illuminant interpolation (scene-adaptive colour matrix)

## Delegation Summary

`claude-opus-5` (architect / orchestrator) hands `SPEC-034` to `claude-opus-4-8`
(acting as verifier) for the **verify** cycle: review the build cold, independently
re-derive the interpolation, prove the red-proof bites, confirm the single-illuminant
regression guard, audit DEC-034/DEC-033 adherence, run the gates, and return one verdict.
The orchestrator ships after the verdict (this session does **not** ship).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-008-colour-transform-white-balance-colour-matrix-display-encode-colorchecker-de.md`
- **Spec (the contract):** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-034-dual-illuminant-interpolation-chromatic-adaptation.md`
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-082-build-dual-illuminant-interpolation-chromatic-adaptation.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — `just test` (NOT `--all-features`);
  the Tier-A colour oracle needs no corpus; corpus tests need `$IRRADIANCE_CORPUS_DIR`.
- **Build:** `src/color.rs`, `tests/color_oracle.rs`, `docs/provenance-ledger.md`, `CHANGELOG.md`.

### Decisions that apply

- `DEC-034` — the DNG `NeutralToXY` reciprocal-temperature weight (N=3) + McCamy cubic CCT +
  committed `ILLUMINANT_CCT` table + matrix lerp + single-illuminant fallback + deferred CM-path CAT.
- `DEC-033` — the committed-constant `f64` `+ − × ÷`-only colour-transform pipeline this extends.
- `DEC-030` — the exact rationals the matrices/neutral are read from (den may be 0 → guard).
- `DEC-017` — the red-proof must mutate a REAL coefficient and assert the OUTPUT changes.

### Constraints that apply

- `no-panics-on-untrusted-input` — every divide in the weight path guarded → `MissingColorCalibration`
  or the graceful single-illuminant fallback, never a panic / NaN in `dst`.
- `oracle-must-be-shown-red` — the interpolation oracle must fail on a perturbed `ColorMatrix2`.

### Prior related work

- `HANDOFF-082` / PR #86 — the build under review (merged to `main` @ `2fbc03f`).
- `SPEC-033` / `DEC-033` — the colour transform this refines; its `src/color.rs` +
  `tests/color_oracle.rs` are the harness.

## Expected Deliverables

### The verify contract

- Independently re-derive the interpolation weight; confirm bit-exact.
- Prove the red-proof bites via ≥2 real-kernel mutations; honest control green.
- Confirm the single-illuminant fallback is byte-identical to SPEC-033.
- Audit DEC-034/DEC-033 adherence (f64 basic-ops only, once per image, no API/Error change,
  deferred CAT not added) and panic-freedom.
- Exercise the behavioral surface (`irr color` end-to-end), not just the shape.
- Run the gates cold; confirm CHANGELOG is `Changed`.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED. Do not ship.

## Out of Scope

- Shipping (merge + reflect + cost totals) — the orchestrator's job after the verdict.
- Rewriting the build's code — a verifier lists what should change, it does not change it.
- SPEC-035 (ColorChecker ΔE) and the deferred ColorMatrix-path scene-white→D50 CAT.

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-034-verify-6466d9` → PR #87 (docs-only) against `main`,
  referencing HANDOFF-083, SPEC-034, STAGE-008, PROJ-002, DEC-034/033/030/017.
- **Completed at:** 2026-11-03
- **All acceptance criteria met?** Yes — every one, independently confirmed (not just the
  build's own tests re-run). See the per-criterion evidence below.
- **For `verify`:** **✅ APPROVED — merge at commit `2fbc03f1c21b4ae30d470e1a6c11c98daa603da5`.**

**What I did, and the evidence (cold review — I did not build this):**

1. **Independently re-derived the interpolation weight.** Wrote my own Python `NeutralToXY`
   reference from SPEC-034 `## Design` + DEC-034 (3 iterations: `CM(g)` lerp → `inv(CM(g))·asn`
   → chromaticity → McCamy CCT → mired weight), not transcribed from `src/color.rs`. For the
   fixture (`CM1`/`CM2`, `asn = (½,1,½)`, illuminants A/D65) it converges to
   `g = 0.5202685520213226` — **bit-exact** to the implementation's value and the in-module
   test's hard-coded constant, through a genuine interior blend (iterations 0.299 → 0.442 →
   0.520, not a trivial clamp).
2. **McCamy, independently.** `McCamy(D65 xy) = 6504.39 K`, `McCamy(D50 xy) = 5002.10 K` — both
   within the tests' tolerances. The committed McCamy epicentre/coefficients
   (`0.3320, 0.1858; 449, 3525, 6823.3, 5520.33`) match McCamy 1992. The `ILLUMINANT_CCT`
   codes/values are the standard EXIF `LightSource` / CIE standard-illuminant CCTs
   (A=2856, B=4874, C=6774, D50/55/65/75 = 5003/5503/6504/7504, ISO studio tungsten=3200).
3. **Red-proof bites — two independent REAL-kernel mutations.** Patched the shipped
   `interpolation_weight` (NOT the test fixtures) two ways and watched the oracle go RED each
   time, honest control green, then restored the tree clean:
   - **M1 drop an iteration** (`0..3` → `0..2`): in-module `interpolation_weight_matches_dng_neutral_to_xy`
     FAILED + 3 integration oracle tests FAILED.
   - **M2 reverse the mired direction** (`(inv_t1 − 1/t)` → `(1/t − inv_t1)`): in-module
     weight-match + `weight_clamps_outside_the_calibration_range` FAILED + 4 integration tests
     FAILED (including the `a_perturbed_second_matrix_turns_the_oracle_red` sentinel itself).
   A no-op red-proof would be a REJECT (DEC-017, the `plane_oracle.rs` lesson); this one is load-bearing.
4. **Single-illuminant fallback byte-identical to SPEC-033.** Confirmed by reading the fallback
   path (`cam_to_xyz_single_illuminant` = SPEC-033's `FM1·D` / `inv(CM1)·D` exactly) and by the
   two passing tests (`single_illuminant_build_is_byte_identical_to_spec_033`,
   `single_illuminant_is_byte_identical_to_spec_033`), plus every shipped SPEC-033 fixture still
   green. `lerp_matrix` reproduces the endpoints to the bit (`g=0 → a`, `g=1 → b`).
5. **DEC-034/033 adherence.** Weight path is `f64`, `+ − × ÷` only — no `powf`/`exp`/`ln`/`sqrt`/
   `mul_add` anywhere in `src/color.rs` (McCamy written `n*n*n`). The weight is computed **once
   per image** in `build_camera_to_srgb_linear`, off `apply_matrix_into`'s per-pixel loop.
   `apply_matrix_into`, the committed `C` (`SRGB_FROM_XYZ_D65`/`BRADFORD_D50_TO_D65`) and the
   public signatures are unchanged; `lib.rs` and `src/bin/irr.rs` are untouched by the build
   commit → **no new `Error` variant, no public API change**. The deferred ColorMatrix-path
   scene-white→D50 CAT was NOT added (the `inv(CM(g))·D` path keeps SPEC-033's implicit-D50 structure).
6. **Panic-free.** Every divide is guarded: singular `CM(g)`, `xy` sum ≤ 0, `T1 == T2` (coincident
   reciprocals), McCamy `0.1858 − y == 0`, den-0 rationals, unknown illuminant code → `None`
   (graceful single-illuminant fallback) or `MissingColorCalibration`, never a panic or a NaN in
   `dst`. Confirmed by reading each guard + the `degenerate_interpolation_inputs_do_not_panic` /
   `interpolation_preconditions_that_fall_back` tests.
7. **Behavioral surface (AGENTS.md §12), not just shape.** Ran `irr color` end-to-end on BOTH
   real dual-calibration corpus frames (`$IRRADIANCE_CORPUS_DIR` set):
   - Canon EOS 5D Mk III (`5G4A9394.DNG`, illuminants 17/21, CM1+CM2+FM1+FM2) → **FM(g) path**,
     well-formed P6 PPM 5920×3950 (140,304,019 bytes = header + `w·h·3·2`), exit 0.
   - Nikon D300S (`_DSC4615.DNG`, illuminants 17/21, CM1+CM2, no FM) → **inv(CM(g)) path**,
     well-formed P6 PPM 4352×2868 (74,889,235 bytes), exit 0.
   Both frames carry two calibrations, so both now interpolate. My independent reference puts the
   scene weight at **Canon g ≈ 0.426** and **Nikon g ≈ 0.763** — genuine interior weights, so the
   scene-adaptive matrix is actually engaged on production data, not silently falling back.
8. **Gates cold.** `just test` — all binaries ok, 0 failed (the one `ignored` is the pre-existing
   SPEC-017/SB-1 corpus finding); `just lint-ci` (`-D warnings`, `--all-targets --all-features`)
   exit 0; `cargo fmt --check` clean; `just validate` 39 artifacts; `just decisions-audit` 0
   structural errors (the DEC-033/DEC-034 same-scope line is the expected advisory — DEC-034
   extends DEC-033, does not contradict). CHANGELOG `[Unreleased] → Changed`, present once,
   correctly stating values shift / public API unchanged / single-calibration byte-identical.
9. **Build matches its handback.** Built WHOLE (the pre-named [M] split was not needed) — all
   named helpers present (`interpolation_weight`, `mccamy_cct`, `cct_of_illuminant`, `lerp_matrix`,
   `xyz_to_xy`, `mul_mat_vec`) plus sensible extra decompositions (`neutral_vector`,
   `cam_to_xyz_interpolated`, `cam_to_xyz_single_illuminant`); no silent scope drop. The two
   documented deviations are sound (see below).

### Cost self-report

- **Tokens (total):** null — not available in this interface (not invented).
- **Estimated USD:** null.
- **Duration (minutes):** ~40 (estimate).
- **Source of the number:** none available. Desktop Claude Code session; `/cost` cannot be read
  here (the known recurring gap — memory `desktop-cost-tool-unavailable-handbacks-null`; the
  session-mgmt usage tool reports only the live context window, not a cumulative total).
  **Recover the real total from this verify session's transcript at ship** (scratchpad session
  UUID `1815e3c4-005e-401b-a35b-dc96bf72a60f`), as SPEC-030/031/032/033 did.

### Drift and new artifacts

- **New decisions emitted:** None. `DEC-034` (design) governs; `DEC-033`/`DEC-030`/`DEC-017`
  apply unchanged.
- **Deviations from spec (both from HANDOFF-082, both confirmed sound):**
  - `ILLUMINANT_CCT` membership — the build committed the standard illuminants with crisp CCTs
    (codes 1/3/17/18/19/20/21/22/23/24) and omitted the fuzzy daylight/fluorescent codes (ranges,
    not points) → unknown code = single-illuminant fallback (DEC-034's stated behaviour). The
    spec's illustrative "248" is not a valid EXIF/DNG `LightSource` code; reading it as `24`
    (ISO studio tungsten, 3200 K) is correct. Affects only uncovered-code handling; no test or
    corpus frame depends on it (both corpus frames use 17/21).
  - Degeneracy policy — a degenerate step *inside* the weight iteration → `None` → byte-identical
    single-illuminant fallback; a singular *final* interpolated `CM(g)` in the build →
    `MissingColorCalibration`. Both are explicitly sanctioned by the Acceptance Criteria ("each
    is `MissingColorCalibration` or the graceful fallback, no panic"); neither panics or NaNs.
- **Follow-up work identified:** None new. One trivial, non-blocking observation (NOT a punch-list
  item): `irr color`'s STDERR diagnostic still prints the SPEC-033-era label "forward_matrix
  present (FM1 path)" / "absent (inv(CM1) path)"; under interpolation the render actually uses
  `FM(g)` / `inv(CM(g))`. It is a cosmetic STDERR string, not the PPM output, not public API, and
  SPEC-034 correctly left `irr.rs` untouched (a signature change is a Non-Goal). If ever refreshed
  it belongs in a later irr-polish spec, not here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The two small ambiguities (the illustrative `ILLUMINANT_CCT` "…"/"248", and
   where a degenerate weight step lands) were already resolved and documented by the build as
   deviations; I confirmed both calls rather than rediscovering them. The spec's pre-named
   fixture-construction trick (choosing `inv(CM)` warm/cool so the scene CCT lands *between* the two
   calibrations, giving an interior `g`) made the interior-blend check easy to reason about.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-034/033/030/017` + `no-panics-on-untrusted-input` + `oracle-must-be-shown-red` fully
   covered it. The build handback's soft suggestion (a future illuminant-touching spec should name
   the authoritative EXIF/CIE source table so membership isn't a judgment call) is reasonable but
   below the bar for a signal now — the current table is sourced in-comment and audited here.

3. **If you did this task again, what would you do differently?**
   — Run the non-cargo gates (`validate`, `decisions-audit`, `fmt`) and the exiftool metadata read
   *first*, while the long `just test` compile ran in the background, rather than waiting — which is
   what I ended up doing. The real-kernel mutation red-proof (patch shipped code, run only the
   targeted colour tests, restore) is the highest-signal step per minute and worth reaching sooner.
