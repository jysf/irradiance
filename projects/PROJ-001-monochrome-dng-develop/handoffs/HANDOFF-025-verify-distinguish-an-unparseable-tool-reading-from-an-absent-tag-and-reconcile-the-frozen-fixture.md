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
  id: HANDOFF-025
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ✅ CORRECTED by the verify cycle, not inherited: every
                                   #   message in this session's transcript reports
                                   #   message.model = claude-opus-5 (134 messages, 79 unique
                                   #   message.id). The hint happened to be right this time —
                                   #   tier_map is now 2 for 5, not 1 for 4.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-03
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-010

project:
  id: PROJ-001
  stage: STAGE-005
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
  tokens_total: 10362360           # REAL combined count — what cost-audit reads
  estimated_usd: 21.33             # per-component at Opus list rates — see notes
  duration_minutes: 15
  branch: feat/spec-010-tri-state-tool-reading
  pr: null                         # not opened — HANDOFF-025 return criterion 8
  completed_at: 2026-09-03
  notes: "VERDICT ⚠ PUNCH LIST — one ship-blocker (SB-1), one new follow-up (FU-3),
    and the build's `closed` disposition on FU-2 re-raised. tokens_total is DEDUPED BY
    message.id (79 unique, 134 messages) from this session's own transcript
    (~/.claude/projects/.../6670ede2-143b-4cb3-a9cd-7aa29e855fe5.jsonl), summed across
    input+output+cache_creation+cache_read per AGENTS.md §4 ('one combined number'),
    captured immediately before writing this block — still an undercount, since the
    write itself and everything after it is not in the number. estimated_usd is
    PER-COMPONENT at the Opus-family published list rates for the model that actually
    ran (message.model = claude-opus-5, NOT tier_map): input 158 x $15 = $0.00,
    output 42,367 x $75 = $3.18, cache_creation 154,683 x $18.75 = $2.90, cache_read
    10,165,152 x $1.50 = $15.25. ⚠ .repo-context.yaml's blended rate_per_mtok 6.60
    would give $68.39 for the SAME token count — 3.2x higher — because a blended rate
    prices 98% cache-read traffic as if it were fresh input. Raised as FU-4.
    duration_minutes is the transcript's own first->last delta (14.4 min), not
    wall-clock. `just handback-sync` NOT run and PR NOT opened, per return criteria 8."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-025: Verify SPEC-010 — the tri-state tool reading, at `f4841b3`

## Delegation Summary

Verify `SPEC-010` at **`f4841b3`** on `feat/spec-010-tri-state-tool-reading`
(pushed, not merged; `main` at `2c0aaed`). The metadata oracle can now tell an
**absent** tag from an **unreadable** one, and an unreadable reading is a
mismatch unless `Sensor::malformed_tags` names the same tag.

## ⚠ READ THIS FIRST — the shipped code is a RECONSTRUCTION

The build disclosed, unprompted, that during its own red-proof work
`git checkout -- tests/support/tools.rs` **wiped the entire SPEC-010 change**,
not just the temporary mutation, because nothing was staged. It **redid the
edits from context** and re-verified.

Self-caught and disclosed, which is the right behaviour and should be credited.
But it means **the shipped implementation is a second writing of itself**, and
the only thing standing between it and a silently dropped requirement is that
the tests pass — and those tests were written by the same session, in the same
sitting, partly before the wipe.

**Treat every acceptance criterion as unverified.** Do not spot-check. The
failure mode to hunt is not a bug; it is an *omission* that no existing test
covers because the test that would have covered it was never rewritten.

## What the orchestrator already reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + both SHAs on `origin` | ✅ `23e413f`, `f4841b3` |
| **CI green on both** | ✅ checked on the runs, not the record — `constraints.yaml` requires the observation |
| `src/`, `Cargo.toml`, `Cargo.lock` untouched | ✅ `git diff main...HEAD` empty on all three |
| 95 tests (was 87), 0 failures | ✅ summed across all six targets, corpus present |
| all 8 named tests exist **exactly once** | ✅ per-target `-- --list`, anchored match, each `1` |
| `ToolValue<T>` is a real tri-state | ✅ `Absent` / `Unreadable(Vec<u32>)` / `Value(T)`, raw values preserved |
| `compare_optional` is **one generic arm** | ✅ per-*state*, not per-tag — the guard `DEC-013` chose and shipped wrong |
| `diff()` not dangling | ✅ a one-line wrapper over `diff_with_malformed(.., &sensor.malformed_tags)`, 10 callers |
| `DEC-014` `accepted`, `DEC-013` still `rejected` | ✅ |

