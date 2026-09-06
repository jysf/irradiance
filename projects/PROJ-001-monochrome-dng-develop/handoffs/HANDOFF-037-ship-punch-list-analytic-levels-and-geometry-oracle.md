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
  to_agent: claude-sonnet-5           # CORRECTED — this system prompt's own model id.
  from_role: architect
  to_role: implementer
  created_at: 2026-09-06
  status: completed

task:
  spec_id: SPEC-015

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handback:
  status: completed
  tokens_total: 21459636           # rounded up ~20% from a measured 17,883,030
  estimated_usd: 9.05              # rounded up ~20% from a measured $7.54
  duration_minutes: 25
  branch: feat/spec-015-analytic-levels-and-geometry-oracle
  pr: null                         # not opened — Out of Scope
  completed_at: 2026-09-06
  notes: "FU-10's new work done: two tier-A positional tests at tests/develop_oracle.rs (1024x768, synthetic, orientations 6+2), 152 tests (was 150), all three mutations (M1 crop_width>100, M2 crop_width>1000, M3 transposed dimensions) watched RED in an isolated crate copy with file-changed+compiled+output-changed evidence, src/ 0 lines changed vs main (git diff --stat empty, md5s match HANDOFF-036's recorded values). FU-6/FU-7/FU-8/FU-9 discharged as fixed in DEC-020/DEC-021/the spec/the test comments; FU-11 not touched (Out of Scope, reported not resolved). Residual stated in the Handback prose: a >2000 gate still evades this fixture, only orientations 2+6 covered, FU-6's rank/frequency blind spot is untouched and inherent. Cost is a transcript sum deduped by message.id from THIS session's own JSONL (638c1488-fc61-4a2f-a31f-a8118ef08c7e.jsonl, identified by the scratchpad-dir uuid, not content match): 154 usage objects / 87 unique ids, all message.model=claude-sonnet-5, raw combined 17,883,030 (input 174 / output 59,055 / cache-read 17,594,660 / cache-write-1h 229,141 / cache-write-5m 0), priced per-component at published Sonnet rates ($3/$15/$6-1h/$0.30-read) = $7.54, both figures rounded up ~20% per this handoff's own point 7."
  synced_at: 2026-09-06
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

- **Branch / SHA:** `feat/spec-015-analytic-levels-and-geometry-oracle` at `47094cc`. No PR opened, `handback-sync` not run, `STAGE-002` not closed (all Out of Scope).
- **Completed at:** 2026-09-06
- **Six follow-ups discharged? one line each:**
  - `FU-6` — `DEC-020`'s `## Consequences` now states the wrong-permutation blind spot as INHERENT (measured twice: unconditional and size-gated 6→8, both 100.0% positionally wrong, both invisible to rank/frequency); `## Validation`'s "Wrong if" clause corrected — Option B (sort-and-zip) is provably equivalent to the shipped merge and shares the blind spot, so it is no longer offered as a remedy.
  - `FU-7` — one sentence added at `tests/develop_oracle.rs`'s AC5(b) section and to `DEC-021`'s `## Consequences`: the orientation red-proof's injected fault reads outside the crop window (a different multiset — degeneracy), so `AC3`'s red-proof is sound but proves less than its name suggests; it never exercises "the wrong permutation."
  - `FU-8` — 20.09% (the clipped-share break-even) recorded next to `AC2` in the spec and in `every_pixel_is_within_half_an_lsb_of_the_exact_affine_map`'s AC2 comment.
  - `FU-9` — `DEC-020`'s `## Validation` "Right if" clause replaced "comfortably under 60s" with the measured 0.3246 s/Mpx and the file-count consequence (4th ≈51.4s, 5th ≈66.6s; `L1026192.DNG` is the 4th).
  - `FU-10` — the new work below.
  - `FU-11` — not mine to fix (Out of Scope); the gate list ambiguity is reported honestly under "Gates run," not silently resolved.
