---
# ⚠ NON-STANDARD CYCLE. `just new-handoff` accepts only build|verify
# (scripts/new-handoff.sh:31), so this file was written by hand — the same shape
# as HANDOFF-034. It is the SHIP cycle's punch-list round, delegated: SPEC-015 is
# APPROVED and sits at `cycle: ship`, and §15 lets a `fixed` disposition be
# discharged "in this spec's own cycles OR AT SHIP".
#
# ⚠ AGENTS.md §4 says ship is "main-loop, not separately metered" and therefore
# null. THAT DOES NOT APPLY HERE: this round is delegated and genuinely metered,
# so report a REAL tokens_total and say so in `notes`.
#
# ⚠⚠ `notes:` MUST BE ONE PHYSICAL LINE. handback-sync transcribes only the first
# line of a multi-line YAML scalar, which leaves an unterminated quote and makes
# the spec's whole front matter unparseable while every gate still reports green.
# That is SPEC-015/FU-4 and FU-5, measured twice — once shipped undetected for two
# days (signal `handback-sync-truncates-multi-line-scalars`).

handoff:
  id: HANDOFF-037
  cycle: ship                  # build | verify | (ship — see the note above)
  from_agent: claude-opus-5
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map, not a measurement.
                                    # Build hint is 0-for-11. CORRECT THIS to what your
                                    # own system prompt reports as `message.model`.
  from_role: architect
  to_role: implementer
  created_at: 2026-09-06
  status: pending

task:
  spec_id: SPEC-015

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handback:
  status: null
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  branch: feat/spec-015-analytic-levels-and-geometry-oracle
  pr: null
  completed_at: null
  notes: null                      # ⚠ ONE PHYSICAL LINE — see the header note
  synced_at: null
---

# HANDOFF-037: SPEC-015's ship punch list — and the one hole we are closing now

## Delegation Summary

`SPEC-015` is **✅ APPROVED at `a3f0063`** (`HANDOFF-036`, 8 follow-ups, 0
ship-blockers) and sits at `cycle: ship`. It closes `STAGE-002`.

You are **not** re-opening the review. `src/` is 0 lines changed and correct — no
mutation any cycle ran found a defect in `SPEC-014`'s shipped arithmetic. Six
follow-ups need discharging, two are already done, and **one piece of new work is
being folded in deliberately** because `STAGE-002` should not close on top of it.