**Two things the build did better than the spec asked**, worth confirming rather
than assuming: the red-proof is **in-test and permanent**, not a one-off manual
mutation — and it exercises the **real shipped comparator** via
`diff_with_malformed(sensor, reading, &[])` rather than a hand-written
re-derivation of it.

## Three findings to confirm or kill

**F-a — `req()` still truncates to its head, and `AC4` may be narrower than the
finding it carries.** `tests/support/tools.rs:296`. The build **documented** the
scope call inline with reasoning, and `AC4` as I wrote it (`BlackLevel = "512
999"` must not read `Some(512)`) **is** met, because `BlackLevel` is optional.
But `SPEC-005/FU-2`'s stated hazard was *"latent on today's mono corpus, **live
at `SamplesPerPixel > 1`**"* — and `BitsPerSample` on a 3-sample file reads
`"8 8 8"`, which `req()` still silently takes as `8`. So `FU-2` is **not closed**
even though its owning AC passes. ⚠ **That is my AC's imprecision as much as the
build's scope call** — judge it as a design finding if you agree, and say so.

**F-b — the red-proof passes vacuously without a corpus.** Measured:
`IRRADIANCE_CORPUS_DIR=/nonexistent cargo test --test metadata_oracle` →
**29 passed in 0.06 s**, including
`removing_the_malformed_comparison_turns_k3iii_red` and its control. This is
`SPEC-005/FU-3`'s shape, now on the **red-proof itself**, which is worse than
where it was found: a proof that cannot run is indistinguishable from a proof
that passed. `just test` names the missing files first, so it is visible through
the recipe — confirm that, and judge whether it is enough.

**F-c — `DEC-013` kept `rejected` with a pointer rather than `superseded_by:
DEC-014`.** The build's reasoning is that `DEC-013` was *wrong on three counts*,
not merely improved upon, and `superseded` would imply the latter. That is a real
judgement about what the two states mean. Check `decisions-audit` treats the
pair sanely and that a reader landing on `DEC-013` is actually routed to
`DEC-014`.

## Your own checks — the list above is not the job

The most valuable outcome is a **fourth** finding, and the reconstruction is
where to look. Suggestions in this repo's grain:

1. **Walk `SPEC-010`'s eight ACs against the code**, not against the test names.
   `AC5` (fixture reconciled against the live tool) and `AC7` (the doc comment
   and `DEC-013` brought true) are prose-shaped and the likeliest to have been
   lost and not noticed.
2. **Mutate each arm of `compare_optional` in turn** — `Absent`, `Value`,
   `Unreadable` — and confirm each has a test that dies. One generic arm is
   elegant and is also three behaviours behind one `match`.
3. **Check `tri_state`'s classifier per field.** `Value` only for the exact
   arity: does a 3-element `ActiveArea` really become `Unreadable` and not
   `Value` of something?

## Return Criteria

1. **Ten gates + `just lint-ci` + `just oracle-meta`**, run by you, pasted.
   Sum across **all six** targets; a zero-match `cargo test <name>` exits 0.
2. ⚠ **`just lint-ci`, not `just lint`** — local clippy is 0.1.97, CI floats at
   0.1.98. And **observe CI green on the SHA you approve**, per
   `constraints.yaml` as amended at STAGE-001's close.
3. **Watch the red-proof fail yourself** (§15 check 9), with the corpus present.
4. **Fuzz** (§15 check 10) — the build claims 10.9 M execs, seeds unchanged.
5. Every mutation: **assert it changed the file and compiled** before concluding.
   ⚠ And heed the build's own lesson — **stage your work before running
   mutate-and-revert experiments on a file you have edited.**
6. Handback with a real `tokens_total` **deduped by `message.id`**, said so, and
   `estimated_usd` per-component at the rates for the model that **actually
   ran** (`message.model`, not `tier_map`). Capture late — the floor convention
   measured ~17% low.
