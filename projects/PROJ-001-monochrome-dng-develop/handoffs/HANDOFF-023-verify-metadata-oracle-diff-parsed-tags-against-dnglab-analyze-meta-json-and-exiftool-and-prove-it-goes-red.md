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
  id: HANDOFF-023
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ CONFIRMED to what ACTUALLY ran, measured from THIS
                                   #   session's own transcript
                                   #   (9c41c779-5edc-4824-a144-d3d8eafa9227.jsonl):
                                   #   110/110 usage objects report message.model =
                                   #   claude-opus-5. Checked, not inherited — the handoff
                                   #   was explicit that being right last round is not
                                   #   evidence. tier_map.verify is now 2 for 2 and
                                   #   SPEC-007/FU-6 is 2 for 5 overall.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-005

project:
  id: PROJ-001
  stage: STAGE-001
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
  tokens_total: 5683739            # REAL combined count — what cost-audit reads
  estimated_usd: 13.75             # tokens_total × your rate, or your harness's number
  duration_minutes: 12
  branch: feat/spec-005-metadata-oracle
  pr: null
  completed_at: 2026-08-22         # YYYY-MM-DD
  notes: "Verdict ✅ APPROVED at 5b1aef7. SB-1 is cleared; no SB-2. Four follow-ups (FU-8..FU-11), none holding the spec. DEDUPED BY message.id and I say so: 110 usage objects, 47 distinct ids, raw 12,944,378 vs deduped 5,683,739 = 2.28x, 96.8% cache-read. Components: input 94, output 37,138, cache-write 144,318, cache-read 5,502,189. estimated_usd computed PER-COMPONENT at published OPUS rates ($15/$75/$18.75/$1.50 per M) because message.model reads claude-opus-5 on all 110 objects — checked against my own transcript, not inherited from tier_map. THIS IS A FLOOR and the handoff's own warning applies to it: the convention runs ~17% low, so expect ~6.6M / ~$16 once this session closes. I REPRODUCED the handoff's floor-bias measurement independently before trusting it — round 1's transcript (cabae9fc) gives 10,203,870 deduped over 74 distinct ids, 139/139 claude-opus-5, exactly as HANDOFF-023 states. THE ONE CLAIM I WAS TOLD NOT TO INHERIT REPRODUCES VERBATIM: with the FU-1 fix simulated as the doc comment describes it, metadata_matches_exiftool_on_every_corpus_file fails with 'PENTAX-K3III-MONO/K3III.DNG: BlackLevelRepeatDim: ours=None, theirs=Some([1, 1])'. I also measured the counterfactual the argument rests on and nobody asked for: with the guard RESTORED on top of that same simulation, all 21 tests go green — so the guard would indeed have absorbed the divergence silently. Both halves of the self-forcing argument hold. FU-8 is where it stops holding: FU-1's fix has at least three shapes and only two of them red. Every mutation asserted applied (shasum before/after) AND compiled (cargo build --all-features --tests exit 0); tree restored byte-identical (tools.rs back to 0b9e3a573dc08c9049727dc260b3233b9f8862913bc52e132ae5fdc2c0bc2d53) and 'git diff 5b1aef7 -- src/ tests/ Cargo.toml Cargo.lock' is empty. handback-sync NOT run."
  synced_at: 2026-08-22
---

# HANDOFF-023: Re-verify SPEC-005's punch-list fix — round 2, at `5b1aef7`

## Delegation Summary

**Round 2 of verify.** Round 1 (`HANDOFF-022`) returned ⚠ PUNCH LIST at
`418be15` — one ship-blocker (`SB-1`, `DEC-013` wrong on three counts), seven
follow-ups, nothing else holding the spec. This round reviews **only the fix**,
at **`5b1aef7`** on `feat/spec-005-metadata-oracle`.

