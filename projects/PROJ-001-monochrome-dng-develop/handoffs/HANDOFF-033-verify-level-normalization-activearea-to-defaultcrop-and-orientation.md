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
  id: HANDOFF-033
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # CONFIRMED, not corrected — this session's transcript
                                    # reports message.model = claude-opus-5 on all 76 unique
                                    # assistant turns (system prompt: "Opus 5 (1M context)",
                                    # id claude-opus-5[1m]). The tier_map prediction was
                                    # RIGHT this time; record that against
                                    # `tier-map-predicts-what-it-should-record`, which had
                                    # been 0-for-9 on the BUILD hint.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-014

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
  tokens_total: 13400000           # transcript floor 11,123,510 deduped by message.id, rounded UP 20% — see ## Handback
  estimated_usd: 31.15             # per-component at opus rates, same uplift
  duration_minutes: 25
  branch: feat/spec-014-level-normalization-geometry-orientation
  pr: null                         # not opened, per this handoff's Out of Scope
  completed_at: 2026-09-05
  notes: "VERDICT: APPROVED at 52e6ecf (src/ byte-identical to 1404aac). 0 ship-blockers, 6 follow-ups FU-2..FU-7. Corpus PRESENT 7/7, ZERO SKIP lines. Rounded up 20% (above both measured misses, 9.9% and 15.4%) to cover the handback turns."
  synced_at: 2026-09-05
---

# HANDOFF-033: Verify SPEC-014 — levels, geometry and orientation, at `80913a3`

## Delegation Summary

Verify `SPEC-014` at **`80913a3`** on
`feat/spec-014-level-normalization-geometry-orientation` (pushed, not merged;
`main` at `e575954`). `claude-opus-5` (architect) hands this to the verifier for
the **verify** cycle.

⚠ **The one thing that makes this spec different from every prior one: it has
no oracle, by design.** `SPEC-013`'s `--raw-checksum` attaches to the uncropped,
un-normalised plane by contract, so nothing in this spec is covered by it, and
`DEC-004` settled that no comparison oracle ever will be — `SPIKE-001` measured
the plane checksum **structurally blind** to a levels error, and the develop
oracle blind up to **+256 (50 %)**. `SPEC-015` is the analytic oracle and is
still in `frame`. **Until it lands, this spec's own tests are the only check
that exists.** Review them as the sole line of defence, not as a supplement.

## What the orchestrator reconciled — reproduce, do not inherit

Everything below was **run by the orchestrator on this branch**, not taken from
the build's handback (DEC-004 rule 1). Reproduce it; do not inherit it.

