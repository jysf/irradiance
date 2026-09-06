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
  id: HANDOFF-035
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED from the tier_map.build prediction (claude-opus-5) —
                                    # confirmed both by this session's own system prompt and by
                                    # message.model on every entry in this session's transcript.
                                    # Standing record stands at 0 FOR 11 on the build hint
                                    # (signal `tier-map-predicts-what-it-should-record`).
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-015

project:
  id: PROJ-001
  stage: STAGE-002
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
  tokens_total: 56224398           # REAL combined count — what cost-audit reads
  estimated_usd: 24.15             # tokens_total × your rate, or your harness's number
  duration_minutes: 65
  branch: feat/spec-015-analytic-levels-and-geometry-oracle
  pr: null                         # not opened — HANDOFF-035 Return Criterion 10
  completed_at: 2026-09-05         # YYYY-MM-DD
  notes: "tokens_total/estimated_usd are per-component (input $3, output $15, 1h cache-write $6,
    cache-read $0.30 per MTok — published Sonnet rates), summed over this session's own
    transcript (69b4c29b-d5cc-4fd8-8ef7-d2da3fdf661c.jsonl, identified by scratchpad-dir uuid,
    not content match — signal orchestrator-transcript-looks-like-a-prior-attempt), deduped by
    message.id, rounded up 20% per this handoff's own instruction. Raw measured combined was
    46,853,665 (~$20.13 per-component); AGENTS.md §4's flat-rate fallback on the same raw total
    would read ~$140 — signal flat-rate-overstates-cached-sessions gained this session as its
    5th data point (~7x), so the per-component figure is reported, not the flat one."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-035: Build SPEC-015 — the analytic levels and geometry oracle

## Delegation Summary

Build `SPEC-015`. It closes `STAGE-002`. `SPEC-014` shipped the develop path and
asserted **its own arithmetic**; this spec checks that arithmetic against
expectations derived independently of how `develop_into` computes them, and
proves the check can go red.

Branch from `main` at `23087dc` (SPEC-014 merged, CI green, 143 tests).
`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` —
the default root does not exist.

## ⚠ The one idea this whole spec turns on

**An oracle that reimplements the transform is a mirror.** Written by the same
project from the same reading of the same spec, a second copy of the eight-case
orientation table fails and succeeds for exactly the same reasons as the first.
`DEC-004` already names the limit — it verifies *"the arithmetic we chose"*.

The design probe found the way out, and **measured that it works**. Read
`## The design decision this spec rests on` in the spec before writing anything.
Two rules follow from it, and they are what the acceptance criteria enforce:

1. **Never write the eight-case orientation table** (`AC3`). Assert the
   *permutation property* instead: `develop_into` rearranges the normalized crop
   window, so the output histogram must equal the histogram of that window taken
   in raster order with **no orientation applied**. Measured to hold exactly on
   the `Orientation 6` frame — with no knowledge of what 6 means anywhere in the
   check.
2. **Never derive expected values from `DEC-018`'s rounding rule** (`AC1`).
   Assert `< 0.5 LSB` from the **exact real-valued** affine map. That bound is
   satisfied by any correct rounding and violated by every wrong map, so it is a
   statement about the transform rather than about our choice. `FU-4`'s existing
   test already pins the choice — do not duplicate it.

Separate **what is forced** (endpoints — exact) from **what is chosen**
(interior rounding — bounded). That distinction is the spec.

## What is already measured — reproduce, do not re-derive

The spec's `## Implementation Context` carries all of it: the tolerance table
across three frames (max **0.499968**, **zero** pixels at or above 0.5, over
111,529,040), the truncation trap (**45.0–50.1 %** of pixels), the histogram
property holding exactly with distinct-level counts landing on the full in-range
domain (**15,872** and **16,164**), and both fault injections with their pixel
counts. The probe ran in **2.6 s** for all three frames in release.

