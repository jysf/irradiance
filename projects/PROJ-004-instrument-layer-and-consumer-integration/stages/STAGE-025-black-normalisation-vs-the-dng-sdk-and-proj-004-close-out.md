---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-025                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-19
shipped_at: 2026-11-20

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Settles the last known divergence between irradiance's develop and the DNG reference implementation (black normalisation by white minus max black vs per-position), and closes PROJ-004's carried follow-ups so the project can close with only consumer-gated stages deferred.
  delivers:
    - "A decided, documented black-normalisation rule (a DEC), measured against Adobe dng_validate on every held file whose blacks differ by position, and implemented if the decision is to change"
    - "A hardened SDK cross-check recipe and an env-gated check that fails rather than skips when switched on without its inputs"
    - "Accurate PROJ-004 records, ready for the project close"
  explicitly_does_not:
    - "STAGE-017 (public writer) or STAGE-004 (crustyimg integration): both wait on a consumer and move to the next project"
    - "Colour accuracy (PROJ-002 SPEC-035)"
    - "Running the Adobe SDK in CI, or vendoring it"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions:
    - tokens_total: 42299676
      estimated_usd: 27.69
      recorded_at: 2026-11-20
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'what's next for PROJ-004?' message at 2026-11-20T03:52Z through this close: 167 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 386 + cache-creation 395,845 (1-hour) + cache-read 41,790,041 + output 113,404); no subagents. Covers the PROJ-004 next-step read, framing STAGE-025, the DEC-070 probe and SPEC-078 design, SPEC-079 design, four handoffs, two builds and two verifies dispatched, two verify-base fast-forwards, a verifier nudge, both ships and this close. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-025: black normalisation vs the DNG SDK and PROJ-004 close-out

## What This Stage Is

PROJ-004's last stage. It has two parts.

**First, it settles one open question about correctness.** Adobe's DNG SDK normalises a
photosite as `(v − black[pos]) / (white − max(black))`. irradiance divides by
`white − black[pos]` instead. On a file whose blacks differ by position, the two differ
by 0 to 5 codes before any gain. The design probe measured this on the Pixel 4a during
STAGE-016 (DEC-069 §Context), and it also affects the Canon's 2047/2048 blacks.

The stage decides, in a DEC, which rule irradiance follows. It weighs:
- the DNG specification's own text;
- the SDK's behaviour;
- what each rule does to the image. The max-black rule keeps one gain for every
  position, and the per-position rule makes each position reach exactly 1.0 at white.

If the decision is to change, the stage implements it on both develop paths and moves
`DEVELOP_VERSION` to 7.

**Second, it closes the follow-ups PROJ-004 has carried,** so that the project can close
with only the consumer-gated stages (STAGE-017 and STAGE-004) deferred.

## Why Now

0.2.3 is out, and every monorail/onyx request has shipped. The black-normalisation gap is
the one known divergence from the DNG reference left in develop. We now have a local
`dng_validate` and a cross-check that measures it (SPEC-060), so the question can be
answered with numbers instead of argument. The remaining follow-ups are small, and
leaving them open would carry them into a project that has nothing to do with them.

## Success Criteria

- A DEC records the black-normalisation rule and why. The design probe has measured both
  rules against `dng_validate` stage 2 on every held file whose blacks differ by position
  (at least the Pixel 3a, 4a and 7 Pro and the Canon).
- **If the rule changes:**
  - both develop paths implement it;
  - `DEVELOP_VERSION` is 7;
  - every changed pin is listed and explained;
  - SPEC-060's no-gain and end-to-end bounds are tightened to the newly measured figures
    (pre-registered at design).
- **If it doesn't change:** the DEC says why, and the SPEC-060 bounds stay as documented.
- `scripts/sdk-stage2.sh` locates `OpcodeList2` by walking the IFDs, and creates its
  output directory only after the in-repo check.
- With `IRRADIANCE_SDK_STAGE2_DIR` set, a missing corpus or SDK output makes the
  cross-check fail, not skip.
- The provenance ledger carries the Adobe licence agreement's terms for the SDK reading,
  or records that none could be found.
- The PROJ-004 brief's stage count and the monorail triage table are accurate.
- Every gate is green.

## Scope

### In scope
- The black-normalisation decision (DEC-070), and its implementation if the decision is
  to change.
- The SDK recipe and env-gated-check hardening.
- The provenance licence line.
- The PROJ-004 record refresh.

### Explicitly out of scope
- STAGE-017 and STAGE-004. They are consumer-gated and move to the next project at the
  PROJ-004 close.
- The optional STAGE-019 leftovers (the O7/O8 tile reorder and the slow 24-pin test).
  They are carried to the next project unless a spec needs them.
- A non-integral `BlackLevel` above `i32::MAX`. It is carried; no held or known file has
  one.
- A release. The next 0.2.x is decided after this stage.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-078 (shipped on 2026-11-20) [M] Black normalisation by white minus the maximum black (DEC-070): the DNG 1.7.1 spec's Rescaling rule, confirmed by the SDK; colour and mono PerPosition only, FirstValue and uniform-black files byte-identical; DEVELOP_VERSION 7; Canon added to the SDK cross-check; bounds tightened (no gain within 1, end to end within 4)
- [x] SPEC-079 (shipped on 2026-11-20) [S] PROJ-004 close-out hygiene, designed 2026-11-20: sdk-stage2.sh walks the IFDs (byte-identical outputs) and creates its output dir only after the in-repo check; with the SDK variable set, a missing corpus file fails instead of skipping; the ledger records that the SDK zip ships no licence agreement document; the monorail triage record marked addressed; FU-2, tier-A fixtures with the maximum black at each slot. No src change

