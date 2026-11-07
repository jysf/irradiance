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
  id: HANDOFF-103
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-07
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-049

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
  tokens_total: 358009             # REAL combined count — what cost-audit reads
  estimated_usd: 6.0               # tokens_total × your rate, or your harness's number
  duration_minutes: 55
  branch: claude/competent-kirch-f3ef94
  pr: 147
  completed_at: 2026-11-07         # YYYY-MM-DD
  notes: "Output-invariant fuse; u16 byte-identical (checksum 339852, oracle green), peak RSS -133.9 MiB; develop time ~2.76x slower (normalise-on-read, accepted per DEC-042). tokens_total is the harness session-usage figure (get_usage / the session token counter), not an interactive /cost cumulative readout, which is not callable as a tool in this delegated session."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-103: colour-develop memory fuse and f32 entry point

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-049` to the implementer for the
**build** cycle: fuse `develop_color_into`'s two scratch buffers into one `w*h`
green plane (normalise-on-read, apply the colour matrix per pixel into `dst`) and
add a public `develop_color_f32_into` — the `u16` output stays bit-identical and
the f32 output equals `to_linear_f32_into(develop_color_into(..))` exactly
(DEC-042).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-049-colour-develop-memory-fuse-and-f32-entry-point.md` — the Acceptance Criteria, Failing Tests and Notes are the contract.
- **Decision:** `./decisions/DEC-042-fuse-the-colour-develop-passes-and-add-an-f32-entry-point.md` — the fused two-pass structure, the normalise-on-read tradeoff, and the f32 quantise-first rule.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).
- **Code:** `src/color.rs` — `develop_color_into` (~249, the three current
  passes), `normalize_mosaic` / `normalize_sample` (~723, normalise-on-read),
  `to_srgb_linear_into` / `apply_matrix_into` (~198 / ~693, the `f64` matrix +
  `quantize`), `build_camera_to_srgb_linear` (~318); `src/demosaic.rs`
  (`demosaic_into` — pass 1 green, pass 2 red/blue; keep green in a `w*h` plane);
  `src/tone.rs` `to_linear_f32_into` (~73, the `/65535.0` the f32 path must match);
  `src/lib.rs` (export the new function).

### Decisions that apply

- `DEC-042` — the fuse + f32 entry point. **This is the spec.** Two passes, a
  single `w*h` green plane, matrix-into-`dst` in pass 2, f32 quantises-then-divides.
- `DEC-041` — the STAGE-012 perf discipline: `dst_after == dst_before` proof +
  measured peak-RSS (and time) in the provenance ledger + this DEC.
- `DEC-038` — the colour maths (ColorMatrix/Bradford). Do NOT change it; the fuse
  applies the same matrix, same order.
- `DEC-016` / `DEC-018` — caller-owned buffers; clamp/round (`quantize`) discipline.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply here:

- `no-panics-on-untrusted-input` — panic-free; no new per-read panic paths.
- `no-new-top-level-deps-without-decision` — zero new dependencies; no SIMD.
- `one-spec-per-pr` — this PR is SPEC-049 only.

### Prior related work