| claim | reconciled |
|---|---|
| branch pushed, HEAD == remote | ✅ `80913a3` local == `refs/heads/…` remote |
| CI observed green on the **shipping SHA** | ✅ **`80913a3` itself** — run `33954821798`, **9/9 jobs**. The handback cites only `1404aac` (run `33954732964`, also 9/9); the head commit is green too |
| 141 tests, 0 failed, corpus present | ✅ summed across all 8 targets: 65 lib + 0 irr + 9 corpus_manifest + 6 develop + 12 ifd_reader + 30 metadata_oracle + 12 plane_oracle + 7 plane_unpack + 0 doc. **Zero SKIP lines** |
| `SPEC-013`'s oracle still passes untouched | ✅ `plane_md5_matches_the_pinned_raw_checksum` green; `tests/plane_oracle.rs` 12/12; `src/plane.rs` **0 lines changed** vs `main` |
| `just lint-ci` (CI's floating stable, not local 0.1.97) | ✅ clean |
| `just validate` / `just cost-audit` | ✅ 17 artifacts valid; cost-audit clean |
| `just decisions-audit` | ✅ **0 structural errors**, 5 scope warnings. `DEC-018`/`DEC-019` sharing `src/develop.rs` is the same shape as the pre-existing `DEC-012`/`DEC-015` pair |
| `DEC-018`, `DEC-019` exist, `status: accepted` | ✅ conf. 0.80 / 0.75, `affected_scope: src/develop.rs` on both |
| provenance row added, class 1 | ✅ `docs/provenance-ledger.md`, `src/develop.rs`, class 1 — specification |
| fuzz target registered + 10 seeds committed | ✅ `fuzz/Cargo.toml` `[[bin]] develop`; `just fuzz-seeds` **regenerates all 10 byte-identically** (tree stayed clean), so the committed seeds match their generator |
| all seven of the spec's **Failing Tests** name a real test | ✅ match count asserted against the 141 live names — **1 match each** (`orientation_six_swaps_the_output_dimensions` has 2: a lib and an integration copy). **Zero vacuous names** (`named-tests-can-pass-vacuously`) |
| `AGENTS.md` + `app.just` edits, unmentioned in the handback | ✅ legitimate — `just fuzz-develop` added, and §6's command block must gain a line when a recipe does (that correspondence is `AC8`) |

### ⚠⚠ The one that mattered — **AC4's fixture is load-bearing, measured**

The spec's central warning is that **an implementation ignoring the `ActiveArea`
origin entirely passes every corpus test**. That is now measured, not predicted.

**First, the premise, re-measured independently** via `irr ifd` on all seven
corpus files rather than trusting the design probe:

| file | compression | `ActiveArea` | decodable |
|---|---|---|---|
| `L1021223.DNG` / `L1026016.DNG` / `L1026192.DNG` | 1 | `top 0, left 0, bottom 5632, right 8392` — **origin (0,0)** | yes |
| `L1000622.DNG` | 1 | **absent** | yes |
| `M2462362.DNG` | 7 | absent | no |
| `K3III.DNG` | 7 | `top 34, left 26` — **the only non-zero origin** | **no** |
| `K3III.PEF` | 65535 | absent | no |

✅ Premise confirmed: **no decodable file can observe the distinction.**

**Then the mutation** — `develop_into` changed to drop `geometry.active_left` /
`active_top` from the source index, i.e. exactly "ignore the `ActiveArea`
origin". All three clauses asserted:

- **file changed** — `git diff --stat` 4 insertions, 4 deletions ✅
- **compiled** — `cargo build --all-features` finished ✅
- **output changed** — ✅ and here is the result that matters:

```
cargo test --all-features --no-fail-fast, under the mutation:
  src/lib.rs             64 passed, 1 FAILED  ← crop_origin_is_relative_to_active_area_not_the_raw_plane
  tests/develop.rs        6 passed, 0 failed  ← every tier-B corpus test STILL GREEN
  tests/plane_oracle.rs  12 passed, 0 failed
  … every other target    0 failed
  ────────────────────────────────────────────
  140 of 141 pass. The ONLY failure is AC4's hand-built fixture.
```

And it does not merely fail — it fails **to the exact wrong value** the wrong
reading produces: `left: 44, right: 172`, where `44 == normalize(11)` (the
raw-plane reading, sample `(1,1)`) and `172 == normalize(43)` (the ActiveArea
reading, sample `(4,3)`). The fixture also carries its own `assert_ne!` against
the wrong reading, so it cannot pass vacuously.

**Verdict on the hardest thing in this spec: `AC4` is met, and its fixture is
provably the single test in the repository that observes the distinction.** The
tree was restored byte-identically afterwards (`git diff --exit-code src/develop.rs`
clean).

You should still reproduce this yourself — that is check 9's whole point — but
you are reproducing a known result, not hunting for one.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-001-monochrome-dng-develop/brief.md`
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-002-the-monochrome-plane-unpack-bit-exact-oracle-geometry.md`
- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-014-level-normalization-activearea-to-defaultcrop-and-orientation.md`
  — read its `## Implementation Context` in full; the blind-spot section is the spec.
- **Build handoff:** `./projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-032-build-level-normalization-activearea-to-defaultcrop-and-orientation.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` (DEC-004 rule 5) — leads with the `cargo +nightly` trap.
- **Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` — the default root does not exist, and a tier-B test passes whether or not the corpus is there. Only `just test` names what is missing.

### Decisions that apply

- `DEC-004` — levels/crop/orientation are verified **analytically, never by comparison**. Its rule 1 is also your job description: a red-proof you did not personally observe failing is a self-report.
- `DEC-016` — caller-owned buffers, no allocation on the algorithmic path. `develop_into` follows it.
- `DEC-018` — **new, this build.** The developed image is `u16`, full-scale, out-of-range levels **clamped**. Confidence 0.80.
- `DEC-019` — **new, this build.** `DefaultCropOrigin` is relative to `ActiveArea`. Confidence 0.75 — the lowest new decision here and the one AC4 exists to defend. §16 says < 0.6 is a yellow flag; 0.75 is not, but ask whether the evidence justifies more or less.
- `DEC-002` — still `proposed` (0.72). `decisions-audit` flags SPEC-014 as built against it, same as SPEC-012. Advisory, pre-existing; no `rayon`, no assumed `std`, no runtime SIMD dispatch.
- `DEC-013` — the cost handback contract.

### Constraints that apply

Full text in `./guidance/constraints.yaml`.

- `no-panics-on-untrusted-input` — geometry is attacker-controlled. `AC6` is the criterion; `#[forbid(unsafe_code)]` and the no-panic lint set are the gates.
- `oracle-must-be-shown-red` — ⚠ read this one carefully against **a spec that has no oracle**. Judge whether the constraint is satisfied, vacuous, or evaded here, and say which.
- `provenance-recorded-per-algorithm` — one new row, class 1.
- `library-not-application` — `irr develop` is a dev affordance, not a product surface.
- `cost-captured-per-cycle` — append your verify session via the handback, not by hand.

### Prior related work

- `HANDOFF-030`/`031` — `SPEC-013`, ✅ APPROVED at `88cc343`, 4 follow-ups. Its `FU-1` (the red-proof passes vacuously where CI runs it) is the nearest neighbour to this spec's oracle question.
- `SPIKE-001` / `SPIKE-002` — the "parameter was always 14" shape, and what a second camera body cost to discover it.

## Your own checks — where the orchestrator did not go

1. **Fuzz (§15 check 10).** The build claims `just fuzz-develop 60` →
   **14,562,321 executions, zero crashes**. The orchestrator did **not** run
   nightly. Run it, report the count and the duration, and say whether the seed
   corpus changed. Ten seeds are committed, including
   `nonzero-active-area-origin.tiff` — check the seeds actually reach the
   rejection branches `AC6` names, rather than being ten shapes that all bounce
   off the same early return.
2. **Is `crop_source_coords`' eight-orientation table right, or just
   self-consistent?** It was **hand-derived**, and the only real files carry
   `Orientation` 1 and 6. `crop_source_coords_matches_the_worked_example_for_all_eight_orientations`
   verifies it against a worked 2×3 example written by the same session that
   wrote the table. That is one author checking their own arithmetic twice.
   Six of the eight values have **no independent corroboration anywhere in this
   repo**. Find some, or say plainly that they are unverified — this is the
   `measurement-over-generalised` shape: "verified for all eight" is a claim
   about eight, backed by one derivation.
3. **`AC7`'s memory number.** 275,890,176 bytes peak RSS, claimed as
   `SPEC-012`'s 182,435,840 + a 93,453,824-byte buffer. That sums to
   275,889,664 — **512 bytes short**. Reproduce the measurement; either the
   accounting is approximate and should say so, or it is exact and the residual
   means something.
4. **`AC2`'s clamp, on real data.** Both real files contain samples below
   `BlackLevel` (min 2 and 108) and both reach `WhiteLevel` exactly. Confirm the
   clamp is exercised **by corpus data**, not only by the unit fixtures — and
   that `max == WhiteLevel` maps to `u16::MAX` and not one below it.
5. **`normalize`'s rounding.** `normalize_maps_the_endpoints_and_an_interior_point`
   asserts its interior point with `assert!((32000..=33500).contains(&got))` — a
   1,500-wide band, which is not an assertion about rounding, it is an assertion
   that the function is roughly linear. Decide whether the exact value is
   knowable and should be pinned. ⚠ `SPEC-015` will assert `BlackLevel → 0` and
   `WhiteLevel → 1` against this; a loose interior is where the two will disagree.
6. **`tests/corpus/manifest.toml`'s corrected note — the orchestrator believes
   the fix introduced a new error.** `FU-1` fixed a note that mislabelled
   `L1000622.DNG`'s `DefaultCropOrigin` as `ActiveArea`. The replacement text
   (`tests/corpus/manifest.toml:190-194`) now reads *"where every Q2M frame's
   crop origin is 12 24 against a **non-zero ActiveArea**"*. Measured above:
   every Q2M frame's `ActiveArea` is `top 0, left 0` — a **zero** origin. Read
   in the vocabulary `SPEC-014` uses throughout, that sentence asserts the
   opposite of the spec's load-bearing claim, in the very note that was being
   corrected to stop a reader believing a decodable non-zero-origin file exists.
   Confirm or kill it, and disposition it.
7. **Does `oracle-must-be-shown-red` bite here at all?** `SPEC-013`'s red-proof
   still runs and still passes (12/12). But this spec adds a whole surface with
   no red-proof of its own. `AC4`'s fixture behaves *like* one — the orchestrator
   watched it go red under a real mutation — but nothing in the repo runs that
   mutation, so it is a red-proof only when someone performs it by hand. Is that
   the same shape as `SPEC-013/FU-1`, and if so, is it acceptable here for the
   same reason, or is `SPEC-015` the answer? Say which, with the reason.

## Out of Scope

If any of these needs doing, it is a spec or a signal, not an expansion here.

- **The analytic oracle** — that is `SPEC-015`, already framed, and it closes
  `STAGE-002`. Do not build it.
- Demosaic, colour, tone — `STAGE-003` / PROJ-002.
- Changing `SPEC-012`'s output. `SPEC-013`'s oracle attaches to the uncropped,
  un-normalised plane and must keep passing untouched.
- Opening the PR, merging, or running `handback-sync`.
- Fixing anything you find. **Report; do not repair.** A punch list is a verdict,
  not a commit.

## Return Criteria — how to hand back

1. **Eleven gates + `just lint-ci`**, run by you, pasted, **summed across all
   targets**. `just lint-ci`, not `just lint` — local clippy is 0.1.97 and CI
   floats at 0.1.98. **Observe CI green on the SHA you approve** (already 9/9 on
   `80913a3`; if you approve a different SHA, that SHA needs its own observed run).
2. **Watch `AC4`'s fixture fail yourself** (§15 check 9, DEC-004 rule 1). Paste
   the mutation, the assertion values, and the count of what else broke.
3. **Fuzz** (§15 check 10) — count and duration, not "a target is committed".
4. **Provenance** (§15 check 11) — one new row, class 1, DNG 1.7 + TIFF 6.0.
   Confirm the class is honest and the source is a published spec.
5. Every mutation: **file changed AND compiled AND output changed.** That third
   clause has caught four false red-proofs in three specs. ⚠ **Stage your work
   before mutate-and-revert** — `SPEC-010`'s build lost its entire change to
   `git checkout --` and shipped a reconstruction.
6. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports. ⚠ **Do not hand-write `cost.sessions`** — fill the
   handback block only, so `handback-sync` runs once cleanly. Hand-writing it has
   caused four duplicate-entry cleanups.
   ⚠ **Round your number up to cover the turns that write the handback itself.**
   Measured on two build sessions in this repo: the self-reported figure came in
   **9.9 %** low (`SPEC-014`, reported 44,845,024, transcript settles at
   49,294,128) and **15.4 %** low (`SPEC-013`, reported 39,061,192, settles at
   45,095,654) — an agent counting its own transcript cannot count its own tail.
   `HANDOFF-031` already rounded up for this reason; do the same and say so.
7. **Correct `handoff.to_agent`** to what your system prompt actually reports.
   Do not run `handback-sync`; do not open the PR.
8. Findings labelled `SB-N` / `FU-N` with §15 dispositions. **Numbering continues
   `SPEC-014`'s own sequence — `FU-1` is taken** (the manifest note, disposition
   `fixed`), so your first new finding is `FU-2`. A `spec:` disposition must
   **name an AC that would fail** without it.