7. **Correct `handoff.to_agent`** to what actually ran.
8. Do **not** run `just handback-sync`; do **not** open the PR.
9. Findings `SB-N` / `FU-N` for **this spec** from 1, each with which of §15's
   four dispositions you think it wants. ⚠ `SPEC-005/FU-9` (`is_active()` ignores
   `status`) is confirmed still open and is **out of this spec's `tests/`-only
   scope** — the build flagged rather than fixed it, which was correct. It needs
   a disposition at ship, not a fix here.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

**Verdict: ⚠ PUNCH LIST.** One ship-blocker, one new follow-up, and the build's
`closed` disposition on its own `FU-2` re-raised. Everything the handoff asked me
to reproduce, I reproduced; the shipped code is sound and the reconstruction lost
nothing I could find in the *implementation*. What it lost — or never had — is one
test, and it is the one that matters most for this spec's headline claim.

Ran as `claude-opus-5` (corrected in the front-matter from the dispatch hint, which
happened to be right this time).

### Ten gates + `lint-ci` + `oracle-meta`, run by me

`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`

| # | gate | result |
|---|---|---|
| 1 | `just build` | `Finished 'release' profile` |
| 2 | `just test` | **95 passed, 0 failed**; `corpus: 7/7 present — no tier-B test will skip` |
| 3 | `just lint` | `clippy -D warnings` ok · `cargo fmt --check` ok |
| 4 | `just typecheck` | ok |
| 5 | `just deny` | `licenses ok` |
| 6 | `just deny-fuzz` | `licenses ok` |
| 7 | `just lint-red-proof` | ✓ control clean (0) → injection rejected (101) → all five lints fired, severity run too |
| 8 | `just lint-no-allow` | exit 0 |
| 9 | `just msrv` | 1.90.0, `Finished` |
| 10 | `just fuzz 120` | **25,064,173 runs in 121 s, 0 crashes**, `cov: 681 ft: 2288 corp: 383` |
| + | `just lint-ci` | ok — `clippy 0.1.98 (88d9e12ae1 2026-08-18)`, confirmed via `--version`, not 0.1.97 |
| + | `just oracle-meta` | 29 passed, 0 failed |

Seeds byte-identical after the fuzz run (`git status --porcelain` empty;
`git diff HEAD -- fuzz/` empty — `fuzz/corpus/ifd` is gitignored, 26 seed files
untouched). Also green: `just validate` (16 artifacts), `just cost-audit`.

**95, summed across all six targets** — `unittests src/lib.rs` 45, `unittests
src/bin/irr.rs` 0, `tests/corpus_manifest.rs` 9, `tests/ifd_reader.rs` 12,
`tests/metadata_oracle.rs` 29, doc-tests 0. Confirmed twice: from `test result:`
lines, and independently from per-target `-- --list` (95 lines matching `: test$`).

⚠ `for t in ... "--test corpus_manifest"; do cargo test $t ...` **silently reports
zero** in zsh — zsh does not word-split unquoted expansions, so `$t` arrives as one
argument, cargo errors, and `2>/dev/null` eats it. That is AGENTS.md §16 rule 3's
exact shape (a non-match becoming control flow) in the verifier's own harness. I hit
it, caught it because 0 was implausible, and re-ran with explicit invocations.

### The eight named tests — per-target `-- --list`, anchored, summed

Grepped `^<name>: test$` against the concatenated per-target listings of **all six**
targets. Every one: **count = 1**.

```
an_absent_tag_and_a_garbled_one_are_not_the_same_reading            1
a_garbled_tool_reading_is_a_mismatch_when_we_read_the_tag_fine      1
a_garbled_tool_reading_agrees_when_we_also_recorded_it_malformed    1
k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason         1
a_multivalued_reading_does_not_truncate_to_its_head                 1
the_frozen_fixture_still_matches_the_live_tool                      1
removing_the_malformed_comparison_turns_k3iii_red                   1
the_malformed_comparison_control_is_green                           1
```

### CI observed green on the SHA I am judging