**Count:** 2 shipped / 0 active / 0 pending

## Design Notes

- **Carried from SPEC-078's verify (2026-11-20).**
  - **FU-2 (tests only):** a tier-A colour fixture whose maximum black is not in the last
    slot. Every tier-A colour fixture puts its largest black last, so a "`B` = last slot"
    bug is caught only by the tier-B Pixel pins, which CI does not run. It fits SPEC-079's
    hygiene scope; fold it in at SPEC-079's design.
  - **FU-1 (behaviour change):** should the four colour entries refuse `W ≤ B` with
    `InvalidLevels`, as mono `PerPosition` does? DEC-070 §3 as amended keeps colour total.
    It needs its own spec and a CHANGELOG entry, so it moves to the next project with
    STAGE-017 and STAGE-004 at PROJ-004's close.
- **Order.** The black-normalisation spec goes first. The hygiene spec touches
  `tests/gain_map_corpus.rs`, whose bounds the first spec may tighten, so the hygiene
  spec builds on the first one's merge.
- **The probe decides; nothing is assumed.** The SDK is the DNG reference
  implementation, but its choice is not automatically the specification's. The DEC
  quotes the specification text it relies on.
- **Design outcome (2026-11-20): change, by DEC-070.** The DNG 1.7.1 specification's
  Rescaling step is explicit: the scale is the inverse of `WhiteLevel` minus "the maximum
  computed black level for the sample plane". The SDK computes the same, and the probe
  brings the Canon, Pixel 3a and Pixel 4a from up to 6 codes off `dng_validate` to within 1.
  `FirstValue` stays as it is.
- **Measure the image effect, not just the code gap.** Under the per-position rule,
  every position reaches exactly white at clip. Under the max-black rule, positions with
  a lower black clip below 1.0. Check whether that matters for highlights on the held
  files.

## Dependencies

### Depends on
- STAGE-016 (SPEC-060's SDK cross-check and `scripts/sdk-stage2.sh`) and STAGE-023
  (DEC-068's per-position black reading). Both are shipped.
- External: the locally built Adobe DNG SDK 1.7.1 `dng_validate`, outside the repo.

### Enables
- The PROJ-004 close, with STAGE-017 and STAGE-004 moved to the next project.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes, both parts.
  - **Black normalisation:** DEC-070 adopts the DNG 1.7.1 specification's Rescaling rule
    (white minus the maximum black), which the SDK also follows. SPEC-078 (PR 276)
    implements it on the colour path and mono `PerPosition`; the Canon, Pixel 3a and 4a go
    from up to 6 codes off `dng_validate` to within 1 (at least 99.5% exact), Pixel end to
    end from 7 to 4. `FirstValue` and uniform-black files are byte-identical;
    `DEVELOP_VERSION` is 7. DEC-070 §3 was amended at ship: the colour path stays total.
  - **Close-out:** SPEC-079 (PR 279) hardens the SDK recipe (an IFD walk, the output
    directory created after the in-repo check) and the env-gated cross-check (set means
    must run), records that the SDK zip ships no licence agreement document, marks the
    monorail triage record addressed, and closes SPEC-078's FU-2 colour fixture gap.
- **How many specs did it actually take?** 2, as planned.
- **How many outlines survived unchanged?** 1 of 2. SPEC-078's outline held. SPEC-079's
  dropped the brief-count refresh (already accurate) and gained FU-2 from SPEC-078's verify.
- **Cost.** SPEC-078 38.72M tokens / $27.20 (estimate 25M, actual M); SPEC-079 20.98M /
  $15.47 (estimate 15M, actual S); orchestration 42.30M / $27.69 (floor). About $70.36 for
  the stage.
- **What changed between starting and shipping?** The design probe settled the question
  outright: the specification's text names the maximum black, so the stage changed the
  rule rather than documenting the gap. The verify turned up that the colour path never
  refused levels, which became a DEC-070 amendment and the carried FU-1.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - None codified. At design, list every assertion a formula change moves (N=2), and check
    each designed mutant can actually fail on its designed fixture (N=1).
- **Signals dispositioned at this close?**
  - `a-changed-formula-lists-every-assertion-it-moves`: N=2 (SPEC-059, SPEC-078); SPEC-079
    moved no assertion, so no new instance. Left `watch`.
  - New `watch` at N=1: `a-designed-mutant-must-be-able-to-fail-on-its-fixture`
    (SPEC-079's third mutant).
  - New `process-debt`: `validate-does-not-parse-feedback-or-docs`, disposition at the
    project close.
  - The other open `watch` lessons were walked; none has a new instance in this stage.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - No. Both are recorded as signals above.
- **Carried to the next project at PROJ-004's close:** FU-1 (a colour levels refusal), the
  SDK script's repo-root detection from its own location, STAGE-017 and STAGE-004.