9. Answer §15's reflection questions in the handback.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / SHA reviewed:** `feat/spec-014-level-normalization-geometry-orientation` at
  **`52e6ecf`** (branch head, local == remote). `git diff 1404aac..HEAD -- src/ tests/ fuzz/
  examples/ Cargo.toml Cargo.lock` is **empty**, so the code I measured is byte-identical to
  the build commit; everything between them is docs. **`src/plane.rs`: 0 lines changed vs
  `main`** — `SPEC-013`'s oracle is untouched, and `tests/plane_oracle.rs` is 12/12 green.
- **Corpus:** **PRESENT, 7/7**, via `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  `just test` printed `corpus: 7/7 present — no tier-B test will skip` and the run produced
  **ZERO SKIP lines**. No test skipped.
- **Completed at:** 2026-09-05
- **All acceptance criteria met?** **Yes, all 8.** AC5 is met *as written* (its text and its two
  named tests are about output *dimensions*), but "orientation is applied" to *pixels* is
  asserted by nothing in the repo — see `FU-3`, which I measured rather than inferred.
- **Verdict:** ✅ **APPROVED at `52e6ecf`** — 0 ship-blockers, 6 follow-ups (`FU-2`…`FU-7`).

### 1. Eleven gates + `just lint-ci`, run by me, summed across all targets

| # | gate | result |
|---|---|---|
| 1 | `cargo fmt --check` | exit 0 |
| 2 | `just lint` (clippy 0.1.97, local) | exit 0 |
| 3 | `cargo test --all-features` | **141 passed, 0 failed**, summed across 9 targets |
| 4 | `just lint-no-allow` | exit 0 |
| 5 | `just deny` | `licenses ok` |
| 6 | `just deny-fuzz` | `licenses ok` |
| 7 | `just msrv` (`+1.90.0`) | exit 0 |
| 8 | `just lint-red-proof` | `✓ lint policy red-proof` — control clean → injection rejected → all five lints fired |
| 9 | `just fuzz-develop 60` | **12,167,207 runs / 61 s, zero crashes** |
| 10 | **`just lint-ci`** | exit 0, and I asserted the version: **clippy 0.1.98 (88d9e12ae1)**, not the local 0.1.97 |
| 11 | `just decisions-audit` | **0 structural errors**, 5 scope warnings |
| + | `just validate` / `just cost-audit` | 17 artifacts valid / clean |
| + | `just decisions-audit --changed` | no active decision's scope matches my (docs-only) changes |

Test total, per target: 65 lib + 0 irr + 9 `corpus_manifest` + 6 `develop` + 12 `ifd_reader`
+ 30 `metadata_oracle` + 12 `plane_oracle` + 7 `plane_unpack` + 0 doc = **141**.

**CI OBSERVED GREEN on the SHA I am approving.** `gh run view` on **`52e6ecf`** → run
**`33980344540`**, `conclusion: success`, **all 9 jobs** (fmt, clippy -D warnings, test,
licenses, licenses-fuzz, MSRV, cost-capture audit, lint-policy-red-proof,
lint-policy-no-allow). Also 9/9 on `80913a3` (`33954821798`) and on `1404aac`
(`33954732964`); no job is path-filtered off a docs commit, so all three ran the full set.

**The seven named failing tests, match-counted against the 141 live names** (rule 2 —
assert the count, never take the first hit): 1 match each, except
`orientation_six_swaps_the_output_dimensions` which has **2** (a lib copy and an integration
copy). `crop_origin_is_relative_to_active_area` matches exactly one real test,
`develop::tests::crop_origin_is_relative_to_active_area_not_the_raw_plane`. **Zero vacuous
names.** Reproduced, not inherited.

### 2. `AC4`'s red-proof — watched failing, by me

**Mutation 1**, `develop_into`: drop `geometry.active_left` / `active_top` from the source
index — i.e. exactly "ignore the `ActiveArea` origin".

- **file changed** — `git diff --stat`: 5 insertions, 6 deletions ✅
- **compiled** — `cargo build --all-features` finished ✅
- **output changed** ✅:

```
cargo test --all-features --no-fail-fast, under the mutation:
  src/lib.rs             64 passed, 1 FAILED
    develop::tests::crop_origin_is_relative_to_active_area_not_the_raw_plane
    assertion `left == right` failed:
      left: 44      <- normalize(11), the raw-plane (wrong) reading, sample (1,1)
     right: 172     <- normalize(43), the ActiveArea (correct) reading, sample (4,3)
  tests/develop.rs        6 passed, 0 failed   <- every tier-B corpus test STILL GREEN
  tests/plane_oracle.rs  12 passed, 0 failed
  ────────────────────────────────────────────
  140 of 141 pass. The ONLY failure is AC4's hand-built fixture.
