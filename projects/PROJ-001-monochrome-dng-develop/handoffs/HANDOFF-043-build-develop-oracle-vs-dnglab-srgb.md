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
  id: HANDOFF-043
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.build. Standing record:
                                    # 0 FOR 12 on the build hint. CORRECT THIS to what
                                    # your own system prompt reports as `message.model`.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-020

project:
  id: PROJ-001
  stage: STAGE-003
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-043: Build SPEC-020 — the develop oracle vs dnglab srgb

## Delegation Summary

Build `SPEC-020`. It ships **STAGE-003's oracle before its subject** — a
SSIMULACRA2-scored comparator against `dnglab analyze --srgb`, with a
tier-A red-proof that runs in CI without the corpus. `SPEC-018`
(WarpRectilinear, the item that most decides `PROJ-001`'s thesis)
`depends_on: [SPEC-020]` explicitly, so its verify has a check it
cannot rewrite.

Branch is already created for you: `feat/spec-020-develop-oracle-vs-dnglab-srgb`
from `main` at `b0be356` (PR #12 merged, CI green). Rebase to the tip of `main`
before you start if it moved.

`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist. The tier-A red-proof MUST run with this
UNSET (`AC7`); the optional tier-B smoke against real dnglab output needs
it.

## ⚠ The one idea this whole spec turns on

**The oracle lands before the develop pipeline.** That is not a schedule
choice, it is a design invariant — the tier-A red-proof cannot say "our
develop output scores ≥ 85 against dnglab" because there is no develop
output yet. The proof is that **the METRIC responds to the fault classes
`DEC-005` calibrated**, on inputs the harness itself produces:

- identical inputs score ≥ 99.9 (`AC3`)
- 1-pixel shift scores < 85 (`AC4`; `DEC-005` measured 62.96 at ¼ res)
- missing radial warp scores < 85 (`AC5`; `DEC-005` measured −68.05)
- gamma 1.05 scores ≥ 85 (`AC6`; `DEC-005` measured 88.51 — the falsifier's
  mirror; a threshold so tight legitimate tone divergence fails would
  train us to ignore the oracle)

The same wiring a real developed render will later go through. Read
`## The design decision this spec rests on` in the spec before writing
anything.

## What is already measured — reproduce, do not re-derive

- **The `--srgb` shape holds on Q2M files**: 19-byte `P6 8368 5584 65535\n`
  header + `w*h*2` bytes of payload = 93,453,843 bytes, verified twice
  (`docs/oracle-contract.md` § "Layer 3 is not what this document
  originally said", 2026-08-18). `{ printf 'P5 8368 5584 65535\n';
  dnglab analyze --srgb F.DNG | tail -c +20; } > ref.pgm` is the shell
  reference; `AC1` productionises it into a typed reader with the length
  assertion.
- **The `≥ 85` threshold is `DEC-005`**, pre-registered with its falsifier
  (`a missing warp must land far below 85`). The calibration table in
  DEC-005's `## Context` is the anchor for `AC4`/`AC5`/`AC6`; do not
  re-derive numbers from other sources.
- **`ssimulacra2 = "0.5.1"` is crustyimg's pinned version**, resolvable
  from crates.io. Sibling repo integration:
  `~/PSeven/experiments/crustimg_redo_plus/crustyimg/src/quality/mod.rs:109`
  — `compute_frame_ssimulacra2(reference_rgb, candidate_rgb) -> Result<f64, _>`
  taking `Rgb::new(data: Vec<[f32; 3]>, w: usize, h: usize,
  TransferCharacteristic::SRGB, ColorPrimaries::BT709)`. Read that
  file to see the call shape — but ⚠ **do not copy its `::image` dep**;
  see the next section.
- **`SPIKE-001`'s warp coefficients** for `AC5`'s fault:
  `f(r) = 0.999251 + −0.06137651·r² + −0.0939155·r⁴ + 0.0558820·r⁶`,
  optical centre `(0.5, 0.5)`, r normalised at the corner (SPIKE-001
  flagged this as unconfirmed against DNG 1.7.0.0 — verify against the
  spec if the exact numeric locations of your synthetic peak matter).

## Two things that make this different from a normal build

**1. `no image crate`, and it stays out even test-side** (`library-not-
application`; SPEC-020 `## Non-Goals`). Crustyimg's `to_ss_rgb` decodes
via `::image::DynamicImage.to_rgb8()`. Do NOT admit `::image` in
`[dev-dependencies]` — the PNM reader is hand-written (`SPEC-013`'s
`parse_raw_pixel_pgm` is the precedent), and the 16→8 sRGB conversion is
one loop:

```rust
// per pixel: sample as f32 / 65535.0 → replicate 3× → Rgb::new(...)
```

**2. The dev-dep sanction lands with this build**, per AGENTS.md §13
rule 4 (a trivial DEV-only dep + its DEC in one pass). BEFORE writing
the DEC:
- Add `ssimulacra2 = "0.5.1"` to `[dev-dependencies]` in the root
  `Cargo.toml`.
- Run `cargo deny check licenses` (both graphs — `just deny` and
  `just deny-fuzz`) and confirm the license expression comes back
  as BSD/MIT/Apache. **If ANY string in the license expression is
  gpl/lgpl/agpl/unlicense, STOP AND ASK.** This is a
  `no-copyleft-dependencies` blocker, not a routine finding.
- Only then write the DEC (`decisions/DEC-NNN-ssimulacra2-devdep-
  sanction.md`), including alternatives considered (hand-roll, shell to
  `ssimulacra2` CLI — both rejected in the spec's AC9 with the
  reasons).

## The fixture choice, pre-registered

`SPEC-020`'s `## Implementation Context` says: bilinear-quality synthetic
input, ~1024×768, must satisfy ALL FOUR invariants (`AC3`/`AC4`/`AC5`/`AC6`).
If your chosen fixture cannot satisfy all four, **stop and report** — that
is the pre-registered rule; the threshold does not move, the fixture
does. Two candidate shapes to start:

1. A downscaled natural photograph with mid-frequency detail (foliage,
   brick, hair). Closer to real develop output SPEC-018/019 will
   produce; needs a licence-clean source.
2. A synthetic gradient + textured overlay (Perlin noise on a Gaussian
   smooth field). Reproducible without attribution.

Prefer 1 if you can find a licence-clean image (CC0 preferred); use 2
otherwise. Record the choice, the source (or generation seed), and the
measured scores on each of `AC3`/`AC4`/`AC5`/`AC6` in the handback —
those numbers become the anchor a future reviewer reproduces.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across
   all targets. `just lint-ci`, **not** `just lint` — local clippy is
   0.1.97 and CI floats at 0.1.98; assert the version you actually
   linted under. **Push and read CI** — `constraints.yaml` requires
   the gate *observed* green on your SHA.
2. **`src/` is 0 lines changed vs `main`.** Show it (`git diff --stat
   main...HEAD -- src/`). This is test-side + docs + Cargo.toml only,
   per `SPEC-020/AC10`.
3. **Watch all four tier-A fault tests fail (or pass) yourself, with
   `IRRADIANCE_CORPUS_DIR` unset**, and paste the measured scores.
   Every mutation on the fixture inputs: file changed **and** compiled
   **and** *output changed*. That third clause has caught four false
   red-proofs in three specs, and one in `PATCH-002` two days ago where
   the obvious injection exercised the wrong path.
4. ⚠ **Stage your work explicitly with `git add <paths>`, never `-A`.**
   The shared checkout has taken concurrent writes this session and once
   before (`SPEC-014/FU-8`, memory `shared-worktree-has-concurrent-
   writers`). A broad `add` could sweep another writer's in-flight
   changes into your commit.
5. **Confirm `ssimulacra2` licence BEFORE writing the DEC** (see § "Two
   things that make this different"). The DEC records the actual
   licence string from `cargo deny`, not an assumption.
6. **No fuzz target.** SPEC-020 adds no library-side input surface (the
   PNM reader parses **test-side** synthetic + dnglab output). Say so
   explicitly rather than adding one; §12 bar 2 does not fire (SPEC-020
   `## Non-Goals`).
7. **Provenance row** for SSIMULACRA2 (`docs/provenance-ledger.md`).
   Class **2 — third-party permissive crate** (not class 1 — did not
   implement from the paper; not class 3 — not reimplementing from
   another impl). Module: `tests/support/ssimulacra2.rs`.
8. **Handback with a real `tokens_total` deduped by `message.id`** from
   your own transcript, priced **per-component** at the rates for the
   model `message.model` reports, **rounded up ~20 %** to cover the
   turns that write the handback — measured at 9.9 %, 15.4 %, 19.2 %
   low across three prior sessions, and 20 % uplift landed the last
   one 3.1 % low.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block
   only, so `handback-sync` runs once cleanly. Hand-writing has caused
   four duplicate-entry cleanups.
   ⚠ **The project transcript directory also holds the ORCHESTRATOR's
   live session**, on a different model, text-matching this delegation
   because it wrote this handoff. It is **not** a prior attempt.
   Identify your own transcript by the uuid in **your own scratchpad
   path**, not by content match (signal `orchestrator-transcript-looks-
   like-a-prior-attempt`, `SPEC-014/FU-8`).
   ⚠ **`notes:` must be ONE PHYSICAL LINE.** `handback-sync` truncates
   multi-line YAML scalars, leaving front matter unparseable while every
   gate reports green (`SPEC-015/FU-4` + `FU-12`; signal
   `handback-sync-truncates-multi-line-scalars`, bar 2, open).
9. **Correct `handoff.to_agent`** to what your system prompt reports.
   Standing record: the build hint is **0 for 12**.
10. **Do not run `handback-sync`; do not open the PR.** The orchestrator
    reconciles the handback against git+disk first (DEC-004 rule 1) and
    opens the PR after.
11. **Findings `SB-N`/`FU-N`** with proposed §15 dispositions, numbering
    from `FU-1` (this spec's own sequence). A `spec:` disposition must
    **name an AC that would fail** without it. A `closed:` disposition
    must name a test that will fail if the closure is wrong, not a
    person to remember (§15 "a close whose trigger is someone
    remembering is not a good close").
12. Answer §15's reflection questions in the handback.

## Out of Scope

- **Any change to `src/`.** See Return Criterion 2 and `SPEC-020/AC10`.
  If the harness surfaces a call-shape defect in `develop_into`, that
  is a **finding for a follow-up spec**, not a reason to edit here.
- **`::image` crate**, in any position (`library-not-application`).
  Return Criterion "Two things that make this different" § 1.
- **A CLI shell-out to `ssimulacra2`** (the libjxl reference binary) —
  rejected in `SPEC-020/AC9`. The runtime dep is worse than a dev-dep.
- **Scoring against real developed output**. There is no develop
  pipeline yet; that is `SPEC-018` / `SPEC-019`'s AC.
- **Full-resolution recalibration of the ≥ 85 threshold**. `DEC-005`
  marks this as a revisit condition tied to when a real render exists
  — `SPEC-018` or `SPEC-019` do it, not this spec.
- **Levels checks.** `DEC-004` + `SPEC-015` cover them; adding
  perceptual coverage for levels is a category error (`DEC-005`).
- **Fuzz target.** Return Criterion 6; §12 bar 2 does not fire.
- **The `--srgb` output for non-monochrome cameras** (`SPEC-020/Non-Goals`).
- **Opening the PR, running `handback-sync`, touching STAGE-003's
  close.** The orchestrator does those.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter
block above AND the `## Handback` section below.** An unfilled handback
is an incomplete cycle, and `just handback-sync` will tell the
orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`
   and a ONE-PHYSICAL-LINE `notes:` scalar**.
2. Fill the `## Handback` section (the reflection is part of it, not
   optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Push the branch. **Do not open the PR** (Return Criterion 10).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked
   you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct
any of this — it transcribes it. The reflection questions are part of
completion.*

### Execution notes

- **Branch / PR:** [link — branch pushed, PR not opened per Return Criterion 10]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain per-AC)
- **Fixture chosen:** [source or generation seed]
- **Measured scores on the chosen fixture:**
  - AC3 identical: [score, ≥ 99.9?]
  - AC4 1-px shift: [score, < 85?]
  - AC5 missing warp: [score, < 85?]
  - AC6 gamma 1.05: [score, ≥ 85?]
- **`ssimulacra2` licence (from `cargo deny`):** [expression]
- **`src/` diff vs main:** [`git diff --stat main...HEAD -- src/` should be empty]
- **Eleven gates + `just lint-ci` locally:** [pasted]
- **CI observed green on:** [SHA + workflow run URL]

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the
number came from. **This is the number that lands in the spec** — the
orchestrator transcribes it via `just handback-sync`, it does not
estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, per-component priced at published rates>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — ssimulacra2 dev-dep sanction (REQUIRED per AC9)
  - `DEC-MMM` — mono 16→sRGB 8 conversion, IF a non-obvious choice
- **Deviations from spec:** [list]
- **Follow-up work identified:** [any FU-N raised; propose disposition]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed
   but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