**⚠ Read this first: the architect wrote the fix.** The orchestrator made the
`SB-1` change, rejected `DEC-013`, wrote the new doc comment, and took the
judgement call the reviewer explicitly delegated. That is the whole reason this
round exists — it is the architect grading their own homework, and the round
before it is the only reason we know the last self-graded artefact was wrong on
three counts.

**Findings continue this spec's sequence** (§15): round 1 used `SB-1` and
`FU-1`…`FU-7`, so start at **`FU-8`** / **`SB-2`**.

## What changed since the reviewed SHA — the entire delta

`git diff 418be15..5b1aef7` — six files, and only one is code:

| file | change |
|---|---|
| `tests/support/tools.rs` | the **only** functional change: one condition, one import, one doc comment |
| `decisions/DEC-013-…md` | `status: accepted` → **`rejected`**, rewritten, original text preserved verbatim below a line |
| `guidance/signals.yaml` | `tier-map-…` corrected to 1-for-4; a floor-bias measurement added |
| `SPEC-005`, `HANDOFF-022`, `HANDOFF-023` | records |

`src/`, `Cargo.toml`, `Cargo.lock` are untouched — verify that yourself.

## The fix, and the judgement inside it

**What changed in code.** `diff()`'s `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM) &&`
is gone; the now-unused import went with it. All eleven fields are compared
unconditionally.

**The judgement.** Round 1 said: *"decide there whether the guard stays (dead
until FU-1) or goes."* The orchestrator **removed** it, over the alternative of
correcting it to be genuinely generic. The stated reason:

> A dead guard is not neutral: it is a decision made in advance, on evidence that
> does not exist yet, that disarms the alarm which would have demanded it.
> Removing it makes `FU-1`'s fix **self-forcing** — fix `FU-1` and `K3III.DNG`
> reds immediately, so the real question gets decided deliberately and with a
> test, rather than absorbed silently.

**Round 1's own reviewer noted the opposite pull** — *"Fix that and the guard
becomes necessary. The record's conclusion may be right; its stated premise
isn't."* Both readings are defensible. **You are invited to disagree with the
call, not merely to check that it was executed.** If you think a corrected,
genuinely-generic guard was the better answer, say so and say why — that is a
legitimate `FU-8`, and possibly an `SB-2` if you think removing it loses a real
guarantee.

## THE ONE CLAIM YOU MUST NOT INHERIT

The new doc comment asserts, as a **measurement**:

> with the `FU-1` fix simulated — a one-element reading mapped to `Some([a, a])`
> instead of `None` — `metadata_matches_exiftool_on_every_corpus_file` fails
> immediately with `PENTAX-K3III-MONO/K3III.DNG: BlackLevelRepeatDim: ours=None,
> theirs=Some([1, 1])`

**Reproduce it.** The orchestrator ran that and nobody else observed it, which
makes it a self-report by exactly the rule this repo applies to everyone else
(§15 check 9). Patch `black_level_repeat_dim`'s parse in
`reading_from_fields` so a one-element reading survives, **assert the mutation
changed the file and compiled**, run the oracle, and confirm the failure names
that file and that field. Then restore byte-identical.

If it does *not* red, the doc comment is false and the whole
self-forcing-alarm argument for removing the guard collapses — that is an
`SB-2`, because it would mean the fix was justified by a measurement that does
not hold.

## Also check

1. **Is the removal actually behaviour-neutral today?** 87 tests summed across
   six targets, corpus present. Confirm nothing else moved.
2. **Does `DEC-013` now match what shipped?** It is `status: rejected` with the
   original preserved. Round 1's three counts should each be stated accurately —
   including count 3, which round 1 settled by *test* (malformed
   `BlackLevelRepeatDim` diffs `[]`; identically malformed `ActiveArea` still
   reds), not by reading. Check the record did not soften that.
3. **Did rejecting a decision leave dangling references?** `just decisions-audit`
   and `just decisions-index --check`. Does anything still cite `DEC-013` as
   though it were live — the spec, the CHANGELOG, `docs/`?