```

**Negative control:** the unmutated tree is 141/141 (run before and again after restore).
Tree restored **byte-identically** — `src/develop.rs` md5 `3887b741249d9a894a0b726ea924f67c`
before and after, `git diff --exit-code -- src/` clean. Work was `git add -A`-staged and
additionally copied to the scratchpad before either mutation (`SPEC-010`'s lesson).

**I also re-measured the premise itself** on all seven corpus files with `irr ifd`, rather
than trusting the design probe or the reconciliation table:

| file | compression | `ActiveArea` | decodable |
|---|---|---|---|
| `L1021223` / `L1026016` / `L1026192` | 1 | `top 0, left 0, bottom 5632, right 8392` — **origin (0,0)** | yes |
| `L1000622.DNG` | 1 | **absent** | yes |
| `M2462362.DNG` | 7 | absent | no |
| `K3III.DNG` | 7 | `top 34, left 26` — **the only non-zero origin** | no |
| `K3III.PEF` | 65535 | absent | no |

✅ **Premise confirmed independently: no decodable file can observe the distinction.**
`AC4`'s fixture is provably the only thing in the repository that can.

### 3. Fuzz (§15 check 10) — and the seeds do NOT all do their job

`just fuzz-develop 60` → **12,167,207 executions in 61 seconds, zero crashes, exit 0**
(`cov: 808 ft: 1923 corp: 309`). Seed corpus **byte-unchanged**: combined md5 of the 10
seeds is `4a48af2e05eeb3c15fe4d938739230a8` before and after, and `just fuzz-seeds`
**regenerates all 10 byte-identically** (tree stayed clean). Zero artifacts under
`fuzz/artifacts/`. `fuzz/corpus/develop` is gitignored, so `git status` stayed clean.

The handoff asked me to check the seeds **reach** `AC6`'s rejection branches rather than
bouncing off a shared early return. I did not eyeball it — I built a throwaway probe crate
**outside the repo** (scratchpad, `irradiance` by path, repo tree never touched) that
replicates `fuzz_targets/develop.rs`'s control flow exactly and reports the branch each
seed exits through:

| seed | outcome |
|---|---|
| `absent-geometry-tags` | REACHED — develops 8×6 → 8×6 (every default applied) |
| `crop-exceeds-active-area` | REACHED — `InvalidDefaultCrop` |
| `crop-origin-out-of-plane` | REACHED — `InvalidDefaultCrop` |
| `zero-size-crop` | REACHED — `InvalidDefaultCrop` |
| `inverted-active-area` | REACHED — `InvalidActiveArea` |
| `orientation-out-of-range` | REACHED — `UnsupportedOrientation { orientation: 9 }` |
| `nonzero-active-area-origin` | REACHED — develops 8×6 → 3×2 |
| `rotated-six` | REACHED — develops 8×6 → 4×6, `Orientation 6` |
| `unrotated-full-plane` | REACHED — develops 8×6 → 6×4 |
| **`black-level-at-white-level`** | ❌ **NEVER REACHES `develop_into`** — see `FU-2` |

**9 of 10 reach the geometry surface; every branch `AC6` names is reached.** The tenth is a
dud and is `FU-2`.

### 4. Provenance (§15 check 11)

One new row in `docs/provenance-ledger.md` for `src/develop.rs`, **class 1 — specification**,
sourced to DNG 1.7 §Chapter 4 and TIFF 6.0 tag 274. **The class is honest** and the sources
are published specifications, not implementations. The row states outright that nothing was
read from `dnglab`/`rawler` (LGPL-2.1) and that `Orientation`'s semantics were cross-checked
against `exiftool`'s labels rather than transcribed from an unverifiable page number — which
is the honest form. `cargo deny` passes on **both** graphs; **no new dependency** was added
(`[dependencies]` is still empty; `fuzz/Cargo.toml` gained a `[[bin]]` stanza only), so §15
check 12 has no subject.

### 5. Handoff check 2 — the eight-orientation table, corroborated independently

The handoff's sharpest question: `crop_source_coords`' table was hand-derived and verified
against a worked example **written by the same session**, and six of the eight values had no
independent corroboration anywhere in this repo. I found some.

I generated a 2×3 grey image whose every pixel value names its own stored `(x,y)` (`10*y+x`),
wrote it eight times as a TIFF carrying `Orientation` 1–8 (**`exiftool` confirmed each file's
actual tag value**, so the input is measured, not assumed), and had **ImageMagick 7.1.2-29**
`-auto-orient` each one. Then I ran the **shipping `develop_into`** over the same 2×3 plane
for all eight values and machine-diffed the two.

```
═══ ImageMagick vs the shipping develop_into ═══
IDENTICAL on all 8 orientations.
cells compared: 48 (expect 48)
lines compared: 8 (expect 8)
```

**All eight values, all 48 cells, byte-identical to an implementation nobody in this project
wrote.** The six previously-uncorroborated values now have a second, independent point.
That is a genuine second measurement in a different direction, not a restatement —
`measurement-over-generalised` is satisfied for this claim, and I am stating the scope
exactly: **one 2×3 fixture, eight orientation values, one external tool.**

### 6. Handoff check 3 — `AC7`'s memory number

Reproduced with `/usr/bin/time -l ./target/release/irr develop L1021223.DNG`, **three runs,
identical every time: 275,906,560 bytes** peak RSS. The build reports **275,890,176** — my
figure is **16,384 bytes higher, exactly one macOS aarch64 page.**

So the answer to "approximate or exact?" is: **approximate, and `AC7` should say so.** The
buffers themselves are exact and `irr` prints them (`raw_plane_bytes 94,887,936`,
`developed_bytes 93,453,824`, file 85,796,864 = 274,138,624 of real buffer, all three alive
at peak — confirming **not in-place**). Everything above that is binary, stack and allocator
overhead measured at page granularity. The 512-byte residual is not a meaningful quantity;
it is noise inside a 16,384-byte page quantum, and my own measurement moved it by a full
page. `DEC-018`'s own wording ("to within rounding") is honest; `AC7`'s bare `=` is not.
That is `FU-5`.

### 7. Handoff check 4 — `AC2`'s clamp, on real corpus data

Measured through the real decode path, on the pixels that **survive the crop** — not merely
present somewhere in the raw plane, which is the distinction that mattered:

| file | raw min / max | below BlackLevel | at WhiteLevel | developed `== 0` | developed `== u16::MAX` |
|---|---|---|---|---|---|
| `L1021223.DNG` (B 512 / W 16383) | 2 / 16383 | 3,460 | 152,091 | **2,331** | **7** |
| `L1000622.DNG` (B 220 / W 16383) | 108 / 16383 | 1,645 | 1,818,840 | **1,784** | **1,814,094** |

✅ **The clamp fires on real corpus data in both directions, after the crop.** And ✅
**`max == WhiteLevel` maps to `u16::MAX`, not one below** — 7 and 1,814,094 developed pixels
land exactly on `65535` on the two files. (Note the Q2M's at-white count collapses 152,091 →
7 across the crop: almost all of them live in the 32-column strip outside `ActiveArea`. The
clamp still fires on the 7 that remain.) `normalize(16383, 512, 16383) == 65535` exactly, by
integer arithmetic, with no rounding slack.

### 8. Handoff check 5 — `normalize`'s rounding. Yes, it is knowable, and it should be pinned.

The exact value is fully determined — integer arithmetic throughout, no `powf`, no
platform variance. Measured: `normalize(8447, 512, 16383) == 32765` and
`normalize(8301, 220, 16383) == 32765`. The test asserts a **1,500-wide band** around a
value known to the LSB.

But the band is the smaller half of the problem. **The interior point chosen is the midpoint,
which is precisely where round-to-nearest and truncation agree** — so even tightening the
assertion at that point would not pin the rule. Measured over the whole domain: for Q2M's
levels the two rules give different answers on **7,935 of 15,872 in-range samples — exactly
half**. Sample 516 is the first: truncate → 16, round-to-nearest → 17, and `develop_into`
gives **17**, so the shipping code does implement the round-to-nearest its doc comment
claims — but **nothing in the repository asserts it.**

This is the disagreement point the handoff predicted for `SPEC-015`, and it is also a §15
check-5 gap: the rounding rule is a non-trivial implementer choice recorded **only in a
function doc comment**. `DEC-018` owns the output representation and does not mention it.
That is `FU-4`.

### 9. Handoff check 6 — the manifest note. **Confirmed: the fix introduced a new error.**

`tests/corpus/manifest.toml:190-194` now reads *"where every Q2M frame's crop origin is 12 24
against a **non-zero ActiveArea**"*. I measured all three Q2M frames myself: `ActiveArea` is
`top 0, left 0, bottom 5632, right 8392` — **origin (0,0)**.

There is a charitable reading ("non-zero" = the tag is present and non-degenerate, unlike
this file where it is absent), and it is probably what was meant. But this is a document that
uses "non-zero `ActiveArea` origin" as its load-bearing phrase throughout `SPEC-014`, in the
one note being corrected **specifically to stop a reader believing a decodable non-zero-origin
file exists**. Read in that vocabulary, the replacement sentence asserts the opposite of the
spec's central claim. **Confirmed, not killed.** `FU-6`.

### 10. Handoff check 7 — does `oracle-must-be-shown-red` bite here?

**Short answer: not as written — it has no subject. And that is a recorded position, not an
evasion. But its *principle* is met for `AC4` and NOT met for `AC5`, and I measured that.**

**As written, the constraint is inapplicable.** Its subjects are "every oracle AND every
gate". `SPEC-014` ships **neither**: no oracle, because `DEC-004` (confidence 0.92, accepted)
already settled that no comparison oracle can cover this surface — `SPIKE-001` measured the
plane checksum structurally blind to a levels error and the develop oracle blind to +256
(50%); and no gate, because the eleven gates are unchanged (`just fuzz-develop` is a fuzz
runner, not a pass/fail claim). So the constraint is **inapplicable, not vacuous and not
evaded** — the difference being that a *decision record* says why, in advance, rather than
this spec asserting it. `SPEC-013`'s red-proof still runs and still passes, untouched, 12/12.

**As a principle — "no verification claim without a demonstrated red" — the picture splits:**

- **`AC4`: met.** I watched it go red under a real mutation, to the exact wrong value, with a
  clean negative control. And it is a **lib unit test**, so unlike `SPEC-013/FU-1`'s red-proof
  it **does run in CI without the corpus**. Its weakness is the one the handoff names: nothing
  in the repo *performs* the mutation, so it is a red-proof only when a human does one.
- **`AC5`: not met, and I measured it.** **Mutation 2**: make `develop_into` resolve the
  orientation for the *dimensions* but apply the identity transform to the *pixels*.
  file changed ✅, compiled ✅, **output changed** ✅ (the developed image goes from the true
  90° CW rotation to a raw row-major read that even leaves a whole row of zeros) — and
  **141 of 141 tests still pass.** A verification claim with no demonstrated red, which is
  exactly what the constraint exists to prevent. That is `FU-3`.

**Is this `SPEC-013/FU-1`'s shape?** For `AC4`, **no** — `FU-1` was a red-proof that existed
but skipped where CI runs it; `AC4`'s assertion runs everywhere. For `AC5` it is **worse than
`FU-1`**: there is no red-proof to skip.

**Is `SPEC-015` the answer?** For `AC4`'s levels half, plausibly. **For `AC5`, no — and I
checked rather than assumed.** `SPEC-015` is still a **bare, unfilled template**: `cycle:
frame`, `## Context` and `## Goal` are boilerplate placeholder text, and it has **zero
acceptance criteria**. §15 requires a `spec:` disposition to name an AC that would fail
without it, and there is no AC in `SPEC-015` to name. Routing `FU-3` there would be
"carried into the next brief" wearing a disposition's clothes. The honest close is a test,
and it is cheap — see `FU-3`.

