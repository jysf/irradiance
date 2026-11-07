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
  id: HANDOFF-100
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-047

project:
  id: PROJ-002
  stage: STAGE-012
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
  tokens_total: 195000             # REAL combined count — what cost-audit reads
  estimated_usd: 1.00              # tokens_total × your rate, or your harness's number
  duration_minutes: 20
  branch: claude/keen-sutherland-1e2389
  pr: null                         # verify PR — see ## Handback (opened after this file)
  completed_at: 2026-11-06
  notes: "APPROVED. tokens_total is this harness's per-session token meter at handback (15.00M budget minus remaining); it counts unique/context-equivalent tokens, NOT the cache-inclusive transcript sum the build reported (28.69M), so it is a floor and is not directly comparable to the build figure. No cache-inclusive /cost sum is exposed to this agent."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-100: colour path honours the ActiveArea origin (verify)

## Delegation Summary

`claude-opus-5` (acting as `architect`) hands `SPEC-047` to `claude-opus-4-8`
(acting as `verifier`) for the **verify** cycle: an independent cold review of
the build in PR #137 (DEC-040 — index the CFA phase and the BlackLevel/WhiteLevel
2×2 slot from the ActiveArea origin, offset form, no crop; `DEVELOP_VERSION` 2→3).

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-047-colour-path-honours-the-activearea-origin.md` — Acceptance Criteria + Failing Tests are the contract.
- **Decision:** `./decisions/DEC-040-index-the-cfa-phase-and-level-slot-from-the-activearea-origin.md` — offset form, and why not a crop.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-099-build-colour-path-honours-the-activearea-origin.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-040` — index the phase/level-slot from the ActiveArea origin via the
  offset `(y.wrapping_sub(active_top) & 1, x.wrapping_sub(active_left) & 1)`; no
  crop, no buffer-length change; `DEVELOP_VERSION` 2→3. **This is the spec.**
- `DEC-038` — the colour develop path + `DEVELOP_VERSION`; its ColorMatrix/
  Bradford maths must be left untouched.

### Constraints that apply

- `oracle-must-be-shown-red` — the grey red-proof must FAIL without DEC-040.
- `no-panics-on-untrusted-input` — offset panic-free for any origin.
- `test-before-implementation`, `no-new-top-level-deps-without-decision`,
  `one-spec-per-pr`.

### Prior related work