4. **`just validate`, `just cost-audit`, and the eight remaining gates.**
5. **Did the architect quietly widen scope?** Round 1 approved everything except
   `SB-1`. Anything in the delta that is not `SB-1`'s fix or its records is scope
   creep and should be called.

## Do NOT re-litigate

Round 1's `FU-1`…`FU-7` are **settled findings** and are dispositioned at ship,
not here. Do not re-argue them. The two claims round 1 killed — that `diff()` is
narrower than it looks, and that the dnglab uniqueness assertion is fake — are
**closed**; they were measured false (31 perturbations; six planted duplicates
all refused). Do not spend the round re-proving them.

`AC2` is met as written. Round 1 established that and the orchestrator accepted
the correction.

## Return Criteria

1. Ten gates plus `just oracle-meta` and `just decisions-audit`, re-run **by
   you**, pasted. Sum across **all six targets**.
2. **Both red-proofs watched failing by you** (§15 check 9) — the tier-A
   perturbation and the tier-B patched-tag, plus the simulated-`FU-1` alarm above.
3. **Fuzz** — the delta is test-only and `tests/support/tiff.rs` did not move, so
   a short run and a seed-hash comparison is enough. Say which you did.
4. Every mutation: **assert it changed the file and compiled** before concluding.
5. Fill the `handback:` with a real `tokens_total` **deduped by `message.id`**,
   and say you deduped. Compute `estimated_usd` **per-component at the rates for
   the model that actually ran** — read `message.model`, do not trust `tier_map`.
   ⚠ Round 1 recorded `8,500,000` as a floor; measured after that session closed,
   the same transcript gives `10,203,870` over 74 distinct ids — the convention
   runs ~17% low, always in the same direction. Capture as late as you can and
   say it is a floor.
6. **Correct `handoff.to_agent`** to what actually ran.
7. Do **not** run `just handback-sync`.
8. Findings labelled `SB-N` / `FU-N` continuing from **`FU-8`** / **`SB-2`**, each
   with which of the four dispositions you think it wants (§15).
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

**A clean ✅ is a fine outcome here and so is a second punch list.** The delta is
one condition, one import and one doc comment — but the last artefact the
architect self-graded was wrong three ways, and that is the prior you should
carry in.

## Handback

**Verdict: ✅ APPROVED** — `5b1aef7` on `feat/spec-005-metadata-oracle`.

`SB-1` is cleared. The guard is gone, `DEC-013` is `rejected` with all three
counts stated accurately, the delta is exactly `SB-1`'s fix plus its records,
and `src/`, `Cargo.toml` and `Cargo.lock` are byte-identical to `main`. **No
`SB-2`.** Four follow-ups, `FU-8`…`FU-11`, none of which holds the spec.

**The judgement call was right and I am not merely certifying that it was
executed.** I was invited to disagree; I measured instead, and the measurement
went the architect's way on both halves. What would have changed my mind is in
`FU-8`, which is the one finding I considered promoting to `SB-2` and did not.

---

### THE CLAIM I WAS TOLD NOT TO INHERIT — reproduced verbatim

Mutation: `black_level_repeat_dim`'s parse in `reading_from_fields` patched so a
one-element reading maps to `Some([a, a])` instead of `None`. Asserted applied
(`0b9e3a57…` → `1bb73d82…`), asserted compiled (`cargo build --all-features
--tests` exit 0).

```
thread 'metadata_matches_exiftool_on_every_corpus_file' panicked at tests/metadata_oracle.rs:105:9:
PENTAX-K3III-MONO/K3III.DNG: 1 field(s) disagree with exiftool:
  BlackLevelRepeatDim: ours=None, theirs=Some([1, 1])
```

That is the doc comment's sentence, word for word, file and field included. The
self-report holds.

### The counterfactual the argument rests on — measured, and nobody asked