Read off the runs via `gh api repos/{owner}/{repo}/commits/<sha>/check-runs`, not
off the record. **9/9 `success` on all three SHAs**, `lint policy red-proof` among
them (the job `constraints.yaml` was amended for):

| SHA | checks |
|---|---|
| `23e413f` | 9/9 success |
| **`f4841b3`** | **9/9 success** — the shipping SHA |
| `a7582ce` | 9/9 success |

### §15 check 9 — I watched the oracle go red, with the corpus present

Every mutation below asserted **exactly one** match before replacing, asserted the
file's SHA-256 **changed**, asserted it **compiled**, and asserted it restored
**byte-identical**. Backups were taken to the scratchpad first — the build's lesson,
heeded (`git checkout --` was never used on a file I had edited).

**AC6's real mutant, on the literal shipped `diff()`** — not its parameterized twin:

```
- diff_with_malformed(sensor, reading, &sensor.malformed_tags)
+ diff_with_malformed(sensor, reading, &[])

→ metadata_matches_exiftool_on_every_corpus_file ... FAILED
  PENTAX-K3III-MONO/K3III.DNG: 1 field(s) disagree with exiftool:
    BlackLevelRepeatDim: ours=None, theirs=Unreadable([1])
  k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason ... FAILED
  a_garbled_tool_reading_agrees_when_we_also_recorded_it_malformed ... FAILED
  → 26 passed; 3 failed        (control, unmutated: 29 passed)
```

Byte-identical to the message the build recorded — independently reproduced. Note
`theirs=Unreadable([1])`: the raw value survives into the mismatch message, which is
`## Outputs`' explicit requirement, exercised for real rather than asserted.

**And the red-proof itself going red**, which is the thing check 9 actually asks for:
under the `Unreadable => true` mutant, `removing_the_malformed_comparison_turns_k3iii_red`
**FAILED**. A red-proof I watched fail.

**AC5's own red** — rotted `WhiteLevel 16383 → 16382` in the committed fixture text:
`the_frozen_fixture_still_matches_the_live_tool ... FAILED`, naming
`SPEC-005/FU-4: reconcile the frozen fixture rather than trusting it stale`. Restored.
The frozen fixture is byte-identical to a live run today (checked directly with
`exiftool -T -n -s3` on `L1021223.DNG`).

### Handoff check 2 — mutate each arm of `compare_optional` in turn

This is where the fourth finding is. Corpus present, all 29 oracle tests:

| mutation | result |
|---|---|
| `Absent => ours.is_none()` → `=> true` | **29 passed. NOTHING DIED.** |
| `Absent => ours.is_none()` → `=> false` | 1 failed (`metadata_matches_exiftool_on_every_corpus_file`) — tier B only |
| `Value(v) => ours == Some(*v)` → `=> true` | 1 failed (`oracle_goes_red_on_a_patched_tag_in_a_real_file`) |
| `Unreadable(_) => malformed_tags.contains(&tag)` → `=> true` | 2 failed (`a_garbled_tool_reading_is_a_mismatch_when_we_read_the_tag_fine`, `removing_the_malformed_comparison_turns_k3iii_red`) |

See `SB-1`.

### Handoff check 3 — `tri_state`'s classifier, per field

Measured through the real parse path, not read off the source. Every optional field
requires **exact** arity; nothing degrades to a `Value`:

```
active_area   from "0 0 5632"           = Unreadable([0, 0, 5632])     ← 3 of 4, correctly NOT Value
white_level   from "16383 16383 16383"  = Unreadable([16383,16383,16383])
active_area   from ""  (empty, not "-") = Unreadable([])               ← errs toward mismatch: right direction
```

`ActiveArea` needs `[a,b,c,d]`, `BlackLevelRepeatDim` `<[u32;2]>::try_from`,
`DefaultCropOrigin`/`DefaultCropSize` `[a,b]`, the scalars `[x]`. Check 3 clears.

### The eight ACs, walked against the code