Branch: `feat/spec-015-analytic-levels-and-geometry-oracle`, head `37f3893`.
`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
150 tests currently pass.

## ⚠ The new work — do this first, it is the only part that is not bookkeeping

### The hole, measured three times

**Every positional test in this repo uses a fixture of 8 pixels or fewer, so any
fault that only manifests at production size is invisible.** Not a theory:

| mutation | what it does | caught by |
|---|---|---|
| unconditional `6 → 8` mapping | 100 % of pixels positionally wrong on a real frame | 3 tests, all tier-A fixtures ≤ 6 px |
| **`6 → 8` gated on `crop_width > 100`** | **same 100 % corruption** | **nothing — 150/150 pass** |
| transposed output dimensions, size-gated (`FU-10`'s `M7`) | wrong output shape on real frames | **only** a tier-B test, and **CI never runs tier B** |

That last column is the point. CI run `34003871323` shows **0/7 corpus files
present** — the entire real-data layer of this oracle (`AC1`–`AC4`) is skipped in
the only gate `constraints.yaml` requires to be observed green. What CI actually
runs of `SPEC-015` is the two red-proofs and the tier-A properties, all on
fixtures of ≤ 8 px.

`FU-6` established that *which* permutation was applied cannot be checked by any
value-based invariant — that limit is **inherent** to `DEC-020` and stays.
`FU-10` is different and is **not** inherent: it is simply that no tier-A fixture
is big enough to cross a size gate.

### What to build

**A tier-A positional fixture at least 1024 px in its larger dimension**, added to
`tests/develop_oracle.rs` (or `tests/develop.rs` if that reads better next to
`SPEC-014/FU-3`'s test — your call, say which and why).

- **Synthetic, generated in the test.** Sample values name their own coordinates
  (`(y * width + x) as u16`, wrapping is fine and expected above 65 535 — say so
  in a comment so the wrap is not mistaken for a bug).
- **Assert positions, not histograms.** Histograms are exactly what cannot see
  this (`FU-6`). Check specific output pixels map to specific source pixels.
- **Cover both a rotating orientation and a flipping one** (e.g. 6 and 2), and
  **the output-dimension case** so `FU-10`'s `M7` is caught tier-A.
- **Cost it.** Measured rate is **0.3246 s/Mpx** serial; 1024×768 ≈ 0.79 Mpx
  ≈ 0.26 s. Report the actual figure. If your design costs more than ~2 s, say so
  and justify it.

### Prove it — three mutations, all shown red

1. `6 => (flip_x(out_y), out_x)` gated on `crop_width > 100` — the mutation that
   currently passes 150/150.
2. The same gated on `crop_width > 1000` — **your fixture must still catch it**,
   or 1024 was the wrong threshold and you should say what is.
3. `FU-10`'s transposed-dimension fault, size-gated, which today only
   `tests/develop.rs:234` (tier B, never run in CI) catches.

Each: **file changed AND compiled AND output changed.** Run them in an **isolated
copy of the crate** — `HANDOFF-036`'s reviewer did, it made `AC7` trivially true,
and this repo's memory records a concurrent-writer incident.

### ⚠ State the residual honestly

This does **not** close the class. A fault gated at `crop_width > 2000` still
evades a 1024-wide fixture, and `FU-6`'s wrong-permutation limit is inherent
regardless of size. **Say in writing what the fixture does and does not buy** —
"≤ 8 px" becoming "≤ 1024 px" is a real improvement and an incomplete one, and
the next reader must not mistake it for closure. `measurement-over-generalised`
applies to this sentence specifically.

## The eight follow-ups — dispositions decided, six left to discharge

| id | disposition | what you do |
|---|---|---|
| `FU-4` | `fixed` | **Already done** — `5b89143`. Nothing to do. |
| `FU-5` | `signal` | **Already done** — `handback-sync-truncates-multi-line-scalars`, bar 2, open. Nothing to do. |
| `FU-6` | `fixed` | `DEC-020`: add the limit to `## Consequences` **and correct `## Validation`**, whose stated remedy (Option B, sort-and-zip) is by that record's own Option D rationale provably equivalent and shares the blind spot. State the limit as **inherent**: two orientations differing only in which corner maps to the origin cannot be separated by any value-based invariant, because that correspondence *is* the table. Then note what the new fixture above does cover. |
| `FU-7` | `fixed` | One sentence in `tests/develop_oracle.rs`'s AC5(b) section and/or `DEC-021`: `AC3`'s red-proof goes red on **degeneracy** (the identity fault reads outside the crop, producing a different multiset), never on a permutation being the *wrong* permutation. The red-proof is sound; its scope is narrower than `AC3`'s name. |
| `FU-8` | `fixed` | Put **20.09 %** next to `AC2` in the spec and in the test's comment — the clipped-share break-even at which a *correct* implementation falls under the 40 % floor. In-range disagreement is structurally ~0.5006; only clipping moves the total, and `L1000622` is already at 10.05 %. Fails in the safe direction (false red), so this is diagnosis, not a threshold change. |
| `FU-9` | `fixed` | Replace "comfortably under 60 s" in `DEC-020`'s `## Validation` with **0.3246 s/Mpx** and the file count it buys: a 4th Q2M-sized file ≈ 51.4 s, a 5th ≈ 66.6 s. Headroom is **exactly one file**, and `L1026192.DNG` is that file. |
| `FU-10` | `fixed` | **The new work above is this finding's fix** — upgraded from `spec:`/`signal` because `STAGE-002` should not close over it. Also record the CI-never-runs-tier-B measurement (0/7 corpus files, run `34003871323`) as evidence on `ci-cannot-prove-bit-exactness`. |
| `FU-11` | `signal` | New `type: process-debt`. `HANDOFF-013`'s ten and `SPEC-013`'s eleven differ by four members and both are cited in shipped artifacts; `SPEC-015`'s `AC9` says "eleven" while its own Non-Goals exclude the eleventh. The fix — one named list in `AGENTS.md` §6 — is **not yours to make**; file the signal with both enumerations written out so a close can adjudicate it. |