The doc comment's *conclusion* is that leaving the guard in place "would have let
that decision happen silently, by absorption." That is a second claim and it was
not measured by anyone. So I measured it: guard **restored** on top of the same
`FU-1` simulation (asserted applied → `1f00ca31…`, compiled).

```
test result: ok. 21 passed; 0 failed
```

**Green. The divergence is absorbed without a word.** Both halves of the
self-forcing argument therefore hold:

| | guard present | guard removed |
|---|---|---|
| today | 21 green | 21 green (behaviour-neutral) |
| `FU-1` fixed (as simulated) | **21 green — silent** | **RED, naming the file and the field** |

### `FU-8` is where it stops holding

See the finding. Short form: the simulated fix is not `FU-1`'s fix, and under
`FU-1`'s fix **as round 1 actually specified it** the alarm does not fire.

---

### The ten gates, plus `oracle-meta` and `decisions-audit`, re-run by me

`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`

| gate | result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `just msrv` | exit 0 |
| `just deny` | exit 0 — `licenses ok` |
| `just deny-fuzz` | exit 0 — `licenses ok` |
| `just lint-red-proof` | exit 0 — control clean → injection rejected (101) → all five lints fired, and still fire without CI's `-D warnings` |
| `just lint-no-allow` | exit 0 |
| `just cost-audit` | exit 0 |
| `just validate` | exit 0 — 9 artifacts |
| `cargo test --all-features` | **87 passed**, summed across **six** targets |
| `just oracle-meta` | exit 0 — 21 passed |
| `just decisions-audit` | exit 0 — 0 structural errors, 2 pre-existing `DEC-000` scope warnings |
| `just decisions-index --check` | exit 0 — no `decisions/INDEX.md` committed, nothing to sync |

Summed per target, not read off one: `45 + 0 + 9 + 12 + 21 + 0 = 87` (lib ·
`irr` bin · `corpus_manifest` · `ifd_reader` · `metadata_oracle` · doc-tests).
**Identical to round 1's 87 at `418be15` — the removal is behaviour-neutral, and
nothing else moved.** All nine spec-named tests confirmed present by exact name
via `--test metadata_oracle -- --list` (21 listed); no zero-match green.

⚠ Note on the reviewed SHA: `HEAD` is `6d17c4d`, which adds only this handoff.
`git diff 5b1aef7..6d17c4d -- src/ tests/ Cargo.toml Cargo.lock justfile` is
empty, so everything above is measured on `5b1aef7`'s code.

### Both red-proofs watched failing by me (§15 check 9)

Detection removed, proof itself confirmed dead, control green so the failure is
attributable. Each mutation asserted applied and compiled; tree restored.

| red-proof | mutation | result | control |
|---|---|---|---|
| `oracle_names_the_one_field_that_was_perturbed` (tier A) | `if false && sensor.bits_per_sample != …` | **FAILED** — `exactly one field must disagree, got []` | `oracle_is_clean_on_an_unmodified_reading` ok; other 19 ok |
| `oracle_goes_red_on_a_patched_tag_in_a_real_file` (tier B) | `if false && sensor.active_area != …` | **FAILED** — `patching ActiveArea must produce exactly one mismatch, got []` | other 20 ok |

Plus the simulated-`FU-1` alarm above, which is the third red I watched.

### Fuzz (return criterion 3)

Short run **and** seed-hash comparison — both, as invited.
`PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd -- -max_total_time=120`
→ **28,562,478 runs in 121 s**, ~236k exec/s, `cov: 661 ft: 2183`, exit 0, **no
crashes** (`fuzz/artifacts/ifd/` empty). Seeds unchanged:
`find fuzz/seeds -type f | sort | xargs shasum | shasum` reads `6a5b9fb7…`
before and after — **the same digest round 1 recorded**. The delta is test-only
and `tests/support/tiff.rs` did not move, so this is the expected result and I
am saying so.

---

## Findings

### `FU-8` — the self-forcing alarm is real for two shapes of `FU-1`'s fix and absent for the one round 1 specified