- `HANDOFF-099` (build) / `PR #137` — the build under review.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Red-proof a real red?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Behavioral surface exercised, not just shape validated (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Cropping the colour output / masked-border removal / DefaultCrop+Orientation
  parity — the deferred follow-up (DEC-040); the build must NOT change
  `develop_color_into`'s buffer lengths or dimensions.
- The colour maths (ColorMatrix/Bradford, DEC-038) and ColorChecker ΔE.
- Demosaic performance, fused-buffer/f32, threading — STAGE-012 backlog.
- Subsampled/tiled layouts.

## Return Criteria — how to hand back

Completed: `handback:` front-matter filled (with a real `tokens_total`), this
`## Handback` section filled, `handoff.status`/`handback.status` → completed,
verdict returned.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/keen-sutherland-1e2389`; verify PR
  opened against `main` (number recorded in the PR itself and reported to the
  orchestrator). Build under review: PR #137, squash-merged to `main` as
  `0e4b4dce92801b6e764e3e94677243731fa428b2` (build branch tip
  `afca83bc5130da5d56bab5dc0322fe4ff6111982`).
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** yes — all six, verified below.
- **For `verify`:** the verdict —
  **✅ APPROVED (at commit `0e4b4dce92801b6e764e3e94677243731fa428b2`).**

#### Verdict detail — ✅ APPROVED

Verified on a verify branch fast-forwarded to the shipped build (`0e4b4dc`), read
cold and run adversarially.

**Acceptance criteria — all met.**

1. **Grey stays grey across 4 parities × 4 phases** — `grey_stays_grey_across_origin_parities_and_bayer_phases`
   passes; tolerance is 8 counts of 65535 (≈1.2e-4, stricter than the spec's
   1e-3). ✓
2. **Red-proof is a REAL red** — `plane_relative_phase_tints_an_odd_origin_grey`
   develops the same odd-origin (3,4) grey mosaic through production
   `develop_color_into` twice: once with `active_area = None` (⇒ origin (0,0) =
   the pre-DEC-040 path) and once with `active_area = Some((3,4))`, then
   `assert_ne!`s them (plus guards: the plane-relative reference spread must be
   >1000, and the origin-honouring render must recover grey ≤8). I neutralized
   DEC-040 at its single source of truth (`active_area_origin` → always (0,0))
   and re-ran: this test FAILED (`assert_ne!` — the two renders collapse) **and**
   `grey_stays_grey` FAILED with a 40,716-count magenta spread, while
   `even_origin_is_byte_identical` and the no-panic test correctly stayed green.
   The red is load-bearing on production code, not a no-op/tautology. ✓
3. **Even-origin byte-identical** — `even_origin_is_byte_identical_to_plane_relative`
   passes and stays green under the revert (correct invariant). ✓
4. **`DEVELOP_VERSION` is `"3"`** — confirmed in `src/lib.rs`. ✓
5. **No panic for any origin** — `develop_color_does_not_panic_for_an_out_of_range_active_area`
   covers (8,8), (15,21) and (u32::MAX, u32::MAX); passes. `wrapping_sub` keeps
   parity well-defined above/left of the origin. ✓
6. **Determinism / no new dep / signature + buffer lengths unchanged** — pure
   integer arithmetic, no RNG; `Cargo.toml`/`Cargo.lock` untouched;
   `develop_color_into`'s signature and buffer lengths unchanged (no crop). ✓

**Fix is DEC-040 as specified.** One shared `active_area_origin(sensor) -> (u32,
u32)` helper (`src/ifd.rs`, pub(crate), `(0,0)` when absent) is the single source
of truth, used by `normalize_mosaic` (`src/color.rs`, the level slot) and
`demosaic_into` → `green_at_site` / `interpolate_red_blue` (`src/demosaic.rs`,
the CFA phase). The offset is `(coord.wrapping_sub(origin) & 1)` at every site.
Neighbour sampling stays plane-relative (only the colour label shifts), which is
correct for an offset-not-crop fix. The even-origin `pos` mapping is
bit-for-bit equivalent to the old `(row&1, col&1)` match.

**Gates — all green locally (and PR #137 CI all-green).** `just test` (0 failed;
4 new SPEC-047 tests pass; Tier-B corpus absent → skips loudly, 0/9, each file
named — the sanctioned behaviour), `just lint-ci` (clippy 0.1.98 pinned, `-D
warnings`, exit 0, zero warnings), `just fmt-check`, `just validate` (51
artifacts), `just cost-audit`, `just decisions-audit` (0 structural errors; 34
pre-existing advisory scope warnings only — DEC-040's overlaps with DEC-033/034
on `color.rs` and DEC-036 on `lib.rs` are non-contradictory: DEC-040 changes only
the index, not the colour maths, and the `lib.rs` change is the sanctioned
`DEVELOP_VERSION` bump — no supersession needed). PR #137 CI: every check passes,
including MSRV 1.90.0, clippy on both graphs, both cargo-deny graphs, the
lint-policy red-proof and the no-`#[allow]`-escape panic-free gate.

**Build reflection honesty** — the build notes name one real mechanical
deviation (grouping args into tuples to stay under clippy `too_many_arguments` on
the library path, no test-only allow) rather than claiming nothing was unclear;
consistent with the diff. No undisclosed deviations found.

### Cost self-report

- **Tokens (total):** 195000 — this harness's per-session token meter at
  handback (15.00M session budget minus remaining). It measures unique /
  context-equivalent tokens, **not** the cache-inclusive transcript sum the build
  reported (28.69M), so it is a floor and is not directly comparable to the build
  figure. A cache-inclusive /cost-style cumulative is not exposed to this agent.
- **Estimated USD:** ~1.00 (order-of-magnitude, Opus list rate on the measured
  meter; a floor for the same reason).
- **Duration (minutes):** ~20
- **Source of the number:** harness report (per-session token meter;
  `get_usage` context-window occupancy agreed at ≈180K). No `/cost` sum available.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none affecting behaviour. One mechanical change
  disclosed by the build (tuple-grouping args for `too_many_arguments`) — no DEC
  needed, no contract change.
- **Follow-up work identified:** none new. The already-planned STAGE-012 backlog
  follow-up (crop the colour output to the active area + full DefaultCrop/
  Orientation parity, DEC-040 "Out") remains the right next shape, out of this P0.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The one thing worth a second look was whether the red-proof
   was load-bearing on production or a test-local tautology; the build realized
   the "plane-relative reference" as an `active_area = None` render through real
   production code, which made the revert test decisive — a good pattern.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red`, `no-panics-on-untrusted-input` and
   `one-spec-per-pr` were the load-bearing ones and were all listed.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The single-point revert at `active_area_origin` is the
   cleanest way to prove the whole DEC-040 is load-bearing in one shot; I'd reach
   for it first next time rather than planning per-call-site reverts.