**One more thing worth recording in the constraint's favour, which the handoff did not
credit:** `DEC-019`'s convention is **not** unprotected. `tests/metadata_oracle.rs::dnglab_crop_origin_is_active_area_plus_default_crop_origin`
is a **live, running comparison oracle** over exactly this convention, on all six DNG corpus
files **including `K3III.DNG`** — the one file with a genuinely non-zero `ActiveArea` origin.
It works from metadata, so the file's undecodability does not block it, and it passed in my
run (30/30). It proves the *convention*; `AC4`'s fixture proves the *wiring*. Together that
is a stronger position than "this spec has no check at all", and it deserves to be said.

### 11. Findings

Numbering continues `SPEC-014`'s sequence; `FU-1` (the manifest note, `fixed`) is the build's.
**0 ship-blockers.** Every finding below is a follow-up: none lets bad data or a panic reach
a consumer — the code is correct where I could measure it, and the gaps are in what the tests
would catch on a **future** change, plus two documentation inaccuracies.

| id | finding | proposed disposition |
|---|---|---|
| `FU-2` | **The `black-level-at-white-level.tiff` fuzz seed never reaches the branch it exists for.** Its generator comment says `AC6: BlackLevel >= WhiteLevel`, but `plane::unpack_into` rejects it first (`sample 0 is 256, which exceeds WhiteLevel 100`), so `develop_into` — and therefore `Error::InvalidLevels` — is never reached from the fuzz corpus. Measured per-seed, 9/10 reach the geometry surface, this one does not. The branch **is** reachable in principle (needs every sample ≤ `WhiteLevel` **and** `black >= white`); the fixture just picked sample bytes that die earlier. | `fixed` — one-line change to `examples/fuzz-seeds.rs` (sample values ≤ 100) plus `just fuzz-seeds`. Note `AC6`'s own five named branches are all reached, so this does not unmake `AC6` |
| `FU-3` | **Nothing in the repository observes `develop_into` applying orientation to pixels.** `crop_source_coords` is unit-tested in isolation; `output_dimensions` is tested for the swap; but the *composition* — what a consumer actually receives — is exercised at **`Orientation 1` only**. Measured: a mutation that keeps the swapped dimensions and applies the identity transform to pixels changes the output and leaves **141/141 green**. This is `SPEC-014`'s own stated hazard shape ("an implementation that ignores X passes every test") on a second axis. Compounded by design: the deliberate `unwrap_or` / `get().unwrap_or(0)` fallbacks mean an off-by-one produces **silent black pixels**, never an error — the right call for `no-panics-on-untrusted-input`, but it removes the loud failure a missing test would otherwise rely on. ⚠ **The code is CORRECT** — 48/48 cells match ImageMagick on all eight values. This is a coverage hole, not a defect | `fixed` — one tier-A test asserting `develop_into`'s **pixel** output for at least `Orientation 6` (the value a real corpus frame carries). The 2×3 fixture in §5 above is the whole test. **Not** `spec: SPEC-015`: that spec has no AC to name |
| `FU-4` | **`normalize`'s rounding rule is neither pinned nor recorded.** The assertion is a 1,500-wide band (`32000..=33500`) around a value that is exactly `32765`; and the interior point chosen is the midpoint, the one place round-to-nearest and truncation **agree**, so the band cannot see the rule at all. The two rules differ on **7,935 of 15,872** in-range samples. The doc comment claims round-to-nearest, the code implements it (sample 516 → 17, not 16), and nothing asserts it. `DEC-018` owns the output representation and does not mention rounding — a §15 check-5 gap. ⚠ This is the exact point where `SPEC-015`'s `BlackLevel → 0` / `WhiteLevel → 1` will disagree with this module | `fixed` — pin the exact interior value, add one assertion at a sample where the rules differ, and add a line to `DEC-018`. All three are small |
| `FU-5` | **`AC7`'s memory accounting is presented as exact and is not.** `275,890,176 = 182,435,840 + 93,453,824` is off by 512, and my own three-run measurement is **275,906,560** — one full 16 KiB macOS page higher. Peak RSS is page-granular and includes binary/stack/allocator overhead that is in neither buffer, so the residual means nothing. The buffers themselves **are** exact and `irr develop` prints them | `fixed` — reword `AC7` (and the build's handback) from `=` to `≈, to within a page`; `DEC-018`'s "to within rounding" is already right. Optionally state the three exact buffer sizes, which are the defensible numbers |
| `FU-6` | **`FU-1`'s own fix introduced a new error** (the orchestrator's suspicion, **confirmed** by my own measurement). `tests/corpus/manifest.toml:190-194` now says every Q2M frame's crop origin sits "against a **non-zero ActiveArea**"; measured, every Q2M `ActiveArea` is `top 0, left 0` — a **zero** origin. In `SPEC-014`'s vocabulary that sentence asserts the opposite of the spec's load-bearing claim, in the very note corrected to prevent that belief. `unrun-docs-carry-errors`, third instance | `fixed` — replace "non-zero ActiveArea" with "an ActiveArea that is present but has a **zero** origin". Consider also adding evidence to `unrun-docs-carry-errors` (N=2 → 3): **a doc correction is itself an unrun doc**, which is a sharper statement than the signal currently carries |
| `FU-7` | **In CI, four of `tests/develop.rs`'s six tests have zero executing assertions.** With no corpus (CI: `0/7 present`), `black_and_white_levels_map_to_the_endpoints`, `the_three_stage_crop_produces_the_measured_dimensions`, `orientation_six_swaps_the_output_dimensions` and `an_unrotated_sibling_keeps_its_dimensions` all return before asserting anything, and all four report `ok`. So **`AC1`, `AC3` and `AC5` carry no evidence whatsoever in the one gate `no-panics-on-untrusted-input` requires to be *observed* green** — on the one spec in this project that has no oracle to fall back on. Structural and known, but sharper here than in any prior spec. (Mitigations I measured, in the build's favour: `develop::tests`' 13 unit tests **do** run in CI, so `AC4`, `AC2`'s tier-A half and `AC6` are genuinely covered there) | `signal: ci-cannot-prove-bit-exactness` — add this as evidence. The signal already owns "a green badge does not mean the decoder is correct"; what is new is that a **no-oracle** spec concentrates the loss, and that the cheap partial fix is the one `SPEC-013/FU-1` already proved works: move a tier-B assertion onto a hand-built fixture |