| AC | verdict |
|---|---|
| AC1 absent ≠ unreadable, all four tags | ✅ all four asserted `assert_ne!`; `BlackLevelRepeatDim` additionally pinned to the exact two states |
| AC2 unreadable is a mismatch unless `malformed_tags` | ✅ and it dies under mutation |
| AC3 `K3III.DNG` green for a stated reason | ✅ the test asserts `Unreadable`, **not** `Absent`, before asserting agreement — the collapse cannot masquerade as the fix |
| AC4 no head-truncation | ✅ **as worded**, on the optional fields. See `FU-2`, re-raised |
| AC5 fixture reconciled against the live tool | ✅ three checks (frozen text vs live, frozen sensor vs live, live reader vs live), and it dies when the fixture rots. "Skip loudly" is the weak half — `FU-3` |
| AC6 red-proof both directions + control | ✅ in-test **and** by literal source mutation, both watched by me |
| AC7 doc comment + `DEC-013` brought true | ✅ `diff()`'s comment states the shipped three-way rule and stops predicting a future; `DEC-014` `accepted` |
| AC8 gates + CI observed | ✅ above |

Non-goals held: `git diff main...f4841b3 -- src/ Cargo.toml Cargo.lock` is **empty**;
`DEC-012` untouched; the dnglab side untouched; `tests/support/corpus.rs` untouched.

### The three handed-down findings

**F-a — CONFIRMED, and the build's `closed` on it is wrong.** Re-raised below as
`FU-2`. I agree with your read that `AC4` as written is met and that the imprecision
is the AC's, not the build's code — so this is a **design finding about the
disposition**, not a defect in what shipped.

**F-b — CONFIRMED, and worse than stated.** Re-raised below as `FU-3`.

**F-c — CONFIRMED SOUND. No finding.** `just decisions-audit`: **0 structural
errors**, and exactly one nudge — *"⚠ DEC-013 and DEC-014 govern the same scope …
if one wins, mark the other superseded"*. A warning, not a failure, and arguably the
correct output: both records genuinely do govern that file, for different reasons. A
reader landing on `DEC-013` meets the `DEC-014` pointer in the second blockquote,
above every line of the original text; `just decisions-audit --changed main` lists it
as *"DEC-013: … — **REJECTED**"*, so even the one-line surface carries the warning.
The build's judgement — that `superseded` would understate "wrong on three counts" —
holds, and it is a better answer than the mechanical one.

---

## Findings

Per-spec numbering, continuing `SPEC-010`'s own sequence (`FU-1`/`FU-2` were raised
by the build in `HANDOFF-024`). Dispositions are my recommendation for ship.

### `SB-1` — ship-blocking — `compare_optional`'s `Absent` arm has no test that dies

**Measured.** Replacing `ToolValue::Absent => ours.is_none()` with `=> true` leaves
**all 29 oracle tests green with the full corpus present**. One of the comparator's
three arms is unproven.

⚠ **This is the exact property `DEC-013` was rejected for.** Count 1 of its rejection,
verbatim: *"Removing the … condition left all 21 oracle tests green … A guard that
nothing dies without is a guard nobody knows works."* `SPEC-010` exists to replace
that guard, and shipped a new one carrying the same defect in a different arm.

⚠ **Sharper: without a corpus, the arm is dead in BOTH directions** — `=> true` and
`=> false` each leave 29 green. The tier-A half is the only half CI runs, so **CI has
never exercised `compare_optional`'s `Absent` arm at all.** `Absent => false` dies
only on `metadata_matches_exiftool_on_every_corpus_file`, a tier-B test.

**The code is correct — do not change the comparator.** The defect is the missing
proof, and it lands squarely on `AC1`, whose entire subject is this distinction:
`an_absent_tag_and_a_garbled_one_are_not_the_same_reading` proves the two *readings*
differ and never proves the *comparator* acts on the difference.

**Fix, already verified by me** — tier A, no corpus, no tool; passes on the honest
tree and **FAILS under the `Absent => true` mutant** (both measured):

```rust
#[test]
fn an_absent_tool_reading_is_a_mismatch_when_we_read_a_value() {
    // fixture_sensor()'s ActiveArea is Some(..) — exiftool saying the tag is
    // NOT IN THE FILE must disagree with that. This is compare_optional's
    // `Absent` arm in its DISCRIMINATING direction; nothing else exercises it.
    let sensor = fixture_sensor();
    let mut reading = fixture_reading();
    reading.active_area = reading_with_column(COL_ACTIVE_AREA, "-").active_area;
    assert_eq!(reading.active_area, tools::ToolValue::Absent);

    let mismatches = tools::diff(&sensor, &reading);
    assert_eq!(mismatches.len(), 1, "exactly one field must disagree, got {mismatches:?}");
    assert_eq!(mismatches[0].field, "ActiveArea");
}
```