**This is my sixth finding and the one that matters.** It is not that the doc
comment is false — I reproduced its sentence verbatim. It is that the sentence
is a *measurement of a proxy* and the paragraph around it states a *category*:

> ⚠ The day `FU-1` is fixed, `K3III.DNG` goes red here — and that is deliberate.

The thing measured was `[1] → Some([1, 1])`. That is not `FU-1`'s fix. `FU-1`'s
disposition, in round 1's own words, is:

> a tri-state on the tool side: `Absent` / `Unparseable(raw)` / `Value`, plus
> **comparing `malformed_tags` against it**

So I built that instead of arguing about it — `blrd_unparseable` added to
`ToolReading` as the `Unparseable` arm, populated from a non-empty reading that
fails `<[u32; 2]>::try_from`, and `diff()`'s `BlackLevelRepeatDim` arm rewritten
two ways. Both asserted applied and compiled; tree restored byte-identical.

| what was patched into `diff()` | `metadata_matches_exiftool_on_every_corpus_file` |
|---|---|
| the doc comment's own simulation (`[1] → Some([1,1])`) | **FAILED** — `BlackLevelRepeatDim: ours=None, theirs=Some([1, 1])` |
| tri-state only — `Unparseable ≠ Absent`, `malformed_tags` **not** consulted | **FAILED** — `BlackLevelRepeatDim` named |
| tri-state **+ `malformed_tags` compared against it** — `FU-1`'s fix as specified | **ok. 21 passed** |

**Under `FU-1`'s stated fix the alarm never fires.** And it does not fire for a
good reason: once the comparator compares our `malformed_tags` against their
`Unparseable`, both sides agree the tag is malformed — which is the *generic*
guard `DEC-013`'s Option C described, implemented on the side that has the
information. The third row is not a bug; it is arguably the right answer.

**Why this is `FU` and not `SB-2`.** The handoff set the bar precisely: *"If it
does not red, the doc comment is false … that is an `SB-2`."* The doc comment's
claim as stated **does** red. And the removal survives all three futures:

- doc comment's sim → guard would have suppressed a real red. Removal wins.
- tri-state only → guard would have suppressed a real red. Removal wins.
- tri-state + `malformed_tags` → the comparison **replaces** the guard, generically
  and on the correct side. The guard would have been redundant. Removal is neutral.

Removal is weakly dominant across every shape `FU-1` can take, so **`SB-1`'s fix
is right on the merits regardless of `FU-8`.** The three counts that rejected
`DEC-013` each stand independently of this one too. What `FU-8` costs is the
*confidence of the recorded justification*, not the change.

**And it is the same defect class as the ship-blocker it was written to fix** —
a measurement of one case, recorded as a property of the category, in a record
whose stated purpose was correcting exactly that. That makes
`measurement-over-generalised` **N=5**, not `N=4`, and this instance is the
sharpest of the five because the author had just finished writing up the fourth.

**Disposition: `fixed`** — two paragraphs, no code. `diff()`'s doc comment and
`DEC-013`'s "Why the guard was REMOVED" section should state the fork: the alarm
fires for a tri-state that treats `Unparseable` as distinct from `Absent`, and
does **not** fire if the tri-state is compared against `malformed_tags`. That is
a better tripwire than the current one, because it tells the person fixing `FU-1`
what the actual decision is instead of promising them a red they may never see.
Fold the N=5 count into `FU-6`'s existing `signal: measurement-over-generalised`
disposition at ship rather than opening a second id for it.

---

### `FU-9` — a `rejected` decision still governs its `affected_scope`, because `is_active()` never reads `status`

`DEC-013` is the **first `rejected` decision in this repo** — the other twelve
are `accepted`, `proposed`, or `superseded`. `scripts/decisions-audit.sh:152-156`:

```sh
is_active() {
    local sb
    sb=$(get_top_scalar "$1" superseded_by)
    [ -z "$sb" ] || [ "$sb" = "null" ]
}
```