Plus one the orchestrator raised during reconciliation:

| `FU-12` | `fixed` | **Already done** — `5b89143`. `SPEC-010` (shipped, archived) carried `FU-4`'s identical defect since 2026-09-03, through ship, `archive-spec` and three later specs, undetected. Repo-wide sweep now reads 0 of 74 front-matter artifacts failing to parse; it was 2. This is the second data point that put `FU-5`'s signal at bar. |

## Out of Scope — the orchestrator does these

- The `## Reflection` block, the `## Follow-ups` table, `cost.totals`,
  `task.complexity_actual`, `just archive-spec`, the provenance/conformance
  currency check, the `brag` entry, **and closing `STAGE-002`**.
- Opening or merging the PR. Running `handback-sync`.
- Re-opening the review, or editing `src/`. `AC7` binds you too: if the new
  fixture finds a real defect in `develop_into`, **stop and report it** — that is
  the most valuable outcome available and must not be absorbed by adjusting
  either side.
- Fixing `FU-11`'s gate enumeration. File the signal; do not edit `AGENTS.md` §6.

## Return Criteria

1. **Ten gates + `just lint-ci`**, run by you, pasted, summed across all targets,
   with the clippy version asserted (local 0.1.97; CI floats at 0.1.98 — use
   `just lint-ci`, **not** `just lint`). ⚠ `FU-11` is exactly about this count
   being ill-defined: **say which list you ran** and treat the discrepancy as the
   finding it is, not as a thing to resolve silently.
2. **Push and read CI.** Observed green on the shipping SHA, run id and job count.
3. **All three mutations watched red**, pasted, each with file-changed +
   compiled + output-changed. Run them in an isolated crate copy; `src/` in the
   working tree must be **0 lines changed vs `main`** at the end — show it.
4. **Test count**: 150 before, say what after, and the new fixture's measured
   wall-clock.
5. **The residual stated in writing** — what the fixture does not buy.
6. Handback with a real `tokens_total` deduped by `message.id`, priced
   per-component, **rounded up ~20 %** (measured 9.9 %, 15.4 %, 19.2 % low across
   three sessions here). ⚠ **`notes:` on ONE PHYSICAL LINE** — see the front-matter
   header; this is `FU-4`'s whole mechanism. Do **not** hand-write `cost.sessions`.
   ⚠ The project transcript directory also holds the **orchestrator's** live
   session on a different model; it is not a prior attempt. Identify yours by the
   uuid in **your own scratchpad path**.
7. **Correct `handoff.to_agent`** to what your system prompt reports.
8. New findings start at **`FU-13`** with proposed §15 dispositions.
9. Answer §15's reflection questions.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes it.*

### Execution notes

- **Branch / SHA:**
- **Completed at:**
- **Six follow-ups discharged?** one line each
- **Test count:** was 150, now ___
- **New fixture:** dimensions, wall-clock, where it lives and why
- **CI:** run id, job count, SHA

### The three mutations

- **M1** `6→8` gated `crop_width > 100`:
- **M2** `6→8` gated `crop_width > 1000`:
- **M3** transposed dimensions, size-gated:

### The residual — what this does NOT close

### Cost self-report

- **Tokens (total):**
- **Estimated USD:**
- **Duration (minutes):**
- **Source of the number:**

### New findings

- `FU-13` … (or "none")

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
