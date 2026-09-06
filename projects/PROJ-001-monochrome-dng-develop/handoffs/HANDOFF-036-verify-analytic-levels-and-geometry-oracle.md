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
  id: HANDOFF-036
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5[1m]       # CONFIRMED, not corrected. System prompt reports
                                    # "Opus 5 (1M context)", exact id `claude-opus-5[1m]`;
                                    # message.model reads `claude-opus-5` on all 184 usage
                                    # objects in this session's own transcript. RIGHT this
                                    # cycle. Scope of that claim: this cycle only — all 17
                                    # prior verify handoffs STORE claude-opus-5, but a stored
                                    # value cannot be told apart from a corrected one, so the
                                    # verify hint's true record is NOT derivable from the
                                    # handoffs and I did not re-derive it
                                    # (signal `tier-map-predicts-what-it-should-record`).
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-05
  status: pending                  # pending | accepted | completed | rejected

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
  tokens_total: 20515070           # REAL combined count — what cost-audit reads
  estimated_usd: 45.60             # tokens_total × your rate, or your harness's number
  duration_minutes: 95
  branch: feat/spec-015-analytic-levels-and-geometry-oracle
  pr: null                         # not opened — Out of Scope
  completed_at: 2026-09-05         # YYYY-MM-DD
  notes: "VERDICT APPROVED at a3f0063 (CI 9/9, run 34003871323); 8 follow-ups FU-4..FU-11, 0 ship-blockers. Cost is a transcript sum deduped by message.id from THIS session's own JSONL (d56874fe-79ae-4cbf-b1b9-c0e078c2dc7b.jsonl, identified by the scratchpad-dir uuid, not by content match): 184 usage objects / 103 unique ids, all message.model=claude-opus-5; raw combined 17,095,892 (input 206 / output 77,549 / cache-read 16,784,762 = 98.2% / cache-write-1h 233,375 / cache-write-5m 0), priced PER-COMPONENT at published Opus rates ($15/$75/$30-1h/$1.50-read) = $38.00, then BOTH figures rounded up 20% per this handoff's point 7 to cover the turns spent writing this handback. ⚠ THIS notes field is deliberately ONE LINE: the build's multi-line scalar is what handback-sync truncated into an unterminated quote in the spec's front matter — see FU-4, which must be fixed BEFORE this entry is synced."
  synced_at: 2026-09-05
---

# HANDOFF-036: Verify SPEC-015 — the analytic oracle, at `7439f49`

## Delegation Summary

Verify `SPEC-015` at **`7439f49`** on
`feat/spec-015-analytic-levels-and-geometry-oracle` (pushed, not merged; `main`
at `23087dc`). It closes `STAGE-002`.

**This is a strong build.** Every number it reports reproduces the design
probe's independently, `src/` is untouched, and the red-proofs run where CI can
see them — which is more than `SPEC-013`'s managed. Verify it on that basis: the
risk is not sloppiness, it is a **well-made oracle with a blind spot the
orchestrator has already measured** (below). Your job is to find the next one.

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| two commits, CI **9/9 on both** | ✅ `2532dc2` run `34000895054`; `7439f49` run `34001284845` |
| **`src/` 0 lines changed vs `main`** (`AC7`) | ✅ `git diff --stat main...HEAD -- src/` is empty |
| 150 tests (was 143), 0 failed | ✅ summed across 9 targets, corpus present |
| "zero skipped" | ✅ the one `SKIP` line is `corpus_absent_file_is_missing_not_an_error`'s deliberate temp-dir probe — pre-existing, not this spec's |
| every measured number reproduces the design probe | ✅ **exactly** — max dev `0.499968 / 0.499968 / 0.499969`, truncation `50.1 % / 49.1 % / 45.0 %`, distinct levels `15872 / 16164` |
| **both red-proofs red with `IRRADIANCE_CORPUS_DIR` unset** (`AC6`) | ✅ ran myself: levels `0.499968 → 264.658371`, 15,841/17,408 px; orientation `[10,0,11,1,12,2] → [0,1,10,11,0,0]`, 6/6 |
| `AC8` — tier-B under 60 s | ✅ **14.68 s** parallel; **35.96 s** at `--test-threads=1`. Both pass; the serial figure is closer to the bound than the handback's "~15 s" implies |
| **`AC3` — no orientation table in the oracle** | ✅ **zero** `5=>`…`8=>` arms in either new file; all 7 mentions of "orientation" in `tests/support/oracle.rs` are doc comments saying it reimplements nothing |
| ten gates, not eleven | ✅ honest — the eleventh has always been fuzz, and the spec's Non-Goals excluded it |
| gates run by me | ✅ all green, `lint-ci` at **clippy 0.1.98** asserted |
| `DEC-020`, `DEC-021`, provenance row, two signals | ✅ present; `decisions-audit` 0 structural errors |

⚠ **Credit where due.** The orientation red-proof carries all three clauses
properly — an `assert_ne!` on the mutant output with an explicit "semantic
no-op, this red-proof has caught NOTHING" message, a green-on-honest control
inside the same test, **and** a separate `the_orientation_fixture_oracle_control_is_green`
for the apparatus. And `FU-2` is a self-caught bug: the build's first
optimisation (pairing distinct values instead of full rank/count) was wrong, its
own honest-tree assertion caught it, and it was filed as a signal rather than
buried. That is the behaviour the handback contract exists to produce.