It tests `superseded_by` only. `DEC-006`/`DEC-007` are filtered out because they
carry one; `DEC-013` carries `superseded_by: null` with `status: rejected`, so it
is "active". Measured:

```
$ just decisions-audit --changed 418be15
⚠ DEC-013 — DEC-013: A tag already recorded in `malformed_tags` is exempt … — **REJECTED**
      re-read this decision before committing; your change touches:
        tests/support/tools.rs
```

and the footer tells the reader to *"confirm your change is consistent with each
decision above, or supersede the decision if it no longer holds"* — instructions
that are meaningless for a record that is already dead.

**⚠ Do not "fix" this by filtering rejected records out, and this is the useful
half of the finding.** Right now that surfacing is the *only* mechanical link
between the tripwire and its explanation. `FU-1`'s fix edits
`tests/support/tools.rs`; `decisions-audit --changed` then fires and points at
the record that explains why the guard is absent. The failure message itself
(`ours=None, theirs=Some([1, 1])`) names no decision and no doc comment. Filter
`rejected` out and the tripwire loses its signpost.

The architect's title edit — appending `— **REJECTED**` to the H1 — already does
most of the work, and it appears in the audit output verbatim. What is wrong is
only the *verb*: a rejected record should be surfaced as **"this decision was
rejected; read why before you re-derive it"**, not as something to be consistent
with.

**Disposition: `spec: SPEC-NNN`** — `scripts/decisions-audit.sh`, one helper and
one output string. It is outside `SPEC-005`'s scope and it is a repo-tooling
change with its own test surface, so it wants a spec rather than a drive-by.

---

### `FU-10` — `DEC-013` asserts the signal ledger reads `N=4`; the ledger reads `N=3`

`DEC-013`'s count 3 closes: *"The signal was already at its `N=3` bar; **it is
now `N=4`**."* `guidance/signals.yaml:210-219` still reads
`evidence: "N=3. (1) … (2) … (3) …"` with `last_touched: 2026-08-18`.

Procedurally, not updating it now is **correct** — `FU-6`'s disposition is
`signal: measurement-over-generalised` and §15 says follow-ups are dispositioned
at ship. The finding is the *tense*. The same commit edited `signals.yaml` (for
`tier-map-predicts-what-it-should-record`), so the author had the file open and
updated a different signal while writing "it is now N=4" about this one. A reader
who follows the citation finds N=3 and no mention of `SPEC-005`.

Small, and I am reporting it small. But it is a record stating that something has
happened which has not — a milder version of `SB-1`'s count 3.

**Disposition: `fixed`** — one line. Either soften `DEC-013` to "this is a fourth
instance; the ledger is updated at ship" or add the evidence now. `FU-8` pushes
the real count to **5**, so whichever is done should say 5, not 4.

---

### `FU-11` — `SPEC-005`'s verify cost session carries a figure its own note argues for, and which is now measured false

`cost.sessions[verify]` records `tokens_total: 8500000`, `estimated_usd: 19.30`,
with a note that says:

> tokens_total is rounded **UP** from the 8,409,731 measured mid-session … the
> true figure is ~8.5M and cannot be measured exactly from inside the session
> that is producing it.

**I reproduced the real number rather than inheriting the handoff's:**

```
ROUND-1 VERIFY TRANSCRIPT cabae9fc…
usage objects: 139  distinct message.id: 74
models: {'claude-opus-5': 139}
components: {input 148, output 67,351, cache-write 180,759, cache-read 9,955,612}
deduped total: 10,203,870
```

**10,203,870** — 20% above what was recorded, and per-component at Opus rates
**$23.38**, not `$19.30`. The reviewer rounded *up* believing that was the
conservative direction and was still 17% low; that is the systematic bias
`5b1aef7` correctly recorded in `signals.yaml`.