⚠ **The tolerance is pre-registered** (`pre-register-the-tolerance`). `< 0.5`,
falsifier a single pixel at `≥ 0.5`. If you measure a max at or above 0.5, that
is a **finding, and you stop** — it is not a threshold to relax.

## Two things that make this different from a normal build

**1. You are writing a check, not a feature. `src/` is off-limits** (`AC7`).
`src/develop.rs`, `src/plane.rs` and `src/ifd.rs` must be **0 lines changed**
against `main`. If the oracle finds a real defect, that is the **most valuable
outcome this spec can have** — stop, report it, and do not adjust either side to
make it pass. An oracle edited until it agrees is worse than no oracle.

**2. The red-proof must run where CI can see it** (`AC6`). `SPEC-013/FU-1` is the
precedent *and* the warning: its red-proof genuinely works and CI has **never
once executed it**, because it needs the corpus. `SPEC-014/FU-7` measured the
same shape from the other side — four of six `tests/develop.rs` tests execute
zero assertions with the corpus absent. So `AC5`'s two faults must go red with
`IRRADIANCE_CORPUS_DIR` **unset**, over a hand-built fixture. `SPEC-013`'s
reviewer measured that shape at **1.47 s** for two cold builds, and
`SPEC-014/FU-3` used it successfully; you are not inventing a mechanism.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. `just lint-ci`, **not** `just lint` — local clippy is 0.1.97 and CI
   floats at 0.1.98; assert the version you actually linted under. **Push and
   read CI** — `constraints.yaml` requires the gate *observed* green on your SHA.
2. ⚠ **`src/` is 0 lines changed vs `main`.** Show it (`git diff --stat main...HEAD -- src/`).
3. **Watch both red-proofs fail yourself, with the corpus absent**, and paste the
   pixel counts. Every mutation: file changed **and** compiled **and** *output
   changed*. That third clause has caught four false red-proofs in three specs.
4. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --` and shipped a reconstruction. md5-verify
   every revert.
5. **`SPEC-013`'s and `SPEC-014`'s tests keep passing untouched** — 143 before,
   say what after.
6. **No fuzz target.** This adds no parser and no new input surface (spec
   Non-Goals). Say so explicitly rather than adding one; §12 bar 2 does not fire.
7. **Provenance row** — separate row, class 1, DNG 1.7. The ledger tracks
   implementations, not features, and this is a second implementation.
8. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports, **rounded up ~20 %** to cover the turns that write
   the handback — measured here at **9.9 %**, **15.4 %** and **19.2 %** low
   across three sessions, and the 20 % uplift landed the last one 3.1 % low.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block only, so
   `handback-sync` runs once cleanly. Hand-writing has caused four
   duplicate-entry cleanups.
   ⚠ **The project transcript directory also holds the ORCHESTRATOR's live
   session**, on a different model, text-matching this delegation because it
   wrote this handoff. It is **not** a prior attempt. Identify your own
   transcript by the uuid in **your own scratchpad path**, not by content match
   (signal `orchestrator-transcript-looks-like-a-prior-attempt`,
   `SPEC-014/FU-8`).
9. **Correct `handoff.to_agent`** to what your system prompt reports. Standing
   record: the build hint is **0 for 10**.
10. **Do not run `handback-sync`; do not open the PR.**
11. Findings `SB-N`/`FU-N` with proposed §15 dispositions, numbering from `FU-1`
    (this spec's own sequence). A `spec:` disposition must **name an AC that
    would fail** without it.
12. Answer §15's reflection questions in the handback.

## Out of Scope

- Anything in `src/`. See `AC7` and Return Criterion 2.
- A second copy of the orientation table (`AC3`) — the failure mode this spec
  is designed around.
- Re-asserting what `SPEC-014` already asserts. Read `tests/develop.rs` and
  `src/develop.rs`'s unit tests first; endpoints on real tags, crop dimensions,
  `Orientation` 1 and 6 dimensions, the `ActiveArea`-relative origin and the
  rounding pin are all already there.
- SSIMULACRA2 / `dnglab --srgb` / any perceptual comparison — `DEC-004` and
  `DEC-005` closed that with measurements.
- A fuzz target (Return Criterion 6).
- Opening the PR, running `handback-sync`, or touching `STAGE-002`'s close.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-015-analytic-levels-and-geometry-oracle` at `2532dc2`. No PR opened (Return Criterion 10).
- **Completed at:** 2026-09-05
- **All acceptance criteria met?** Yes, all nine (AC1–AC9). No defect found in `src/develop.rs` — the oracle's every measured number reproduces the design probe's own pre-registered `## Implementation Context` exactly (see Return Criteria reply below), so AC7's "stop and report" branch does not fire.
- **For `verify`:** N/A — this is the `build` cycle.