## ⚠ The blind spot the orchestrator measured — confirm, then decide what it means

**A rank/frequency oracle cannot distinguish one valid permutation from
another.** This follows directly from `DEC-020` and it is not a bug — it is the
price of refusing to reimplement the orientation table. But **nothing says so in
writing**, and a reader will otherwise assume the oracle covers orientation on
real data. It does not.

Measured. Mutation: apply **`Orientation 8`'s mapping where the file says 6** —
a *valid, same-size, same-multiset* permutation (verify the bijection yourself:
for this geometry both 6 and 8 are genuine bijections of the crop window, so no
out-of-bounds zeros appear to give it away):

```
6 => (out_y, flip_y(out_x))            // honest
6 => (flip_x(out_y), out_x),           // MUTANT — Orientation 8's mapping
```

Result on 46,726,912 real pixels — file changed, compiled, output changed:

```
AC1  every_pixel_is_within_half_an_lsb_of_the_exact_affine_map .... PASSED  (blind)
AC3  the_developed_histogram_is_the_normalized_crop_windows ....... PASSED  (blind)
AC4  distinct_output_levels_equal_distinct_input_levels ........... PASSED  (blind)
```

**All three of SPEC-015's tier-B oracle tests pass on a wrong permutation
applied to a real 47-megapixel frame.** What caught it: `SPEC-014`'s
`crop_source_coords_matches_the_worked_example_for_all_eight_orientations`,
`SPEC-014/FU-3`'s `develop_into_applies_orientation_to_pixels_not_only_dimensions`,
and this spec's own red-proof **honest-tree guard** — all three positional, all
three on hand-built fixtures of 6 pixels or fewer.

**So: levels and crop-window contents are checked on 111.5 M real pixels;
*which* permutation was applied is checked on 6.** That may be entirely
acceptable — the positional coverage exists and `DEC-020` bought real
independence for it — but **judge it and say so with the reason.** If it is
acceptable, the orchestrator's view is that it belongs in `DEC-020`'s
`## Consequences` in as many words, because it is exactly the kind of limit a
future reader will assume away. If you disagree, say why.

## Your own checks — where the orchestrator did not go

1. **Does the mutate-and-rebuild actually rebuild?** `DEC-021`'s orientation
   half copies, mutates and rebuilds a probe crate (`SPEC-013`'s `DEC-017`
   mechanism). `SPEC-013`'s verify found three ways to fool that apparatus.
   **Break the rebuild deliberately** — make the injection non-compiling, and
   separately make it a semantic no-op — and confirm the test fails loudly
   rather than silently comparing something against itself. The `assert_ne!`
   guard suggests the no-op case is handled; prove it.
2. **Is `FU-2`'s blind spot actually closed?** The build says it replaced
   distinct-value pairing with full rank/count. Construct a fault that changes
   **only multiplicity** — same set of distinct values, different counts — and
   confirm `bound_check` and `multiset_equal` both catch it. That is the exact
   class `distinct-value-dedup-drops-multiplicity` names, and the fix should be
   provable rather than asserted.
3. **What is `AC1`'s sensitivity floor?** The levels red-proof uses
   `BlackLevel + 64`, which produces a max deviation of **264.66** — four orders
   of magnitude over the 0.5 bound. `DEC-004` measured `BlackLevel + 1` as
   SSIMULACRA2 **100.00**, i.e. completely invisible to the develop oracle.
   **Does this oracle catch `+1`?** Nobody has measured it. If it does, that is
   a strong result worth recording; if it does not, the oracle's floor is worth
   knowing before `STAGE-002` closes on it.
4. **Is `the_orientation_fixture_oracle_control_is_green` load-bearing?**
   Mutate it and see what dies. A control that cannot fail is not a control
   (`DEC-009`).
5. **`AC8` under CI's conditions.** 14.68 s parallel / 35.96 s serial locally.
   CI is a different machine and `cargo test` there may serialise differently.
   Confirm the margin is real rather than local.
6. **`AC2`'s floor.** It asserts `> 40 %` against a measured 45.0–50.1 %. Is
   5 points of margin enough for a file the corpus does not yet hold? Judge
   whether the floor is defensible or merely passing.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-015-analytic-levels-and-geometry-oracle.md`
  — read `## The design decision this spec rests on` first; it is the spec.
- **Build handoff:** `HANDOFF-035` and its `## Handback`.
- **Stage:** `STAGE-002` — this spec closes it.
- **Toolchain brief:** `guidance/toolchain-brief.md` (DEC-004 rule 5).
- **Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.

### Decisions that apply

- `DEC-004` — analytic, never by comparison. Its rule 1 is your job description.
- `DEC-005` — why SSIMULACRA2 cannot do this (and check 3 above).
- `DEC-020` — **new.** Rank/frequency, never positional. Conf. 0.85. This is the
  decision the blind spot above falls out of.
- `DEC-021` — **new.** The two red-proofs use deliberately different mechanisms.
  Conf. 0.85. Check 1 above is aimed at its riskier half.
- `DEC-018` / `DEC-019` — the rounding rule and the crop-origin convention the
  oracle must *not* read as its source of truth.
- `DEC-009`, `DEC-017` — control discipline and the mutate-rebuild mechanism.

