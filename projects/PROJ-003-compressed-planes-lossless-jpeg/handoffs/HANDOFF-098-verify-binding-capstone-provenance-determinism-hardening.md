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
  id: HANDOFF-098
  cycle: verify                    # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8        # tier_map.verify — the executing agent (this session)
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-11-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-045

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 200000             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 40
  branch: claude/spec-045-verify-039bc9
  pr: null                         # verify PR against main — filled below after the PR opens
  completed_at: 2026-11-06         # YYYY-MM-DD
  notes: "Verdict APPROVED. The build is already merged to main (PR #129, squash commit a2d6376); this verify PR is a thin bookkeeping superset cut from the current main tip (the handback, the verify prompt, the timeline + cost-session update). No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-089..097). 200000 is a rounded-up reading of this verify session's context-window occupancy near handback via ccd_session_mgmt get_usage (188,916 of 1,000,000, 19% before the bookkeeping writes). A snapshot / lower bound: per-turn context resends are not captured. Not invented, but not a /cost number — treat as a FLOOR; the orchestrator supersedes it from the transcript at ship. All gates green locally with the real corpus (9/9 present, no tier-B skip): the plane-level determinism pin ran on both held files, the three header guards and the later-DHT rule pass with the build's own red-proof, the truncated/hostile/tiled strips are typed errors through the public unpack route, and the plane/develop/demosaic fuzz smokes (4.63M/3.73M/5.00M runs, 31s each) are panic-free from the committed Compression-7 seed. PR #129's CI is green on its exact head for the broad sweep that has no corpus."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-098: Binding capstone — provenance, determinism, hardening (verify)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-045` to the verifier for the
**`verify`** cycle — a cold review of the STAGE-011 capstone: the DNG
Compression-7 **binding** provenance-ledger row, a plane-level macOS/Linux
determinism pin, the lossless-JPEG header hardening the external 0.2.0 review
flagged (`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces), and continuous
fuzzing of the **public** Compression-7 route. Build is `PR #129`
(`HANDOFF-097`), already merged to `main` (squash commit `a2d6376`). This is the
**last** STAGE-011 spec — with it, STAGE-011 and PROJ-003 close.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-045-binding-capstone-a-provenance-ledger-entry-for-the.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-097-build-binding-capstone-provenance-determinism-hardening.md`
- **Prompt:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/prompts/SPEC-045-verify.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`
- **External review:** `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` (§2.1 / §2.4 folded here)

### Decisions that apply

- `DEC-036` — the dev-only `fuzzing` feature + `__fuzz_ljpeg` hook. The build
  **kept** the hook and **added** public-path coverage, dispositioning DEC-036's
  open Validation clause in prose (no new DEC). Confirmed: the hook is retained
  (`src/lib.rs` ~161, feature-gated, `#[doc(hidden)]`), the `fuzzing` feature in
  `Cargo.toml` is untouched, `ljpeg` stays `pub(crate)`, and the public route is
  now fuzzed through all three targets.
- `DEC-035` — clean-room T.81. The three header guards are Annex B/C/H
  conformance (point transform `H.1.2.1`, DC class-0 destinations `B.2.4.2`,
  redefined-table-wins `B.2.4.2`); no algorithm change.
- `DEC-003` — corpus never committed; the synthetic own-work `compression7-sof3.tiff`
  seed and the `lj92_strip` / `compression7_dng` builder are fine to commit.
- `DEC-018` — u16 modular arithmetic; the determinism argument rests on the
  integer-only decode path.
- `DEC-037` / `DEC-039` — flagged by `decisions-audit` because the stage file /
  `src/ljpeg.rs` are touched; consistent (no new multi-component colour; the
  APPn/COM skip is untouched).

### Constraints that apply

- `no-panics-on-untrusted-input` — every new guard returns a typed `Error`
  (`LjpegInvalidPointTransform`, `LjpegInvalidHuffmanTable`); the public-route
  hostile-strip and fuzz coverage confirm no panic.
- `no-new-top-level-deps-without-decision` — none added.
- `oracle-must-be-shown-red` — the later-DHT rule is red-proven against
  first-wins inside the test.