### 12. Two things I was asked to judge, answered plainly

- **`DEC-019`'s confidence (0.75) — should it move?** **No. Leave it at 0.75.** My work
  strengthened the *positive* evidence (premise re-measured on 7 files, `AC4` watched red to
  the exact wrong value, the `dnglab`/`exiftool` metadata oracle running green on the one
  real non-zero-origin file, ImageMagick confirming the orientation half of the same
  pipeline). But the thing that caps the confidence has **not** changed: **zero decodable
  real files with a non-zero `ActiveArea` origin.** Raising it on the strength of a fixture
  this project wrote is precisely the "accumulated authority" failure §15's own provenance
  note warns about. `DEC-019`'s Validation section already names the event that should move
  it; wait for that event.
- **§16's confidence flag.** No referenced decision is below 0.6 — `DEC-002` 0.72
  (`proposed`), `DEC-004` 0.92, `DEC-016` 0.85, `DEC-018` 0.80, `DEC-019` 0.75. No yellow
  flag fires. `decisions-audit`'s `DEC-002`-is-still-`proposed` warning is advisory and
  pre-existing (`SPEC-012` carries it too); nothing in this build assumes `rayon`, ambient
  `std`, or runtime SIMD dispatch.
- **§15 check 7 (`cost.sessions` for prior cycles):** present and correct — `design`
  null-with-note (permitted, main-loop), `build` `44,845,024`. Nothing missing, nothing to
  flag. ⚠ For the record only: the reconciliation established that the build's self-report
  is **9.9% low** (transcript settles at 49,294,128) and `cost.sessions` still carries the
  reported figure. Not mine to repair, and not a gate failure.