### Constraints that apply

- `oracle-must-be-shown-red` — this time the constraint **has** a subject. `AC6`
  is the strongest form this repo has shipped: red **with the corpus absent**.
  Confirm that is genuinely what CI runs.
- `library-not-application` — the oracle lives in `tests/`, not the library.
- `provenance-recorded-per-algorithm` — one new row, separate from
  `src/develop.rs`'s, because the ledger tracks implementations.
- `test-before-implementation`, `no-panics-on-untrusted-input`.

## Out of Scope

- **Fixing anything.** Report; do not repair. A punch list is a verdict.
- **Editing `src/`.** `AC7` binds you too.
- Opening the PR, merging, running `handback-sync`, or closing `STAGE-002`.

## Return Criteria

1. **Ten gates + `just lint-ci`** (there is no fuzz gate — the spec's Non-Goals
   excluded it; confirm that reasoning rather than inheriting it), run by you,
   pasted, summed across all targets, with the clippy version asserted.
   **Observe CI green on the SHA you approve.**
2. **Watch both red-proofs fail yourself, with the corpus absent** (§15 check 9,
   `DEC-004` rule 1), and paste the numbers.
3. **Confirm or kill the permutation blind spot above**, and say what it means.
4. Checks 1–6 under *Your own checks*, each answered with a measurement.
5. Every mutation: file changed **and** compiled **and** *output changed*.
   ⚠ **Stage your work before mutate-and-revert**; md5-verify every revert.
6. **Provenance** (§15 check 11) — one new row, class 1, honest?
7. Handback with a real `tokens_total` **deduped by `message.id`**, priced
   **per-component**, **rounded up ~20 %** — measured here at 9.9 %, 15.4 % and
   19.2 % low across three sessions.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block only.
   ⚠ The project transcript directory also holds the **orchestrator's** live
   session, on a different model, text-matching this delegation because it wrote
   this handoff. It is **not** a prior attempt — identify yours by the uuid in
   **your own scratchpad path** (`SPEC-014/FU-8`).
8. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
9. Findings `SB-N`/`FU-N` with §15 dispositions — numbering **continues this
   spec's sequence, `FU-1`…`FU-3` are taken**, so your first is `FU-4`.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-015-analytic-levels-and-geometry-oracle` at **`a3f0063`**. No PR opened, nothing committed to `src/`, `handback-sync` NOT run, `STAGE-002` NOT closed (Out of Scope).
- **Completed at:** 2026-09-05
- **All acceptance criteria met?** Yes, AC1–AC9, with one wording defect in `AC9` itself (`FU-11`) and two limits that are met-as-written but under-recorded (`FU-6`, `FU-8`).
- **For `verify`:** ✅ **APPROVED at `a3f0063`** — 8 follow-ups (`FU-4`…`FU-11`), **0 ship-blockers**.

**Why `a3f0063` and not `7439f49`.** `src/`, `tests/`, `Cargo.toml`, `Cargo.lock`,
`fuzz/`, `examples/`, `.github/`, `scripts/`, `app.just` and `deny.toml` are
**byte-identical** across `2532dc2`, `7439f49`, `c57f88d` and `a3f0063`
(`git diff --stat <a> a3f0063 -- <those paths>` empty for all three), so every
measurement below applies to all four. I ran the gates on `a3f0063` and observed
CI green on it: **run `34003871323`, headSha `a3f0063`, all 9 jobs success**.
The only thing that is *not* identical across those SHAs is the spec's own front
matter, and that is `FU-4`.

### The gates, run by me

`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, **7/7 corpus files present**.

⚠ **"Ten gates" is not a well-defined phrase in this repo** (`FU-11`): `HANDOFF-013`'s
ten and `SPEC-013`'s verify eleven differ by four members. I ran the **union**, so
the count is not load-bearing:

```
 1. cargo fmt --check                                          exit 0
 2. cargo clippy --all-targets --all-features -- -D warnings   exit 0   clippy 0.1.97 (local)
 3. cargo test --all-features                                  exit 0   150 passed, 0 failed
 4. cargo check --all-targets --all-features  (typecheck)      exit 0
 5. cargo build --release                                      exit 0
 6. ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-f     exit 0   cargo 1.90.0 asserted
 7. cargo deny check licenses            (library graph)       exit 0   "licenses ok"
 8. cargo deny --manifest-path fuzz/... check licenses         exit 0   "licenses ok"
 9. ./scripts/lint-red-proof.sh                                exit 0   control clean (0) -> injection
                                                                        rejected (101) -> all five lints
                                                                        fired at src/lib.rs:59-63, and
                                                                        still fire without -D warnings