What is wrong is not the number — it is a floor and floors are honest. It is that
the **falsified justification stays in the spec with nothing pointing at the
correction.** The signals entry says *"Recorded, not retro-fixed: the reviewer's
number is theirs to report."* I agree with the principle and it does not reach
this: nobody is asking to overwrite the reviewer's self-report, only to stop the
spec asserting "the true figure is ~8.5M" when the repo has measured 10,203,870.
`cost.totals` also sums the low figure, and `just calibration` reads it.

**Disposition: `fixed`** — one appended sentence in that session's `notes`
pointing at `tier-map-predicts-what-it-should-record`'s floor-bias measurement.
Do not rewrite `tokens_total`.

---

## What I checked and did not turn into findings

**Scope creep — called, and benign.** The reviewed span is two commits, not one:
`80c2711` (`HANDOFF-022`, `HANDOFF-021`'s handback, `SPEC-005`'s build+verify
cost sessions, the `0-for-3` signal note) and `5b1aef7` (the fix). `5b1aef7`
itself touches exactly five files: `tests/support/tools.rs`, `DEC-013`,
`signals.yaml`, `HANDOFF-022`, `SPEC-005`. Of these:

- `tools.rs` + `DEC-013` — `SB-1`'s fix and its record. In scope.
- `HANDOFF-022` + `SPEC-005` — round 1's handback and cost session. In scope.
- `signals.yaml` — two changes. The `0-for-4 → 1-for-4` correction is the
  architect fixing a factual error they wrote in `80c2711` and round 1 corrected;
  that is not creep. The floor-bias paragraph **is** new work beyond `SB-1` — I
  am calling it because the handoff asked me to — but it is a measurement of the
  handback being processed, filed in the ledger built to accumulate exactly that,
  and it is honest against the architect's own interest. **No finding.**

⚠ Minor slip in `HANDOFF-023`'s own delta table, for the record: it lists
`HANDOFF-023` as part of `418be15..5b1aef7` (it is in `6d17c4d`) and omits
`HANDOFF-021`, which is in the span. Nothing turns on it.

**Dangling `DEC-013` references — none live.** ~50 hits across the repo, and all
but three are the **template's** `docs/decisions/DEC-013-delegated-cost-handback.md`,
a different file in a different namespace (§10). The three that mean this record:
`tests/support/tools.rs:301,315` (both say "now `rejected`") and `DEC-013` itself.
`SPEC-005`'s `references.decisions` is `[DEC-003, DEC-004, DEC-012]` — `DEC-013`
was never listed, so nothing there to stale. `CHANGELOG.md`'s oracle section
describes the three-way divergence assertion (`AC4.3`, still true) and never
mentioned the exemption. `status: rejected` is a valid value to
`decisions-audit` (`VALID_DEC_STATUS`), which is why it lints clean. The one
machine-visible residue is `FU-9`.

**Does `DEC-013` match what shipped, count 3 included?** Yes, and it did not
soften the part round 1 settled by test. It reproduces the decisive experiment —
*"a malformed `BlackLevelRepeatDim` diffs `[]`, while an **identically** malformed
`ActiveArea` still reds"* — and says explicitly that the reviewer settled it "with
the decisive test rather than by reading." Counts 1 and 2 are likewise stated as
measured, twice each, with the restoration discipline named. The original text is
preserved verbatim below a rule, under a heading that tells the reader not to act
on it. This is a good record.

**Is the removal behaviour-neutral today?** Yes. 87 across six targets, corpus
present, identical to round 1's 87 at `418be15`; the same 21 oracle tests by
name; clippy `-D warnings` clean, so the dropped import left nothing behind.
`grep -n 'malformed_tags' tests/support/tools.rs` now returns **one** line and it
is a doc comment — there is no exemption left in the comparator, which is what
the record claims.

**`§15` check 6 — reflection.** Round 1 raised `FU-7` because `HANDOFF-021` had
no build reflection. For *this* round there is no separate handoff: the fix was
made by the orchestrator under `DEC-004` rule 1, and `5b1aef7`'s commit message
is a genuine account of the three counts and the judgement, not a stub. Nothing
further to raise.