### Cost self-report

- **Tokens (total):** 56,224,398 (real, from this session's own transcript — see below)
- **Estimated USD:** $24.15 (per-component: input $3, output $15, 1h cache-write $6, cache-read $0.30 per MTok — published Sonnet-tier rates)
- **Duration (minutes):** ~65
- **Source of the number:** harness report — this session's own transcript file,
  `~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/69b4c29b-d5cc-4fd8-8ef7-d2da3fdf661c.jsonl`,
  identified by the **scratchpad-dir uuid** (`69b4c29b-d5cc-4fd8-8ef7-d2da3fdf661c`), not by
  content match — the same project transcript directory also holds the orchestrator's own live
  session from writing this handoff (signal `orchestrator-transcript-looks-like-a-prior-attempt`).
  Summed every `assistant` message's `usage` object, **deduped by `message.id`**: 128 unique ids,
  input 256 / output 230,579 / cache-write 470,321 (all 1h) / cache-read 46,152,509 = 46,853,665
  raw combined, 98.5% cache-read. Rounded up **20%** per this handoff's own instruction (point 8)
  to cover the remaining turns writing this handback → **56,224,398**.
  ⚠ **Priced per-component, not by AGENTS.md §4's flat-rate fallback** — that fallback
  (`tokens_total × input rate`) gives ≈$140 on the raw total here, a ~7x overstatement matching
  signal `flat-rate-overstates-cached-sessions` (N=4 before this session; this session is its 5th
  data point, added directly to that signal's evidence in this same commit's sibling file).

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-020` — the analytic oracle compares by RANK and FREQUENCY, never by position (the
    property-set design: how AC1/AC2/AC3 avoid the eight-case orientation table, and the
    rejected first attempt that got this measurably wrong)
  - `DEC-021` — the develop oracle's two red-proofs use DIFFERENT mechanisms, on purpose
    (in-process field mutation for the levels fault; `DEC-017`'s mutate-copy-rebuild-run for the
    call-site fault)
- **Deviations from spec:** None from the spec's own requirements. One implementation pivot
  worth naming: the first working version of `tests/support/oracle.rs`'s AC1/AC2 check sorted
  the full ~47-megapixel arrays (first with an `f64` comparator, then with a primitive `u16`
  one) and measured **91.78s–79.32s for a single tier-B test alone**, past AC8's pre-registered
  60s bound. Per AC8's own instruction ("propose a subsample rather than silently shipping a
  slow suite"), I looked for a cheaper mechanism instead of subsampling (subsampling would have
  weakened AC1's "every pixel" claim and AC3's "holds exactly" claim) and found one: both
  sequences are bounded to 65,536 possible `u16` values, so a frequency-table merge computes the
  identical rank-preserving pairing in O(n + 65536) instead of O(n log n). The **first version**
  of that merge was itself wrong (see Finding FU-2 below) and was caught by its own
  honest-tree assertion before it ever reached this handback. Final tier-B suite: **~15s**.
- **Follow-up work identified:** see Findings below. None rise to needing a new spec.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec itself. The one real ambiguity was mechanical, not conceptual: AC6's
   "over a hand-built fixture — the shape `SPEC-014/FU-3` used, and the shape `SPEC-013`'s
   reviewer measured as costing 1.47s" reads as one mechanism for both red-proofs. Reading both
   cited artifacts closely showed they're two different antecedents (a fixture SHAPE for one,
   a rebuild MECHANISM for the other) that happen to fit the levels fault and the orientation
   fault differently — `DEC-021` exists because working that out took real thought, and a future
   reader of this handoff shouldn't have to redo it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — `DEC-004`, `DEC-017` and `DEC-018` between them cover every load-bearing precedent this
   build needed (why no oracle may reimplement the transform, the mutate-rebuild red-proof
   mechanism, and the rounding-rule trap AC1's bound is designed around). The one thing I'd add
   for a *future* spec in this same shape: a pointer from AC8's cost bound to the general fact
   that a design-probe's `--release` measurement does not bound a debug `cargo test`'s cost for
   O(n log n)-or-worse per-pixel work — this build had to rediscover that the hard way, though it
   is arguably implied by AC8's own "a debug cargo test will be slower" line.

3. **If you did this task again, what would you do differently?**
   — Write the frequency-table/rank-merge form of the bound check FIRST, instead of starting
   with the "obvious" sort-and-zip and optimizing only after measuring AC8's bound blown. The
   O(n log n) version was never going to fit a real ~47-megapixel frame in a debug build, and the
   65,536-value-domain fact (`u16`) that makes the O(n) form possible was available from the very
   first line of the spec's own AC1 text. I would also write the rank-preserving merge's
   correctness argument (monotonicity + permutation ⇒ rank order = positional order) down
   BEFORE coding it, rather than after catching a bug in a plausible-looking-but-wrong
   simplification of it — the bug (Finding FU-2) was real but cheap to catch only because the
   honest-tree assertion happened to be strict enough to notice an impossible result.

### Findings (`SB-N` / `FU-N`, proposed §15 dispositions)

No ship-blockers. Three follow-ups, numbered from `FU-1` per this spec's own sequence:

- **`FU-1`** — This session's cost computation reproduced the existing
  `flat-rate-overstates-cached-sessions` process-debt signal's pattern exactly (a ~7x
  overstatement from AGENTS.md §4's flat-rate fallback vs. per-component pricing, squarely
  inside the signal's already-measured 2.6x–14.7x spread). **Proposed disposition:**
  `signal: flat-rate-overstates-cached-sessions` — evidence added directly to that entry
  (5th data point) in `guidance/signals.yaml`, in this build's own commit.
- **`FU-2`** — A first attempt at optimizing `tests/support/oracle.rs`'s AC1 bound check paired
  the i-th smallest *distinct* raw value against the i-th smallest *distinct* actual value
  (weighting only a secondary tally by count), rather than the i-th smallest value *counting
  repeats*. It compiled, ran fast, and was wrong: it cannot see a fault that shifts the wrong
  NUMBER of pixels into an already-valid, already-occurring bucket (exactly `AC5`'s `BlackLevel`
  fault) — only a fault that introduces an impossible value. Caught by the check's own
  honest-tree assertion reporting an impossible infinite deviation, before this handback, not by
  a reviewer. **Proposed disposition:** `signal: distinct-value-dedup-drops-multiplicity` — new
  lesson entry (`type: lesson`, `status: watch`, `N=1`) in `guidance/signals.yaml`, in this
  build's own commit; the concrete fix (`bound_check`'s rank-preserving frequency-table merge) is
  already shipped, so nothing further is owed on the code — only the pattern is worth watching
  for a second, unrelated instance.
- **`FU-3`** — This oracle's `DECODABLE` set is the three files the design probe measured
  (matching its own `111,529,040`-pixel total exactly), not `tests/plane_oracle.rs`'s four —
  `LEICA-Q2-MONO/L1026192.DNG` shares `L1021223.DNG`'s levels, geometry and orientation exactly,
  so it would add no arithmetic this oracle can observe. **Proposed disposition:** `closed` —
  already stated as a deliberate, justified scope choice in `tests/develop_oracle.rs`'s own
  `DECODABLE` doc comment; nothing further to decide.