10. cargo clippy --lib -F x5   (lint-no-allow)                 exit 0
11. ./scripts/cost-audit.sh                                    exit 0   all shipped specs recorded
12. ./scripts/decisions-index.sh --check                       exit 0   no INDEX.md committed (21 DECs)
--  just lint-ci   PATH-prefixed +stable, FORCE-RELINTED       exit 0   clippy 0.1.98 (88d9e12ae1)
                   (touch'd every .rs first; "Checking irradiance" in the log, not a cache hit)
```

**Test total, summed across all 10 targets** (`cargo test --all-features`, 70.08 s wall):

```
 66 lib · 0 irr · 9 corpus_manifest · 7 develop · 7 develop_oracle
 12 ifd_reader · 30 metadata_oracle · 12 plane_oracle · 7 plane_unpack · 0 doc
 = 150 passed, 0 failed, 0 ignored.  ZERO `SKIP` lines — tier B genuinely executed.
```

Also run: `just validate` (17 artifacts) · `just decisions-audit` (0 structural errors,
6 scope warnings, all pre-existing or benign nesting; the new `DEC-020`/`DEC-021`
same-scope warning is not a contradiction — one is the property set, the other the
injection mechanism, and each says so) · `just decisions-audit --changed main`.

**The eleventh gate — confirmed, not inherited.** `AGENTS.md` §12 bar 2 fires on
"a parser spec that adds a new input surface". Measured:
`git diff --stat main...HEAD -- src/ Cargo.toml fuzz/` is **empty** — this spec adds
no library surface at all — and `fuzz/fuzz_targets/develop.rs` (`SPEC-014`) already
drives the exact chain the oracle consumes (parse → sensor → unpack → develop,
including attacker-controlled `Orientation`). Bar 2 does not fire; the Non-Goal is
correct. I ran the target anyway, to confirm it is invoked rather than merely
committed (`SPEC-003`'s lesson): **`fuzz-develop` 7,213,667 runs in 31 s, zero
crashes, exit 0**, seed corpus byte-unchanged (42 files, md5
`1efa3d4cab36835859d03370cc46d74c`), `git status` clean.

### Both red-proofs, watched by me, with `IRRADIANCE_CORPUS_DIR` **unset**

```
$ unset IRRADIANCE_CORPUS_DIR && cargo test --all-features --test develop_oracle -- --nocapture

SKIP LEICA-Q2-MONO/L1026016.DNG — MISSING at .../tests/corpus/tier-b/...
SKIP LEICA-M-MONOCHROM/L1000622.DNG — MISSING at ...
SKIP LEICA-Q2-MONO/L1021223.DNG — MISSING at ...

RED-PROOF (BlackLevel+64, hand-built, no corpus):
    honest max_deviation=0.499968  faulted max_deviation=264.658371
    — 15841/17408 pixels wrong, the fault turned AC1's bound red
RED-PROOF (orientation identity at call site, hand-built, no corpus):
    honest=[10, 0, 11, 1, 12, 2]  mutant=[0, 1, 10, 11, 0, 0]
    — 6/6 pixels wrong, AC3's histogram property correctly rejects the mutant

test result: ok. 7 passed; 0 failed; finished in 1.01s
```

Both numbers reproduce the orchestrator's exactly. **And `AC6` is genuinely what CI
runs** — verified against the CI log, not the shape: in run `34003871323`,
`tests/develop_oracle.rs` reports `running 7 tests`, and
`the_oracle_is_red_on_a_levels_fault`, `the_oracle_is_red_on_an_orientation_fault`
and `the_orientation_fixture_oracle_control_is_green` are each named `... ok`,
the whole target in 0.70 s with three `SKIP`s. `SPEC-013/FU-1` is genuinely closed
for this oracle.

### The permutation blind spot — CONFIRMED, and worse than measured

Reproduced in an **isolated copy of the crate** (`AC7` binds me, so `src/` in the
real tree was never edited at all; every md5 below is against the untouched repo).
Mutation `M1`: `6 => (flip_x(out_y), out_x)` — `Orientation 8`'s mapping where the
file says 6. File changed (md5 `8c2fc59a…` → `f89eab22…`), **compiled**, and
**output changed** (`poshash c1a7b9ca7a076032` → `df38f4a6e2fe56e2`).

On `L1026016.DNG`, 46,726,912 real pixels, honest vs mutant dumped and compared
sample-for-sample:

```
positionally-wrong pixels = 46,712,160 / 46,726,912   (100.0%)
multisets identical:  True
distinct levels:      15872 honest / 15872 mutant
```

and SPEC-015's three tier-B oracle tests, on that frame:

```
AC1/AC2  every_pixel_is_within_half_an_lsb_of_the_exact_affine_map .... PASSED
         max |shipped - exact| = 0.499968, truncation 50.1% / 49.1% / 45.0%
AC3      the_developed_histogram_is_the_normalized_crop_windows ....... PASSED  ("holds exactly")
AC4      distinct_output_levels_equal_distinct_input_levels ........... PASSED  (15872 / 16164)
```

Every reported number is **byte-identical to the honest tree's**. Full suite under
`M1`, `--no-fail-fast`, corpus present: **147 passed / 3 failed**, and the three are

```
develop::tests::crop_source_coords_matches_the_worked_example_for_all_eight_orientations  (2x3, tier A)
develop_into_applies_orientation_to_pixels_not_only_dimensions                            (6 px, tier A)
the_oracle_is_red_on_an_orientation_fault    — and it failed on its honest-tree
    assert_eq!, `left: [2, 12, 1, 11, 0, 10] right: [10, 0, 11, 1, 12, 2]`,
    i.e. on its POSITIONAL guard, not on its oracle
```

**Then I pushed it one step further, and the answer changed.** Mutation `M6`:
the same 6→8 swap, **gated on `crop_width > 100`**, written inside
`crop_source_coords` so `develop_into`'s call-site text stays byte-identical and
the red-proof's needle still matches. Every hand-built fixture in this repo has
`crop_width <= 3`, far below the gate. File changed, compiled, output changed
(46,712,160 / 46,726,912 px, 100.0%, verified by dump-and-compare against the
honest binary).

```
FULL SUITE under M6, corpus PRESENT, --no-fail-fast:   150 passed, 0 failed.
NOTHING caught it.
```

**So: the mitigation that makes `DEC-020`'s limit acceptable — "the positional
coverage exists" — is real but covers only faults that manifest at ≤ 6 pixels.**
The moment a positional fault is content- or size-dependent, it is invisible to
the entire repo. That is `FU-6`, and it is why I think this belongs in `DEC-020`'s
`## Consequences` **and** in its `## Validation`, not only the former.

**`DEC-020`'s own falsifier has already fired.** Its `## Validation` says *"Wrong if
a future fault shape exists that a rank-preserving merge cannot distinguish from an
honest tree while a true per-pixel positional check could"* — that is exactly `M1`
and `M6`. And the remedy that clause gestures at, *"before assuming Option B
(sort-and-zip) is needed after all"*, cannot help: `DEC-020`'s own Option D
rationale says the shipped merge is *"provably equivalent to Option B … same
pairing, same weights"*, so Option B carries the identical blind spot. Only
Option A (positional) can see it, and Option A is the thing the spec exists to
reject. **The limit is inherent, not closable** — two orientations that differ only
in which corner maps to the origin cannot be told apart by any value-based
invariant, because that correspondence *is* the eight-case table. The right
response is to write that down, not to try to fix it.

### The six checks, each answered with a measurement

**1. Does the mutate-and-rebuild actually rebuild? — YES, all three ways fail loudly.**

| break | result |
|---|---|
| `M2a` injection made **non-compiling** | panics `cargo build --release failed in /var/folders/…/irradiance-develop-oracle-mutant-85302-…` with rustc's own `E0425 … --> src/develop.rs:352`, i.e. the rebuild demonstrably compiled **the injected source**. No stale-artifact path: the temp dir is `pid`+`nanos`-unique per run. |
| `M2b` injection made a **semantic no-op** (re-emit the honest call) | panics `assertion left != right failed: the injected identity-at-call-site fault did NOT change develop_into's output — it is a semantic no-op, and this red-proof has caught NOTHING`, `left: [10, 0, 11, 1, 12, 2] right: [10, 0, 11, 1, 12, 2]`. The third clause is live. |
| `M2c` call site's **arguments reflowed** onto one line (a realistic future refactor; semantics identical) | panics `expected exactly one call to crop_source_coords in src/develop.rs's develop_into; found 0 — the call site moved, update this test`, `left: 0 right: 1`. The needle asserts its match count (`AGENTS.md` §16 rule 2) and dies through its own message (rule 3). |

**2. Is `FU-2`'s blind spot actually closed? — YES, and provably, against the rejected implementation.**
I reconstructed `DEC-020`'s rejected **Option C** (per-distinct-value pairing) and ran
both against multiplicity-only faults — same distinct-value **set**, different counts:

```
case 1  Q2M levels (B=512 W=16383), 1 px moved between two OCCURRING values
        distinct SET identical: true  ({0,4,8} both sides)
        SHIPPED bound_check   0.258459 -> 4.129229   CAUGHT
        AC3 multiset_equal                            CAUGHT
        REJECTED Option C     0.258459 -> 0.258459   MISSED
case 2  W-B == 65535 (AC4's widest legal range; the map is the IDENTITY, so
        adjacent levels differ by exactly 1 — the FINEST shift possible)
        SHIPPED bound_check   0.000000 -> 1.000000   CAUGHT  (2x the 0.5 bound)
        REJECTED Option C     0.000000 -> 0.000000   MISSED
case 3  FU-2's own named shape (extra pixel collapsed onto an occurring 0)
        SHIPPED CAUGHT (0.372188 -> 363.372188); Option C also caught this one,
        because it changes the distinct SET — which is precisely why cases 1
        and 2 are the ones that matter.
```

So `bound_check`'s rank-preserving merge closes the class, its **worst-case
sensitivity floor is 1.0 against a 0.5 bound** (case 2), and the rejected version
provably had the hole. `FU-2` is closed on the code, and the signal is the right
home for the pattern.

**3. `AC1`'s sensitivity floor — measured for the first time, and it is a strong result.**
Same in-process mechanism as the levels red-proof (`DEC-021`), on all three real
frames, oracle told the **true** levels:

| fault | `L1021223` | `L1026016` | `L1000622` | AC1 | AC3 | AC4 |
|---|---|---|---|---|---|---|
| honest (control) | 0.499968 | 0.499968 | 0.499969 | green | green | green |
| `BlackLevel + 64` | 264.658371 | 264.658371 | 259.878797 | **RED** | **RED** | **RED** |
| **`BlackLevel + 1`** | **4.618424** | **4.618424** | **4.546309** | **RED** | **RED** | **RED** |
| `BlackLevel - 1` | 4.612312 | 4.612312 | 4.541607 | **RED** | **RED** | **RED** |
| `WhiteLevel + 1` | 4.000000 | 4.612312 | 4.541607 | **RED** | **RED** | green |
| `WhiteLevel - 1` | 2.382333 | 4.618424 | 4.546309 | **RED** | **RED** | mixed |

**Yes — it catches `BlackLevel ± 1`, at 9.1x the bound, on every decodable frame.**
`DEC-004` measured that same fault as SSIMULACRA2 **100.00** (invisible to the
develop oracle) and `--raw-checksum` is bit-identical to it by contract. And the
reason the `< 0.5` bound is well chosen is now measurable rather than aesthetic:
the map's own quantum is `65535 / (W - B) ≈ 4.13`, so **no levels-tag fault can
land between 0.499968 and ~4.13**. The pre-registered tolerance sits in a real gap.
This belongs in the spec or `DEC-004`; it is the strongest single number this
spec produced and nobody had it.

**4. Is `the_orientation_fixture_oracle_control_is_green` load-bearing? — YES, and both of its assertions are.**
`M3` (stage the control's probe crate **mutated**) → fails,
`left: [0, 1, 10, 11, 0, 0] right: [10, 0, 11, 1, 12, 2]`.
`M4` (keep `M3` **and** neutralise the positional `assert_eq!`) → still fails, on the
second assertion: `the honest tree must satisfy AC3's own property, or the red-proof
above proves nothing about the fault`. Neither assertion is decorative.

**5. `AC8` under CI's conditions — the honest answer is that CI never runs tier B at all.**
Measured in CI run `34003871323`: the whole `develop_oracle` target took **0.70 s**
with three `SKIP`s, because `DEC-003` keeps the corpus out of the repo. `AC8`'s 60 s
bound is therefore a **developer-machine** bound, and the margin question is about
the machine that holds the corpus. On this one:

```
tier-B target, parallel                       15.09 s
tier-B target, --test-threads=1               33.48 s
the three tier-B tests only, serial           36.20 s      <- the number that matters
```

Reproduces the orchestrator's 14.68 / 35.96. **But the margin has a measured
ceiling** (`FU-9`): 36.20 s over 111,529,040 px = **0.3246 s/Mpx**, so a fourth
Q2M-sized decodable file (46.7 Mpx) lands at ≈51.4 s and a fifth at ≈66.6 s.
`DEC-020`'s Validation says the suite *"stays comfortably under 60 s as corpus
files are added"* — measured, that is true for **exactly one** addition, and
`L1026192.DNG` (the file the build's `FU-3` deliberately excluded) is that one.

**6. `AC2`'s floor — defensible, but its margin is not the 5 points it looks like.**
I decomposed the quantity instead of re-measuring it. The in-range disagreement
rate is **0.5006 / 0.5006 / 0.5001** — structurally one half, and independent of
image content, because `frac(v · 65535/(W-B))` is equidistributed. The **only**
thing that moves AC2's total is **clipping**: a pixel at or outside `[B, W]` maps
to exactly `0.0` or `65535.0`, where `round == floor`, contributing zero
disagreement.

```
file          clamped<=B   clamped>=W   clipped share   AC2 total   (design probe said)
L1021223         2,331            7        0.01%         50.05%        50.1%
L1026016        16,020      835,607        1.82%         49.15%        49.1%
L1000622         1,784    1,814,094       10.05%         44.99%        45.0%
```

**Break-even: a CORRECT implementation falls under AC2's 40% floor once the clipped
share exceeds 20.09%.** `L1000622.DNG` is already at 10.05% — half way there. A dark
frame, a lens-cap calibration shot, or a heavily blown exposure crosses 20% easily,
and every one of those is routine in a camera RAW corpus. The floor is defensible
today (2x headroom in the quantity that actually varies) and it fails **loudly and
in the safe direction** — a false red on a new corpus file, never a false green —
but the number worth recording is 20.09%, not "5 points". That is `FU-8`.

### Every mutation: file changed AND compiled AND output changed

All seven mutations ran in an **isolated copy** of the crate under the session
scratchpad, so `src/` in the working tree was never edited (`AC7`). The copy was
md5-verified identical to the repo before and after every one:

```
Cargo.toml               14c3d25bc0393df379b6dfb5c9f3ffae   OK
src/develop.rs           8c2fc59ac430ef834066762d8464203b   OK
src/plane.rs             2b86d470b26ed0bd548380ac0a5943cf   OK
src/ifd.rs               56d43e6f2e05609e45e1d64c75059bb9   OK
src/lib.rs               00b13c4ebf0f96b25ad05d80691c03b5   OK
tests/develop_oracle.rs  9b04c1d50a426d6a1a524b9ec5c01293   OK
tests/support/oracle.rs  a36197140e4c239b157212f9ee6d800d   OK
tests/develop.rs         d8d99018c36c30eb4a39b59a91a0fe06   OK
```

`git status --short` in the real repo shows exactly one entry throughout: the
staged `[~] → [x]` timeline edit this cycle is required to make.

⚠ **One mutation had to be redesigned mid-flight, and saying so matters.** My first
size-gated attempt (`M5`) edited `develop_into`'s call site, which also changed the
red-proof's injection **needle** — so the suite went red on
`found 0 — the call site moved`, which looks like a detection and is not one. `M6`
moves the gate inside `crop_source_coords` so the call-site text stays byte-identical.
**`M5`'s single failure was an artifact of my own mutation; `M6`'s 150/150 is the
real result.** A reviewer's mutation can be confounded exactly the way a build's can.

### One more hole, hunted and found closed — but only on a corpus machine

`M7`: `output_dimensions` returns the crop **transposed** for orientations 5–8, gated
on `crop_width > 100` — same pixel *count*, wrong *shape*. SPEC-015's oracle sizes its
own buffer from `output_dimensions` (the code under test) and then compares only
multisets, so **all three tier-B tests are blind to it**. It is caught — but by
`tests/develop.rs:234`'s **tier-B** `orientation_six_swaps_the_output_dimensions`
(`left: (8368, 5584) right: (5584, 8368)`), while its **tier-A** namesake at
`src/develop.rs:582` uses an 8x6 fixture and passes straight through the gate.
**CI, which never has the corpus, is blind to `M7`.** That is a `SPEC-014` coverage
property surfaced here, not a `SPEC-015` defect — `FU-10`.

### The provenance row (§15 check 11)

Present, one row, `docs/provenance-ledger.md`, **class 1 — specification**, and honest.
It is correctly a *separate* row from `src/develop.rs`'s ("the ledger tracks
implementations, not features"), it names DNG 1.7 Chapter 4 as the source, and its
claim that the eight-case table appears nowhere is one I verified rather than took:

```
match-arm-shaped lines "<digit> =>"   tests/develop_oracle.rs : 0
                                      tests/support/oracle.rs : 0
                                      src/develop.rs          : 8   (the real table, for contrast)
all 7 "orientation" mentions in tests/support/oracle.rs are doc comments (lines 4,6,15,70,107,145,181)
the oracle never reads `sensor.orientation` at all
```

`AC3` met, by construction rather than by discipline.

### Confidence discipline (§16)

`DEC-020` and `DEC-021` are both 0.85 — above the 0.6 verify flag, and honest for
what they are. `DEC-020`'s 0.85 survives `FU-6`: its **decision** is right and its
**Consequences** are incomplete, which is a documentation fix, not a confidence cut.
`decisions-audit` flags `DEC-002` as still `proposed` for `SPEC-012`/`SPEC-014` —
pre-existing, not this spec.

### Findings

**8 follow-ups, `FU-4`…`FU-11`. 0 ship-blockers.** Numbering continues this spec's
sequence (`FU-1`…`FU-3` were raised by the build).

| id | finding | proposed §15 disposition |
|---|---|---|
| `FU-4` | **`SPEC-015`'s front matter has not been valid YAML since `c57f88d`.** `handback-sync` wrote the build's multi-line `notes:` scalar as one truncated line, leaving an unterminated double-quoted scalar at line 98. Psych: `found unexpected end of stream while scanning a quoted scalar at line 98 column 14` — broken at `c57f88d` and `a3f0063`, clean at `2532dc2` and `7439f49`. **Every gate is blind to it**: `just validate` says "17 artifact(s) … have valid required front-matter", CI's `cost-capture audit` is green, and the repo's awk readers still return the right numbers. I simulated the next `handback-sync` with the script's own awk: the verify session's `tokens_total` lands **inside** the open scalar and the file still does not parse. No information is lost (HANDOFF-035 carries the note intact, twice). | `fixed` — one line, **before** the verify handback-sync, not after. Re-terminate the scalar or fold the note to one line. My own handback `notes:` above is deliberately single-line for this reason. |
| `FU-5` | The class. `scripts/_lib.sh:301` `get_handback_field` carries a comment saying this exact breakage was *"Measured on SPEC-001 build round 3, where two of five cost sessions landed broken"* — and the guard it added covers only the `#`-comment path. A multi-line scalar reaches the same end state through `print; exit` on the first matching line. Second mechanism, same defect, same silence, in the function that was hardened against it. | `signal` — new `type: lesson`, or evidence on an existing entry. The code fix is small (consume the folded scalar, or re-quote on write) but the *pattern* is the point. |
| `FU-6` | **The permutation blind spot, confirmed and extended.** Unconditional 6→8: 46,712,160/46,726,912 px (100.0%) positionally wrong, multisets byte-identical, all three tier-B tests green, caught only by three tier-A fixtures of ≤6 px. **Size-gated 6→8 (`M6`): the same 100.0% corruption, 150/150 tests pass, nothing catches it.** `DEC-020`'s `## Validation` already names this falsifier and it has now fired; the remedy it gestures at (Option B, sort-and-zip) is by that record's own Option D rationale *"provably equivalent … same pairing, same weights"* and shares the blind spot exactly. | `fixed` — record it in `DEC-020`'s `## Consequences` (as the orchestrator proposed) **and** correct its `## Validation`, whose stated response cannot work. State the limit as **inherent**: two orientations differing only in which corner maps to the origin cannot be separated by any value-based invariant, because that correspondence *is* the table. |
| `FU-7` | **`AC3` is named "the permutation property" and its red-proof exercises the one fault class that is not a permutation.** The injected identity fault makes `develop_into` read outside the crop window, so the mutant is `[0, 1, 10, 11, 0, 0]` — three zeros, a *different multiset*. AC3 goes red on degeneracy, never on a permutation being the *wrong* permutation, and by `FU-6` no test can show otherwise. The red-proof is sound; its scope is narrower than AC3's name. | `fixed` — one sentence in `tests/develop_oracle.rs`'s AC5(b) section and/or `DEC-021`, saying which half of AC3 the red-proof proves. |
| `FU-8` | **`AC2`'s `> 40%` floor is content-dependent and its real margin is 20.09%, not 5 points.** In-range disagreement rate is structurally 0.5006/0.5006/0.5001; only the clipped share moves the total (clamped pixels land on exact integers where `round == floor`). Clipped 0.01%/1.82%/**10.05%** → 50.05%/49.15%/44.99%. A correct implementation falls under 40% once clipping exceeds **20.09%** — a dark frame or a blown exposure gets there easily. Fails loudly, safe direction (false red, never false green). | `fixed` — put 20.09% next to `AC2` so the first false red is diagnosed in a minute. Or `signal` if the pattern (a content-dependent threshold pre-registered from three frames) is judged recurring. |
| `FU-9` | **`AC8`'s headroom has a measured ceiling of exactly one file.** 36.20 s serial over 111,529,040 px = 0.3246 s/Mpx ⇒ a 4th Q2M-sized decodable file ≈51.4 s, a 5th ≈66.6 s. `DEC-020`'s Validation's *"stays comfortably under 60 s as corpus files are added"* is true for one addition, and `L1026192.DNG` (excluded by the build's `FU-3`) is that one. | `fixed` — replace "comfortably" in `DEC-020`'s Validation with the s/Mpx figure and the file count it buys. |
| `FU-10` | **CI never runs tier B, and one class of geometry fault is caught only there.** `develop_oracle` took 0.70 s with three `SKIP`s in CI run `34003871323`. `M7` (transposed output dimensions, size-gated) is caught **only** by `tests/develop.rs:234`'s tier-B test; the tier-A namesake at `src/develop.rs:582` uses an 8x6 fixture and passes. A `SPEC-014` coverage property, surfaced here. | `spec:` — a small spec that gives the tier-A dimension test a fixture wide enough to be reachable, or `signal` if the class ("tier-A fixtures are all ≤ 8 px, so any size-gated fault is corpus-only") is the more useful record. It is the same root as `FU-6`. |
| `FU-11` | **"Ten gates" is not well defined, and `AC9` is unsatisfiable as written.** `HANDOFF-013`'s ten (fmt, clippy, test, msrv, deny, lint-red-proof, lint-no-allow, cost-audit, decisions-index, deny-fuzz) and `SPEC-013`'s verify eleven (test, fmt, clippy, lint-no-allow, lint-red-proof, typecheck, build, msrv, deny, deny-fuzz, fuzz) differ by four members; both appear in shipped artifacts. `SPEC-015`'s `AC9` says "eleven gates" while its own Non-Goals exclude the eleventh, so the build marked `[x]` having run ten — correctly, and the handoff needed a line to explain it. | `signal` — process friction. One named list in `AGENTS.md` §6 would end it; until then a cycle spends real effort deciding what the count means. |