## Out of Scope

- `#[non_exhaustive]` on the public enums (SPEC-042), the opcode `MAX_TAG_VALUES`
  bound (PROJ-001), `LinearizationTable` rejection (cross-cutting, no held file),
  `tag_name` in `Error` `Display` (cosmetic) — all explicit SPEC-045 Non-Goals.
- Retiring the `__fuzz_ljpeg` hook (kept — DEC-036 Validation).
- The reconstruction algorithm, routing, bit-exact oracle (SPEC-046/043/044,
  shipped); Bayer/CFA colour multi-component and tiled decode (DEC-037).

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/spec-045-verify-039bc9`, cut from the
  current `main` tip (`7023e8e`, which already contains the merged build
  `a2d6376` and the docs-only roadmap refresh `#128`). The build is already in
  `main`, so this verify PR adds only bookkeeping — a clean thin superset.
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** yes — all seven of SPEC-045's `## Acceptance
  Criteria`; all nine `## Failing Tests` present and passing.
- **Verdict: ✅ APPROVED.** The build is already merged to `main` at commit
  `a2d63768c39bca33a17da306480bc86a25e8a141` (PR #129); no code change is
  requested.

**Acceptance-criteria confirmation (run locally with
`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, corpus
9/9 present, no tier-B skip):**

1. **Provenance.** `docs/provenance-ledger.md` carries a new DNG Compression-7
   **binding** row (`src/plane.rs` → `src/ljpeg.rs`, SPEC-043/046/044/045,
   provenance class 1, `dnglab` run-as-tool only, clean-room per DEC-035), and
   the `src/plane.rs` row's "compressed files … never decoded" note is corrected
   to the SPEC-043 routing (both DNG `Compression 7` files now decode;
   only `K3III.PEF` stays `UnsupportedCompression`). `just validate` green
   (50 artifacts).
2. **Plane-level determinism.** `compression7_plane_is_byte_identical_across_two_decodes`
   ran on **both** held files (no SKIP): two independent `unpack_into` runs into
   separately-allocated buffers (second pre-filled `0xFFFF`) are byte-identical
   and equal the pinned `dnglab` digest. The integer-only / cross-platform basis
   is documented in the test and the ledger row. `plane_oracle` 17/17 in 58s.
3. **`Pt >= P`.** `point_transform_at_or_above_precision_is_a_typed_error` — a
   new additive `Error::LjpegInvalidPointTransform { point_transform, precision }`
   checked in `parse_headers` where both are known; `Pt = P - 1` still parses;
   the shipped point-transform round-trip is unchanged. The enum is
   `#[non_exhaustive]`, so the variant is additive/semver-safe.
4. **`Tc != 0` / `Th > 3`.** `dht_with_nonzero_class_or_bad_destination_is_a_typed_error`
   — reused `Error::LjpegInvalidHuffmanTable`, guarded per table (incl.
   mid-segment); destination 3 still parses.
5. **Later DHT replaces.** `a_later_dht_with_the_same_th_replaces_the_earlier`
   — `parse_dht` overwrites the `Th` slot in place; a valid table following a
   bogus one decodes with the later; red-proven against first-wins.
   `single_table_and_shared_td_two_component_decode_unchanged` and
   `two_component_restart_is_pinned` (§2.3, ri = 1/2/3 MCUs) characterize the
   shipped Nf=1 and Nf=2 decodes as unchanged.
6. **Public-route hostile strips.** `truncated_compression7_strip_through_unpack_into_is_a_typed_error`
   drives every proper prefix of a synthetic SOF-3 strip, plus `Pt`/`Tc`/`Th`
   corruptions, through the public `plane::unpack_into` — all typed errors, no
   panic; `tiled_compression7_plane_is_unsupported_strip_layout` is a clean
   `UnsupportedStripLayout`. `compression7_synthetic_container_round_trips_through_unpack_into`
   proves the own-work builder at P = 12/14/16 both byte orders. `plane_unpack` 23/23.
7. **Public-route fuzzing.** The `require_uncompressed().is_err()` early-return
   is gone from `fuzz/fuzz_targets/{plane,develop,demosaic}.rs`; a
   `compression7-sof3.tiff` seed is present in all three seed dirs; the
   `__fuzz_ljpeg` hook + `fuzzing` feature are retained. Bounded smokes:
   `just fuzz-plane 30` 4,631,162 runs, `just fuzz-develop 30` 3,728,205 runs,
   `demosaic` (direct) 5,004,127 runs — each 31s, DONE, zero crashes.
8. **Policy and gates.** `just test` exit 0 (all 17 binaries pass, 0 failures;
   the one `ignored` is the pre-existing SPEC-017 Q2M dead-pixel measurement,
   unrelated). `just lint-ci` clean (pinned clippy 0.1.98, `--all-targets
   --all-features -- -D warnings`); `just fmt-check` clean; `just validate`
   green; `just decisions-audit --changed a2d6376^` advisory flags
   (DEC-017/029/033/035/036/037/039) all honoured, no drift, no new DEC.
   PR #129's CI is green on its exact head for the broad sweep that has no
   corpus: `rust / test`, clippy (both graphs), fmt, MSRV 1.90, cargo-deny (both
   graphs), the lint-policy and fuzz-lint red-proofs, the panic-free no-`#[allow]`
   escape job, cost-capture, and both fuzz smokes. No new dependency or public
   API; `ljpeg` stays `pub(crate)`; `#![forbid(unsafe_code)]` untouched.

### Cost self-report

- **Tokens (total):** 200,000 — **FLOOR** (context occupancy, not `/cost`).
- **Estimated USD:** null.
- **Duration (minutes):** ~40.
- **Source of the number:** `ccd_session_mgmt get_usage` read 188,916 / 1,000,000
  (19%) before the bookkeeping writes; rounded up to cover them. No `/cost` meter
  in the Code tab.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-036's Validation clause is dispositioned
  in the build handback's prose (keep the hook, add public-path coverage); no new
  DEC is warranted.
- **Deviations from spec:** the build's four documented deviations are all
  acceptable — (a) `tests/support/tiff.rs` carries its own small SOF-3 encoder
  (`lj92_strip`) because the crate's `encode_lj92` is `cfg(test)`-internal and
  invisible to integration tests, its correctness proven by the public-route
  round-trip at three precisions and both orders; (b) `two_component_restart_is_pinned`
  lives in `src/ljpeg.rs` (tier A) since the two-component encoder is
  crate-internal; (c) the Compression-7 seed was added to all three seed dirs
  (each target reads its own); (d) a pre-existing blank line that split the
  provenance-ledger table was fixed. None expand scope.
- **Observations (non-blocking, not punch-list):** the follow-ups the build
  named are real but out of scope — three hand-kept SOF-3 encoders could share
  one `tests/support/lj92.rs` (small chore), and `Error::LjpegUnsupportedComponentCount`'s
  Display still says "only single-component (Nf = 1)" plus a stale comment in
  `tests/plane_unpack.rs::compression7_planes_route_to_the_decoder` — both
  cosmetic. Candidates for the post-PROJ-003 code review / 0.2.0 cleanup, not
  this spec.
- **Follow-up work identified:** none new. SPEC-045 is the last STAGE-011 spec;
  after ship, run Prompt 1d (Stage Ship) for STAGE-011, then Prompt 1e (Project
  Ship) for PROJ-003, then the full code review and the 0.2.0 release.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — Nothing.
   The build is already merged, so the one adjustment was to cut the verify
   branch from the current `main` tip (not from a build PR head) and treat the
   verify PR as pure bookkeeping — the brief said exactly this.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** — No. `decisions-audit` surfaces DEC-017/029/033 because `src/lib.rs`
   and `tests/plane_oracle.rs` are touched, but the touches are the new `Error`
   variant / its Display and an independent determinism test — none affects those
   decisions.
3. **If you did this task again, what would you do differently?** — Start the
   full `just test` in the background at the top of the session (its tier-B
   corpus oracles buffer for ~7 minutes on their own), then run `lint-ci`,
   `fmt-check`, and the three fuzz smokes while it finishes — which is what this
   session did, and the fuzz targets' separate `fuzz/target` dir means they never
   contend with the main build lock.