**Disposition: `fixed`** — in this spec's punch-list round. Ten lines, tier A, and it
brings the arm count from two-of-three proven to three-of-three.

### `FU-2` — RE-RAISED — the build's `closed` disposition fails AGENTS.md's own test for a good close

The build closed `FU-2` (`req()` truncates a multi-valued required tag) with:
*"no test regresses if this is revisited when `SamplesPerPixel > 1` lands (PROJ-002)."*

AGENTS.md, *Where an unresolved follow-up goes*: **"A close whose trigger is a *test
that will fail* is a good close; a close whose trigger is someone remembering is
not."** That close's own sentence states its trigger is someone remembering.

And the hazard is now **measured on both halves**, so it is no longer a prediction:

1. **exiftool really does emit space-separated per-sample values.** On a 4×4 RGB TIFF
   built with ImageMagick (2026-09-03), `exiftool -T -n -s3 -IFD0:SamplesPerPixel
   -IFD0:BitsPerSample` prints `3` and **`16 16 16`**.
2. **`req()` swallows it.** Through the real parse path: `BitsPerSample = "8 8 8"` →
   `bits_per_sample = 8`, and `diff()` against a sensor reading `bits_per_sample = 8`
   returns **`[]`** — diffs clean, exactly `SPEC-005/FU-2`'s stated hazard.

**Fix, already verified by me** — compiles and leaves **all 29 tests green on the
real corpus**, inside this spec's `tests/`-only scope:

```rust
match values_for(fields, tag).map(|v| v.as_slice()) {
    Some([x]) => Ok(*x),
    Some(multi) => Err(ToolError::Parse {
        tool: "exiftool",
        message: format!("{tag} is required and single-valued, got {multi:?}"),
    }),
    None => Err(ToolError::Parse { /* … absent, unchanged … */ }),
}
```

That converts "someone remembers at PROJ-002" into a loud typed error the moment a
multi-sample file arrives — which is the whole difference between the two kinds of
close.

**Disposition: `fixed`** (preferred — eight lines, zero regression, measured), or
`spec: SPEC-NNN` naming the PROJ-002 sample-unpack work. **Not `closed`.**

### `FU-3` — follow-up — `corpus-status` states something false when the tools are absent

**Measured.** Corpus fully present, `exiftool` off `PATH`:

```
$ cargo test --all-features --test metadata_oracle     → 29 passed in 0.02 s
$ cargo run --example corpus-status                    → corpus: 7/7 present —
                                                          no tier-B test will skip
```

Every tier-B oracle test skipped. The one loud surface does not merely stay quiet —
**it asserts the opposite of what happened.** That is a strictly worse failure than
`F-b` as handed to me, because `just test`'s contract is what a reader trusts instead
of reading the suite.

Two supporting measurements:

- `just oracle-meta` — the recipe `AC8` and this handoff's return criteria both name —
  runs no `corpus-status` at all. `IRRADIANCE_CORPUS_DIR=/nonexistent just oracle-meta`
  → **29 passed, zero SKIP lines** in the output. The 27 `SKIP` lines exist but are
  captured; only `--nocapture` shows them. `corpus.rs:303-305`'s own comment already
  concedes this (*"NOT the surface that satisfies the spec's criterion 4"*), so the
  corpus half is a known, documented condition — the **tool** half has no surface at all.
- `SPEC-010` enlarged the blast radius: the red-proof, its control, and `AC5`'s
  reconcile all now sit behind this gate. `IRRADIANCE_CORPUS_DIR=/nonexistent cargo
  test --test metadata_oracle` → 29 passed in 0.06 s, `removing_the_malformed_
  comparison_turns_k3iii_red` among them. **A proof that cannot run is
  indistinguishable from a proof that passed.**