**`§15` check 7 — prior cycles' cost.** Present: design (null with a real note,
correct for an un-metered main-loop cycle), build (30,114,705), verify
(8,500,000 — see `FU-11`). `just cost-audit` exit 0.

---

## Discipline notes

- **Every mutation asserted applied and compiled** before any conclusion.
  `shasum -a 256` before and after each edit (`0b9e3a57…` → `1bb73d82…` →
  `1f00ca31…` → `cf2c633c…` → `ece05360…` → `00978272…`), `cargo build
  --all-features --tests` exit code checked each time — including the two that
  **failed to compile first** (`E0560`: I patched `Sensor`'s literal where
  `ToolReading`'s was meant), which is precisely why the assert-it-compiled rule
  exists and why neither near-miss became a conclusion.
- **Tree is pristine.** `git diff 5b1aef7 -- src/ tests/ Cargo.toml Cargo.lock`
  is empty; `tests/support/tools.rs` is back to
  `0b9e3a573dc08c9049727dc260b3233b9f8862913bc52e132ae5fdc2c0bc2d53`.
  `git status --porcelain` shows only the pre-existing untracked
  `reports/daily/2026-08-21.md`. No probe files left behind.
- **`handoff.to_agent` corrected, and checked rather than inherited** — 110/110
  usage objects in this session's transcript read `message.model:
  claude-opus-5`. The handoff was explicit that being right last round is not
  evidence, so I read my own. `tier_map.verify` is now **2 for 2**;
  `SPEC-007/FU-6` is **2 for 5** overall.
- **`tokens_total` deduped by `message.id`, and I say so** — 110 usage objects,
  47 distinct ids, raw 12,944,378 vs deduped 5,683,739 = **2.28×**, 96.8%
  cache-read. `estimated_usd` per-component at Opus rates. **It is a floor**, and
  the handoff's own ~17% correction says the closed-session figure will be
  ~6.6M / ~$16 — I am stating that rather than quietly booking the low number,
  which is `FU-11`'s whole lesson applied to myself.
- **`just handback-sync` NOT run**, per return criterion 7.

## Acceptance criteria — unchanged from round 1 except where `SB-1` touched them

| AC | verdict |
|---|---|
| AC1 — exiftool on all seven, naming file/field/ours/theirs | ✅ re-observed; ⚠ "all seven" still unasserted (`FU-3`, round 1) |
| AC2 — absence compared, not skipped | ✅ met as written (settled round 1) |
| AC3 — six unique dnglab scalars, count asserted | ✅ (settled round 1) |
| AC4.1 — `cropArea.p` arithmetic | ✅ |
| AC4.2 — PEF excluded by name and reason | ✅ |
| AC4.3 — malformed tag read three ways | ✅ **unaffected by the rejection**, as `DEC-013` claims — verified: the test still asserts all three, and it is now the only thing pinning the divergence |
| AC5 — goes red, with controls, tier A in CI | ✅ both watched failing by me, plus a third |
| AC6 — missing tool skips loudly, naming it | ✅; ⚠ `FU-3` (round 1) |
| AC7 — no new dependency | ✅ `git diff --stat 04aaf4b 5b1aef7 -- src/ Cargo.toml Cargo.lock fuzz/Cargo.toml` empty |
| AC8 — transcribed columns gone | ✅ in `ifd_reader.rs`; ⚠ `FU-5` (round 1, `closed`) |
| AC9 — ten gates + `oracle-meta` + fuzz | ✅ all green, 28.6M execs, seeds unchanged |

**`SB-1` is cleared and nothing replaced it.** `FU-8`…`FU-11` join round 1's
`FU-1`…`FU-7` for disposition at ship — eleven rows in `SPEC-005`'s
`## Follow-ups` table, and `FU-8` is the one to read first.