- `SPEC-048` / `DEC-041` (shipped, PR #142/#144) — the demosaic fast path this
  builds on (`depends_on`); keep it intact. `SPEC-047` / `DEC-040` — the
  `active_area_origin` phase offset in the same path; leave it intact.
- The pyirradiance rawpy/LibRaw benchmark (report under `docs/reviews/` / the
  pyirradiance repo) — the ~140 MB scratch and the f32-caller double-copy it flags.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-049's Acceptance Criteria.
- All failing tests in SPEC-049 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-XXX-<slug>`.
- PR description referencing: this handoff ID, the spec ID, the stage
  ID, the project ID, all referenced `DEC-*`, and any new `DEC-*`
  created during implementation.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- **Any change to the `u16` colour output, the colour maths (`DEC-038`), or the
  demosaic kernel** — bit-identity is the gate; if the `u16` output moves, stop.
- Changing `develop_color_into`'s existing signature — the f32 path is a NEW
  function (`develop_color_f32_into`), additive only.
- The colour-geometry crop follow-up (§3B) and P3 threading — separate specs.
- `unsafe`, SIMD, new dependencies; a one-row normalised cache (a possible later
  optimisation if the normalise-on-read recompute regresses time — note it, don't
  build it here).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/competent-kirch-f3ef94` → PR #147 (against `main`).
- **Completed at:** 2026-11-07
- **All acceptance criteria met?** yes.
  - `develop_color_into`'s `u16` output is **byte-identical** to the pre-fuse
    `normalize → demosaic_into → matrix` composition, proven across all four
    Bayer phases by `tests/color_oracle.rs::fused_develop_color_is_byte_identical_to_the_reference`
    (an independent reference pipeline), and corroborated by the first-9-sample
    checksum (`339852`) being unchanged across the fuse. The existing colour
    goldens — the compose test, SPEC-047 grey-stays-grey across every origin
    parity and Bayer phase, the even-origin regression guard, and the SPEC-034
    dual-illuminant set — all stay green.
  - The `normalized` and `camera_rgb` scratch buffers are gone; only the `w·h`
    green plane remains. Measured peak RSS at 23.4 MP (5712×4096) falls from
    **376,143,872 B (358.7 MiB)** to **235,749,376 B (224.8 MiB)**, a
    **−140,394,496 B (−133.9 MiB)** drop — within ~16 KiB of the exact size of
    the eliminated `camera_rgb` buffer (`5712·4096·3·2 = 140,378,112 B`). Row
    recorded in `docs/provenance-ledger.md`'s STAGE-012 table.
  - `develop_color_f32_into` exists, is exported through `pub mod color`, and
    equals `to_linear_f32_into(develop_color_into(..))` **to the bit** for every
    Bayer phase (`develop_color_f32_equals_to_linear_of_u16`): it quantises to
    `u16` first (the identical `quantize`, `DEC-018`) then divides by `65535.0`.
  - Panic-free (the five-lint `-D warnings` policy holds via `just lint-ci`),
    deterministic (committed-constant `f64` basic ops, integer normalise, no
    threading/SIMD), zero new dependencies. `develop_color_into`'s signature is
    unchanged; `develop_color_f32_into` is the only public-surface addition.
- **For `verify`:** n/a (build cycle).

### Cost self-report

- **Tokens (total):** 358009
- **Estimated USD:** ~6.0 (order of magnitude; Opus list rate, no cache
  discount, no input/output split)
- **Duration (minutes):** ~55
- **Source of the number:** harness report — the session-usage API
  (`get_usage`, which matches this session's token counter). An interactive
  `/cost` cumulative readout is not available as a callable tool in this
  delegated session; this figure is the harness's own session token accounting,
  reported rather than invented. If the orchestrator has the true `/cost`
  cumulative at ship, prefer it.

### Drift and new artifacts

- **New decisions emitted:**
  - None. The build followed `DEC-042` as written.
- **Deviations from spec:**
  - The fused traversal lives in `src/demosaic.rs` as a new `pub(crate)`
    `develop_color_fused` (reusing the existing private Hamilton-Adams kernel +
    reflect helpers verbatim), with `src/color.rs` supplying the normalise-on-read
    accessor and the matrix-applying sink. The spec framed the change as "in
    `src/color.rs`"; splitting the kernel-adjacent traversal into `demosaic.rs`
    keeps the bit-identity guarantee structural (the same kernel helpers feed
    both paths) and leaves the shipped `demosaic_into` untouched. No behavioural
    deviation — the `u16` output is byte-identical.
  - The fused path uses the reflecting border logic for **every** pixel (not the
    SPEC-048 interior fast path). Output is identical (the reflecting path is
    bit-identical to the fast path, `DEC-041`); this is the memory-first choice
    `DEC-042` anticipates, and it is the main driver of the time regression
    below. `demosaic_into`'s own fast path is untouched and intact.
- **Follow-up work identified:**
  - **Develop time regressed ≈2.76× on the colour path** (243 ms → 671 ms at
    23.4 MP) because removing the normalised scratch re-normalises each photosite
    on every neighbour read (up to ~9×) and the fused path forgoes the interior
    fast path. This is exactly the tradeoff `DEC-042` names ("memory, not speed")
    and explicitly defers. A follow-up spec in STAGE-012's backlog should add the
    **one-row normalised cache** `DEC-042` describes (and/or a normalise-on-read
    interior fast path that computes slot parity by XOR instead of per-read), to
    recover the time while keeping the single green plane. Not built here per the
    handoff's Out-of-Scope.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The one judgement call the spec left open was *where* the
   fused traversal should live: the spec said "in `src/color.rs`", but the
   Hamilton-Adams kernel and reflect helpers are private to `src/demosaic.rs`.
   Reaching for them from `color.rs` would have meant either duplicating the
   kernel (a bit-identity hazard) or widening its visibility; putting the fused
   traversal beside the kernel it reuses was the lower-risk reading and the spec
   did allow `demosaic.rs` changes ("keep green in a `w·h` plane").

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-042`, `DEC-041` (perf discipline), `DEC-038` (untouched colour
   maths), `DEC-040` (ActiveArea-origin phase) and `DEC-018` (quantise) were all
   named and sufficient. The error-ordering subtlety (CFA error before
   calibration error) was not spelled out but was easy to preserve by building
   the matrix inside the sink setup, after the CFA check.

3. **If you did this task again, what would you do differently?**
   — Measure the baseline peak-RSS *and* time first (I did), but I would also
   have measured the time cost of normalise-on-read against the fast path up front
   so the ≈2.76× regression was a known number at design, not a build-time
   discovery — it is fully within `DEC-042`'s stated tolerance, but measuring it
   earlier would have let the one-row-cache follow-up be framed as a sibling spec
   from the start.