**Disposition: `spec: SPEC-NNN`** — teach `corpus-status` to probe `exiftool`/`dnglab`
and stop claiming "no tier-B test will skip" when they are missing, and run it from
`oracle-meta` as well as `test`. One file, one fix — a spec, not a signal.

### `FU-4` — follow-up — the blended `rate_per_mtok` overstates a cache-heavy cycle 3.2×

`.repo-context.yaml:66` carries a single `rate_per_mtok: 6.60`. This cycle: 10,362,360
tokens, of which **98.1% is `cache_read`**, priced at roughly a tenth of fresh input.
Per-component at the model's list rates → **$21.33**. The blended rate on the same
count → **$68.39**. Whichever is right, `just calibration` is comparing estimates to
actuals through a multiplier that moves with cache hit rate, which is not a property
of the work.

**Disposition: `signal: <new>`** — this is a property of the cost *convention*, recurring
across every cycle, not a defect in one file. Pairs with the ~17% floor-convention
undercount the return criteria already warn about, which pushes the same number the
other way.

### `SPEC-005/FU-9` — confirmed still open, still correctly out of scope

`is_active()` (`scripts/decisions-audit.sh:152-156`) reads only `superseded_by`, never
`status`. Confirmed by running it: `just decisions-audit --changed main` lists
`DEC-013` as governing `tests/support/tools.rs` alongside `DEC-014`. The build flagged
rather than fixed it — correct, `scripts/` is outside this spec's `tests/`-only scope.

Worth recording that the build **mitigated** it without being asked: `DEC-013`'s title
now ends `— **REJECTED**`, so the audit's own one-line output carries the warning even
though the filter does not. That is "fix the verb, not the filter" honoured in spirit
at zero cost. Still **needs a disposition at ship**, not a fix here.

---

### What I could not fault

The reconstruction did not lose anything I could find in the implementation, and two
things are better than the spec asked for, both confirmed rather than assumed:

- The red-proof is **in-test and permanent**, and it calls the **real shipped
  comparator** — `diff_with_malformed` is the exact function `diff()` delegates to, a
  one-line wrapper, 10 call sites. I verified the seam is not a re-derivation by
  mutating `diff()` itself and getting the identical failure.
- `k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason` asserts `Unreadable`
  **before** asserting agreement. Without that line the test would pass under the very
  collapse this spec closes. It is the difference between a test and a test that means
  something.

`compare_optional` really is one generic arm taking a tag *number*, per-state and not
per-tag — `DEC-013`'s Option C, implemented as written this time.

### Reflection (§15)

1. **What would I do differently next time?** Assert the harness before trusting the
   harness. My per-target `-- --list` loop returned 0 for three of six targets because
   zsh does not word-split `$t`, and `2>/dev/null` swallowed cargo's complaint. I
   caught it only because 0 was implausible — which is luck, not method. AGENTS.md §16
   rule 3 is written about gates; it applies just as hard to the verifier's own
   scaffolding, and I had just finished reading it.
2. **Does any template, constraint, or decision need updating?** Yes, and it is `FU-3`'s
   real shape: `oracle-must-be-shown-red` says every oracle ships with a deliberate-fault
   test, and says nothing about that test being *reachable* in the environment that runs
   it. `SPEC-005/FU-3`, this handoff's `F-b`, and `FU-3` here are the same defect at
   three widening radii. The constraint should require that a red-proof either runs in
   CI or fails loudly when it cannot — recorded as `FU-3`'s spec rather than acted on
   here.
3. **Is there a follow-up spec I should write now before I forget?** `FU-3`. It is one
   file, it has an owner-shaped fix, and it is the third appearance of its class.
4. **Where was the worst defect caught?** `verify` — `SB-1`, an untested comparator arm,
   found by mutation and not by any existing test.
5. **What can a user do now that they couldn't before?** Confirmed as the build predicted:
   before, a garbled reading of any of the four multi-valued tags diffed clean against
   our own `None` — `SPEC-005` measured 5/5. After, it is a mismatch unless
   `malformed_tags` names the same tag, and I watched the guard's removal turn
   `K3III.DNG` red with `theirs=Unreadable([1])` printing exiftool's raw value. The one
   caveat on that sentence is `SB-1`: the *absent* half of the distinction is enforced by
   code no test defends yet.