**Nothing here is ship-blocking.** `src/` is 0 lines changed and correct: no
mutation I ran found a defect in `SPEC-014`'s shipped arithmetic, and the oracle
reproduces the design probe's numbers exactly on all three frames. `FU-4` is the
only one that must be done *before* the next automated write rather than at ship,
and it is one line.

### Drift and new artifacts

- **New decisions emitted:** none. `FU-6`, `FU-7` and `FU-9` are amendments to
  `DEC-020`/`DEC-021`, not new decisions — the decisions are right, their
  Consequences and Validation are incomplete.
- **Deviations from spec:** none. `AC7` respected in the strongest available form:
  every mutation ran in an isolated copy of the crate, so the working tree's `src/`
  was never edited at all, not even transiently.
- **Follow-up work identified:** `FU-10` is the only one that might want its own
  spec, and it is small. `STAGE-002` can close on this.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec; it is the clearest one in this project, and reading
   `## The design decision this spec rests on` first was the right instruction. The
   one genuine cost was `FU-11`: deciding what "ten gates" meant took real time,
   because the repo carries two enumerations differing by four members and both are
   cited in shipped artifacts. I ran the union rather than adjudicate it, which is
   the right move for a reviewer and the wrong state for a repo.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-004`, `DEC-009`, `DEC-017`, `DEC-018`, `DEC-020`, `DEC-021` and
   `oracle-must-be-shown-red` between them cover everything this review needed. The
   one thing I would add to the *verify* prompt rather than the spec: **a reviewer's
   mutation can be confounded by the red-proof's own injection needle.** `M5` cost me
   a run and looked like a detection. The rule that saves it is the one this repo
   already has — assert that the mutation changed the *output*, not just that a test
   went red — applied to the reviewer's own mutations, not only the build's.

3. **If you did this task again, what would you do differently?**
   — Copy the crate to the scratchpad and mutate **there** from the first minute. I
   did, and it made `AC7` trivially true, made seven mutations cheap and reversible,
   and removed any chance of a mutate-and-revert accident in a repo whose memory
   already records one concurrent-writer incident. I would also go **straight** to
   the size-gated form of a mutation. The unconditional 6→8 answers "is the oracle
   position-blind?" (yes) but the size-gated one answers the question that actually
   matters — "does anything else cover it?" — and only the second one turned 147/150
   into 150/150. The gate is what separates a limit that is backstopped from one
   that is not.