- **Test count:** was 150, now **152** (2 new tier-A tests).
- **New fixture:** `rotating_orientation_is_positionally_correct_at_production_scale` / `flipping_orientation_is_positionally_correct_at_production_scale`, `tests/develop_oracle.rs` (appended after the existing red-proof/control section) — chosen over `tests/develop.rs` because this is `SPEC-015`'s own finding (`DEC-020`'s blind spot), the module doc already discusses it, and `DEC-020`/`DEC-021` are the decisions being amended in the same commit. 1024x768 (786,432 px), synthetic, generated in-test (`sample(x,y) = (y*width+x) as u16`, wrapping past 65,535 by design). Measured wall-clock: `develop_into` itself 0.157s (rotate) / 0.164s (flip); full two-test run 0.19s; the whole `develop_oracle` target (9 tests, corpus absent) 0.91s — both well under the ~2s budget.
- **CI:** run `34015901883`, headSha `47094ccbd31bce02d11a3ea5277dd25bab43212e`, **9/9 jobs success** (fmt, clippy, test, MSRV 1.90.0, license policy x2, lint-red-proof, panic-free/#[allow] policy, cost-capture audit).

### The three mutations

All three run in an **isolated copy** of the crate (`rsync`'d to the session scratchpad, excluding `target/`/`.git`), confirmed by md5 against `src/develop.rs` before (`8c2fc59a...`, matching `HANDOFF-036`'s own recorded value) and after each mutation (a different hash every time — file genuinely changed). Each mutation ran only the two new tests (`--test develop_oracle _positionally_correct_at_production_scale`), except M3 which ran the full `develop_oracle` target to also confirm the other 8 tests stay green (the mutation is isolated to the transposed-dimension bug).

- **M1** `6→8` gated `crop_width > 100`: **RED.** File changed (md5 `77ac0f06...`), compiled (`Finished` in 4.34s), output changed — `rotating_orientation_is_positionally_correct_at_production_scale` FAILED, `786432/786432` pixels positionally wrong (100.0%), `flipping_...` (orientation 2, untouched by this arm) still green.
- **M2** `6→8` gated `crop_width > 1000`: **RED.** File changed (md5 `9cc32a5f...`), compiled, output changed — identical failure, `786432/786432` (100.0%) wrong. Confirms 1024 was not too small a fixture for the STRICTER gate `HANDOFF-037` specifically asked to be tested against.
- **M3** transposed `output_dimensions` for orientations 5-8, gated `crop_width > 100`: **RED.** File changed (md5 `4e251f3b...`), compiled, output changed — `rotating_...`'s dimension assertion FAILED: `output_dimensions` returned `(1024, 768)` instead of the correct `(768, 1024)`. The other 8 tests in the target (including `flipping_...`, orientations 1-4 unaffected) stayed green, confirming the mutation is isolated to the 5-8 transpose path.

Scratch mutant copies deleted after use. Real working tree confirmed unaffected throughout: `git diff --stat main...HEAD -- src/` empty, and `src/develop.rs`/`plane.rs`/`ifd.rs`/`lib.rs` md5s unchanged from `HANDOFF-036`'s recorded values (`8c2fc59a.../2b86d470.../56d43e6f.../00b13c4e...`) at the end of this session.

### The residual — what this does NOT close

Read this before treating `FU-10` as closed:

1. **The size gate is bounded, not eliminated.** A fault written as `crop_width > 2000` (or any `N` larger than 1024) still evades this fixture. No finite fixture dominates every possible gate constant — this raises the floor a working positional fixture needs from 8px to 1024px; it does not remove the concept of "big enough to hide behind."
2. **Only two of eight orientations are covered.** `rotating_...` exercises orientation 6, `flipping_...` exercises orientation 2. Orientations 3, 4, 5, 7, 8 have no positional coverage above 8px anywhere in this repo.
3. **`FU-6`'s wrong-permutation blind spot in `DEC-020`'s own rank/frequency techniques is untouched and INHERENT**, independent of size — `bound_check` and `multiset_equal` cannot distinguish one valid permutation from another because that correspondence *is* the eight-case table, on a fixture of any size. This fixture backstops that limit only by being a different kind of check (positional), not by closing the limit itself.
4. **1024px is still far short of a real frame** (Q2M is ~8400x5600). A fault that only manifests statistically or numerically at multi-megapixel scale (as opposed to a simple size-gate on `crop_width`) is not what this fixture was built to catch and may not be caught by it.

`20.09%` becoming a known number (`FU-8`) and `0.3246 s/Mpx` becoming a known number (`FU-9`) are real, narrow improvements, same as this one — none of the three is a general fix for the class of "a real-scale property no <=8px fixture can exercise."

### Cost self-report

- **Tokens (total):** 21,459,636 (rounded up ~20% from a measured 17,883,030)
- **Estimated USD:** $9.05 (rounded up ~20% from a measured $7.54)
- **Duration (minutes):** ~25
- **Source of the number:** This session's own transcript, identified by the scratchpad-dir uuid (`638c1488-fc61-4a2f-a31f-a8118ef08c7e.jsonl`), not by content match — the project transcript directory also holds the orchestrator's live session on a different model and would text-match this delegation because it wrote the handoff. Deduped by `message.id`: 154 usage objects / 87 unique ids, all `message.model = claude-sonnet-5`. Raw combined 17,883,030 (input 174 / output 59,055 / cache-read 17,594,660 = 98.4% / cache-write-1h 229,141 / cache-write-5m 0), priced PER-COMPONENT at published Sonnet rates ($3 input / $15 output / $6 1h-cache-write / $0.30 cache-read per MTok) = $7.54, then both figures rounded up ~20% per this handoff's own instruction (point 7), to cover the commit/push/CI-watch tail after this measurement was taken.

### New findings

- none. No mutation run this session found a defect in `develop_into`, `output_dimensions`, or `crop_source_coords` — every mutation was a deliberately injected fault, and `src/` is 0 lines changed vs `main` throughout (verified by `git diff --stat` and md5, both above). `FU-11`'s gate-count ambiguity is pre-existing and explicitly not mine to resolve (Out of Scope) — reported under "Gates run," not filed as new.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing major. The one genuine judgment call was where the new fixture should live (`tests/develop_oracle.rs` vs `tests/develop.rs`) and what mechanism `M3` (the transposed-dimension mutation) should actually take — `HANDOFF-036`'s prose describes M7's *effect* ("same pixel count, wrong shape") but not its exact code shape, so I had to choose one (gating `oriented_dimensions`' 5-8 arm to return `(width, height)` instead of `(height, width)`) that matches the description and is catchable by a direct `output_dimensions` assertion.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The one thing worth naming for the next reader: a positional test for `Orientation` necessarily encodes the same per-case formula `DEC-020`/AC3 forbid the RANK/FREQUENCY oracle from encoding, and the handoff didn't call out that this is fine (it's a different mechanism answering a different, permutation-specific question) versus a violation of AC3's discipline. I added an explicit comment distinguishing the two in the test file and in `DEC-020`'s `## Consequences` so a future reader doesn't flag the new tests as reintroducing the mirror `SPEC-015` exists to avoid.

3. **If you did this task again, what would you do differently?**
   — Nothing procedurally — writing the fixture, proving it red under all three mutations in an isolated copy, and updating `DEC-020`/`DEC-021`/the spec in the same pass (rather than splitting decision-record fixes from the code that motivates them into a later round) all worked cleanly. If I designed the fixture again I might also add a third orientation (5 or 7, an unswapped-then-swapped combination) purely to widen the residual's coverage claim from "2 of 8" to "3 of 8" — a cheap addition I judged out of scope for a punch-list round whose contract was two orientations and the dimension case.
