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
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify, not a measurement
                                    # (signal `tier-map-predicts-what-it-should-record`).
                                    # CORRECT THIS to whatever your own system prompt
                                    # reports as `message.model` before you hand back —
                                    # the build hint has now been wrong 9 times running.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-05
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: feat/spec-014-level-normalization-geometry-orientation
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

- **Branch / SHA reviewed:**
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **Verdict:** ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

- **Tokens (total):**
- **Estimated USD:**
- **Duration (minutes):**
- **Source of the number:**

### Drift and new artifacts

- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