### Drift and new artifacts

- **New decisions emitted:** none. `DEC-018` and `DEC-019` were emitted by the build; both
  are `accepted`, both scope `src/develop.rs`, and I found no contradiction between them
  (the same shape `decisions-audit` already tolerates for the `DEC-012`/`DEC-015` pair).
- **Deviations from spec:** none found. All 8 ACs met; `AC5` met as written (see above).
- **Follow-up work identified:** `FU-2`…`FU-7`, all follow-ups, all dispositions proposed
  above. None crosses `SPEC-014`'s ship undecided.
- **Repo changes I made:** exactly two, both documentation — the timeline's verify line
  (`[ ]` → `[~]` on entry, `[x]` on completion) and this handback. **No code was changed.**
  Both mutations were reverted and verified byte-identical by md5. I did **not** run
  `handback-sync`, did **not** open the PR, did **not** merge, and fixed **nothing** I found.

### Cost self-report

- **Tokens (total): 13,400,000**
- **Estimated USD: $31.15**
- **Duration (minutes): ~25**
- **Source of the number:** this session's own transcript,
  `~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/152ae964-ca8c-4e65-8fbf-d9867b04bd82.jsonl`,
  **deduped by `message.id`** — 143 usage objects, **76 unique ids**, all reporting
  `message.model = claude-opus-5`. No sub-agent was used, so there is no `subagent_tokens`
  split.

| Component | Tokens (measured floor) | Rate (opus, per M) | Cost |
|---|---:|---:|---:|
| `input_tokens` | 152 | $15.00 | $0.00 |
| `output_tokens` | 52,930 | $75.00 | $3.97 |
| `cache_creation_input_tokens` | 188,581 | $30.00 | $5.66 |
| `cache_read_input_tokens` | 10,881,847 | $1.50 | $16.32 |
| **Measured floor** | **11,123,510** | — | **$25.95** |
| **Reported, +20%** | **13,400,000** | — | **$31.15** |

**Rounded up, and here is the reasoning rather than a gesture.** The floor is measured at the
moment of writing and cannot include the turns that write this handback — the failure this
repo has now measured twice (`SPEC-014` build 9.9% low, `SPEC-013` build 15.4% low). I used
**+20%**, above both observed misses, because this handback is unusually long and its turns
are cache-read-heavy. Priced **per-component**, not at a flat rate
(`flat-rate-overstates-cached-sessions`): `cache_read` is 98% of the tokens but 63% of the
cost. Rates are the standard published opus-tier list prices, the same ones `HANDOFF-031`
used — treat `estimated_usd` as the order-of-magnitude figure AGENTS.md §4 asks for, not an
invoice.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Almost nothing; this was the best-prepared verify handoff I have seen in this repo, and
   the seven "checks the orchestrator did not make" were all real and all productive. The one
   thing that cost me a loop was **`zsh` does not word-split unquoted variables**, so my first
   eight-orientation loop silently ran once instead of eight times and printed a result that
   looked plausible (orientation "1" labelled `LeftBottom`). That is this repo's own rule 2 in
   a different costume — *the output was wrong rather than erroring* — and the only reason I
   caught it is that the label and the number disagreed. Worth noting for `AGENTS.md` §6: this
   repo's shell is `zsh`, and a `for x in $var` loop that works in the docs will not work here.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Yes, one, and it materially changed my answer to check 7: **`SPEC-005`'s
   `dnglab_crop_origin_is_active_area_plus_default_crop_origin` is a live comparison oracle
   over `DEC-019`'s convention**, running on all six DNGs including the only non-zero-origin
   file. `DEC-019` cites it; the handoff's framing ("this spec's own tests are the only check
   that exists") does not, and it is a stronger position than the framing allows. The claim
   that is exactly true is narrower: *no oracle covers this spec's **pixel** path.* The
   metadata half of `DEC-019` is oracle-covered today.
3. **If you did this task again, what would you do differently?**
   — Build the out-of-repo probe crate **first**, before running any gate. Nearly every
   finding here (`FU-2`, `FU-3`, `FU-4`, and both real-data measurements) came from a 30-line
   scratch binary linking `irradiance` by path, and it let me measure the shipping code
   directly without touching the tree or fighting `report-do-not-repair`. It is a better tool
   than mutation for "is this claim actually asserted anywhere", and it is reusable — I would
   propose it as a standing verify affordance rather than something each reviewer reinvents.

4. **Where was the worst defect caught?** — `verify`
   *(`FU-3` and `FU-4` are the two that matter, and both are coverage/documentation gaps
   caught here rather than defects that escaped. No incorrect behaviour was found in `src/`.)*
