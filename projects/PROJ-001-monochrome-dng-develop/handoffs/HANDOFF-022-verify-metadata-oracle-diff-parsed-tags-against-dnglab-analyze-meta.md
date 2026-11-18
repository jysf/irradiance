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
  id: HANDOFF-022
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ CONFIRMED to what ACTUALLY ran, measured from this
                                   #   verify session's own transcript
                                   #   (cabae9fc-893b-430e-9bb9-d03022543d90.jsonl):
                                   #   123/123 usage objects report message.model =
                                   #   claude-opus-5. tier_map.verify predicted opus and
                                   #   opus ran — so SPEC-007/FU-6 is 1 for 4 on this
                                   #   spec's cycles, not 0 for 4. The hint was right THIS
                                   #   time; it was still checked rather than assumed.
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
  tokens_total: 8500000            # REAL combined count — what cost-audit reads
  estimated_usd: 19.30             # tokens_total × your rate, or your harness's number
  duration_minutes: 21
  branch: feat/spec-005-metadata-oracle
  pr: null
  completed_at: 2026-08-22         # YYYY-MM-DD
  notes: "Verdict ⚠ PUNCH LIST. DEDUPED BY message.id and I say so: 123 usage objects, 65 distinct ids, raw 16,016,382 vs deduped 8,409,731 = 1.90x, 97.5% cache-read. Deduped components: input 130, output 49,334, cache-write 161,965, cache-read 8,198,302. estimated_usd computed PER-COMPONENT at published OPUS rates ($15/$75/$18.75/$1.50 per M) because message.model reads claude-opus-5 on all 123 objects — checked, not inherited from tier_map. tokens_total is rounded UP from the 8,409,731 measured mid-session because the transcript is still being written as this block is authored (the handback prose and the final gate re-run are not yet in it); the true figure is ~8.5M and cannot be measured exactly from inside the session that is producing it. Four handoff findings confirmed (F-1/F-2/F-3 merged as SB-1, F-4 as FU-1), F-5 confirmed and BROADER than stated (FU-3), both handoff-suggested sixth findings KILLED, three new findings raised (FU-2/FU-4/FU-5). Every mutation asserted applied AND compiled; tree restored byte-identical (shasum checked each time) and `git diff 418be15 -- src/ tests/ Cargo.toml Cargo.lock` is empty."
  synced_at: 2026-08-22
---

# HANDOFF-022: Verify the metadata oracle — SPEC-005 at `418be15`

## Delegation Summary

Verify `SPEC-005` at **`418be15`** on branch **`feat/spec-005-metadata-oracle`**
(not merged; `main` is at `04aaf4b`). Nine acceptance criteria, nine named tests,
87 tests total across six targets, `src/` untouched.

**This handoff carries FOUR findings the orchestrator surfaced during
reconciliation, and one process irregularity. They are not conclusions — they are
required checks. Confirm or kill each one yourself.** Where I state a
measurement, reproduce it; where I state a reading of the code, read it. If you
think I am wrong, say so with the command that shows it — that has happened
before here and it was the right outcome both times.

## ⚠ Process irregularity, disclosed up front

The build session **reported done but did not finish the cycle**: it left
`HANDOFF-021`'s `handback:` block entirely null, did not branch, did not commit,
and asked the orchestrator to run `/cost` — a client-side command the assistant
cannot execute, and which would have measured the *orchestrator's* session, not
the build's.

The orchestrator therefore finished the **mechanical remainder** per `DEC-004`
rule 1: branched, committed, recovered the real token figure from the build
session's own transcript, and filled the handback. **The code is entirely the
build's; the commit and the handback are not.** Weigh that when you judge
`§15` check 6 (implementer reflection) — there is no build reflection to read,
which is itself worth a finding.

## Context the Receiving Agent Needs

Read `SPEC-005` in full (its `## Implementation Context` is a measured probe, not
background), `AGENTS.md` §12 and §15, `guidance/constraints.yaml`,
`decisions/DEC-012` and the **new** `decisions/DEC-013` — note that
`docs/decisions/DEC-013` is a *different file in the template's namespace*
(§10). Corpus: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.

## What the orchestrator already re-ran (reproduce, do not assume)

- `cargo fmt --check`, `clippy --all-targets --all-features -D warnings`,
  `just msrv`, `just deny`, `just deny-fuzz`, `just lint-red-proof`,
  `just lint-no-allow`, `just cost-audit`, `just validate` — **all exit 0**.
- `cargo test --all-features` → **87 passed, summed across all six targets**
  (`45+0+9+12+21+0`), corpus present.
- `git diff --stat` on `src/`, `Cargo.toml`, `Cargo.lock` — **empty**.
- Both red-proofs **read line by line** and judged genuine: tier A perturbs
  `bits_per_sample`→13 and asserts *exactly one* mismatch named `BitsPerSample`
  against an empty-diff control; tier B XORs `ActiveArea`'s payload in an
  in-memory copy, **asserts the patch changed the buffer**, asserts one mismatch
  named `ActiveArea`, then re-diffs unpatched as its control. Nothing is written
  to disk.

## THE FOUR FINDINGS — confirm or kill each

### F-1 — `diff()`'s `malformed_tags` guard is dead code

Measured by the orchestrator: comment out the `!sensor.malformed_tags.contains(...)`
condition at `tests/support/tools.rs:350` and **all 21 oracle tests stay green**
(mutation asserted applied by `diff`, tree restored byte-identical). Reproduce it.
If nothing dies when a guard is removed, the guard is not guarding.

### F-2 — `DEC-013`'s premise appears to be false

`DEC-013` says `K3III.DNG` "would fail `AC1`'s own test on `BlackLevelRepeatDim`
forever". Check whether it would. `exiftool` reports a bare `1` for that tag;
`tools.rs:247-248` runs `<[u32;2]>::try_from(v.as_slice()).ok()`, which on a
one-element vector yields **`None`**; our reader also reports `None`
(`DEC-012` drops the value). `None == None` — no mismatch, so the permanent red
the decision exists to prevent may never have been possible. **This is the
finding I am least certain of** — a decision record is a serious artifact and I
would rather be corrected than have it quietly stand. Verify it directly.

### F-3 — `DEC-013` records choosing Option C and appears to ship Option B

Option C is quoted in the record as *"`diff()` reads `malformed_tags` and skips
exempted fields **generically** … no per-file knowledge in the comparator at all
… needs no update when a FUTURE file exercises a different malformed tag."*
`tools.rs:350` reads `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM)`
— one hardcoded tag. `grep -n 'malformed_tags' tests/support/tools.rs` returns
exactly one code site. If that reading holds, this is the same shape as
`SPEC-008/FU-4` and would be the **fourth** instance of
`measurement-over-generalised`, which is already at its bar.

### F-4 — the oracle may not distinguish "absent" from "unparseable"

`ActiveArea`, `DefaultCropOrigin` and `DefaultCropSize` all parse via
`values_for(...).and_then(|v| match v.as_slice() { [..] => Some(..), _ => None })`.
A **garbled** tool reading and an **absent** tag both become `None`, so a garbled
reading silently *agrees* with a `None` on our side. `AC2` exists for exactly
this: *"An oracle that ignores `None` cannot catch a reader that invents values."*
Judge whether `AC2` is met as written. **The spec did not anticipate this and
neither did I** — if this is real it is a design gap as much as a build one, and
should be labelled that way.

### F-5 — three tier-B tests compute a coverage counter and never assert it

`tests/metadata_oracle.rs:94`, `:137`, `:194` each maintain `checked` and report
it via `eprintln!`, which `cargo test` swallows without `--nocapture`
(measured in `SPEC-002/F2`). Measured: with `IRRADIANCE_CORPUS_DIR` pointing at a
nonexistent path, `metadata_matches_exiftool_on_every_corpus_file` **passes
having checked zero files**. `AC1` says "on all seven files". This is
`named-tests-can-pass-vacuously` (an `accepted` signal) occurring inside the
spec whose whole job is to stop things passing vacuously.

## Your own checks — do NOT limit yourself to my list

The most valuable thing you can do is find a **sixth**. Two suggestions, both in
this repo's grain:

1. **Is `diff()` narrower than it looks?** It has eleven explicit comparisons.
   Perturb each of the eleven fields in turn on the tier-A fixture and confirm
   each one produces a mismatch. A field that is compared but whose *reading*
   side is never populated compares `None` to `None` forever. This is
   `SPEC-008/FU-1`'s shape one level up.
2. **Is the dnglab uniqueness assertion real?** The build claims each key's match
   count is asserted unique before use. Plant a duplicate key and watch it
   refuse, per `attribute-text-inside-doc-comments`' general form.

## Return Criteria

1. Ten gates re-run **by you** and pasted. Sum test counts **across all six
   targets** — a zero-match `cargo test <name>` exits 0.
2. **Watch both red-proofs fail yourself** (§15 check 9). A red you did not
   personally observe failing is a self-report.
3. **Fuzz (§15 check 10)** — the build claims 13,455,965 execs, seeds unchanged.
   Re-run it: `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd`.
4. Confirm each of the nine named tests **exists** per-target via `-- --list`
   before trusting any green.
5. Every mutation you run: **assert it changed the file, and assert it compiled,**
   before drawing a conclusion. Five failures here, historically.
6. Fill the `handback:` block including a **real `tokens_total`, deduped by
   `message.id`**, and say you deduped. You *can* get this — read your own
   transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`; the session id is
   in the scratchpad path in your system prompt. Compute `estimated_usd`
   **per-component at the rates for the model that actually ran** — check
   `message.model`, do not trust `tier_map`. On this spec's build, Opus rates
   would have overstated by **5.0×** and the repo's flat `rate_per_mtok` by
   **14.7×**.
7. **Correct `handoff.to_agent`** to what actually ran.
8. Do **not** run `just handback-sync` — the orchestrator runs it.
9. Label findings `SB-N` / `FU-N` for **this** spec from 1. Each will be
   dispositioned at ship into `fixed` / a spec / a signal / an explicit close
   (§15, *Where an unresolved follow-up goes*) — say which of the four you think
   each wants.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

**Verdict: ⚠ PUNCH LIST** — `418be15` on `feat/spec-005-metadata-oracle`.

The oracle is real. All eleven comparisons are live from **both** sides, the
dnglab uniqueness discipline holds against planted duplicates, both red-proofs
fail when their detection is removed, and `src/` is untouched. What holds it
back is one artifact: **`DEC-013` is wrong on three counts and the code it
governs is dead**. That is `SB-1`, and it is the only ship-blocker.

### The ten gates, re-run by me

`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`

| gate | result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `just msrv` | exit 0 (1.90.0, whole target set) |
| `just deny` | exit 0 — `licenses ok` |
| `just deny-fuzz` | exit 0 — `licenses ok` |
| `just lint-red-proof` | exit 0 — control clean → injection rejected (101) → all five lints fired |
| `just lint-no-allow` | exit 0 |
| `just cost-audit` | exit 0 |
| `just validate` | exit 0 — 9 artifacts |
| `cargo test --all-features` | **87 passed**, summed across **six** targets |

Test counts summed per target, not read off one: `45 + 0 + 9 + 12 + 21 + 0 = 87`
(lib · `irr` bin · `corpus_manifest` · `ifd_reader` · `metadata_oracle` ·
doc-tests). `just oracle-meta` also green — 21 passed.

**Return criterion 4 — the nine named tests exist.** Confirmed per-target via
`cargo test --all-features --test metadata_oracle -- --list`: 21 tests, of which
all nine spec-named tests are present by exact name. No zero-match green.

**Return criterion 3 — fuzz (§15 check 10).**
`PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd -- -max_total_time=300`
→ **51,162,034 runs in 301 s**, ~170k exec/s, `cov: 661 ft: 2172`, exit 0, **no
crashes** (`fuzz/artifacts/ifd/` empty). Seeds unchanged — `find fuzz/seeds -type f | sort | xargs shasum | shasum`
reads `6a5b9fb7…` before and after. Seeds-unchanged is the expected result and I
am saying so, per AC9.

**§15 check 3** — `just decisions-audit --changed`: no active decision's
`affected_scope` matches. `just decisions-audit`: 0 structural errors, 2 pre-existing
`DEC-000` scope warnings unrelated to this spec.

### Return criterion 2 — I watched both red-proofs fail

Not "the red-proof passes". I removed the detection each one depends on and
confirmed the proof itself dies, with the control staying green so the failure is
attributable. Every mutation: **asserted applied** (shasum before ≠ after),
**asserted compiled** (`cargo build --all-features --tests` exit 0), restored
byte-identical.

| red-proof | mutation | result | control |
|---|---|---|---|
| `oracle_names_the_one_field_that_was_perturbed` (tier A) | `if false && sensor.bits_per_sample != …` | **FAILED** — `exactly one field must disagree, got []` | `oracle_is_clean_on_an_unmodified_reading` still ok |
| `oracle_goes_red_on_a_patched_tag_in_a_real_file` (tier B) | `if false && sensor.active_area != …` | **FAILED** — `patching ActiveArea must produce exactly one mismatch, got []` | still ok |

I also watched **AC1's own message** go red, by perturbing `black_level` inside
the AC1 loop on the real corpus:

```
LEICA-Q2-MONO/L1021223.DNG: 1 field(s) disagree with exiftool:
  BlackLevel: ours=Some(519), theirs=Some(512)
```

File, field, ours, theirs — AC1's "Not 'mismatch'" is met literally.

---

## Findings

### `SB-1` — `DEC-013` is wrong on three counts, and the guard it governs is dead

Merges the handoff's `F-1`, `F-2` and `F-3`. They are one artifact and one fix,
so they get one id.

**(a) The guard is dead code (`F-1` confirmed).** Removed the
`!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM)` condition at
`tests/support/tools.rs:350`, leaving the bare value comparison. Mutation asserted
applied (`9122a8cb…` → `cfa8f07d…`), compiled (one unused-import warning, no
error), and **all 21 oracle tests stayed green with the corpus present**.
Restored byte-identical.

**(b) `DEC-013`'s premise is false (`F-2` confirmed — you were right to doubt it).**
Measured on the real file:

```
exiftool RAW values_for  = Some([1])      <- the bare 1 the record describes
ToolReading (theirs)     = None           <- reading_from_fields collapses it
Sensor      (ours)       = None           <- DEC-012 drops it
malformed_tags           = [50713]
diff()                   = []
```

`tools.rs:247-248` runs `<[u32;2]>::try_from(v.as_slice()).ok()` on a
one-element vector, which is `None`. `None == None`. The "permanent, expected
red" `DEC-013` exists to prevent **cannot occur**, with or without the guard.

⚠ **But note *why* it cannot occur, because it changes the fix.** The premise is
false only because the tool side silently degrades a shape-odd reading to
"absent" — which is `FU-1`. Fix `FU-1` and the guard becomes live and necessary.
So `DEC-013`'s *conclusion* may well be right; its *stated premise* is not, and
its guard is currently load-bearing for nothing.

**(c) The record says Option C; the code ships something narrower (`F-3` confirmed).**
`grep -n 'malformed_tags' tests/support/tools.rs` returns exactly one code site
(line 350) plus two doc-comment lines. Option C is quoted in the record as
*"no per-file knowledge in the comparator at all … needs no update when a FUTURE
file exercises a different malformed tag."* I tested that property directly with
a different malformed tag:

```
BlackLevelRepeatDim malformed (the ONE hardcoded tag) -> []
ActiveArea          malformed (a FUTURE file's tag)   -> ["ActiveArea"]
```

Option C's stated property **does not hold**. A future file with a malformed
`ActiveArea` produces exactly the unexplained permanent red the record was
written to prevent — while the record tells its reader that cannot happen. It
reads `malformed_tags` (Option C's mechanism) but gates on one hardcoded tag
(Option B's specificity), and is recorded as neither.

**Disposition: `fixed`** — correct or supersede `DEC-013` in this spec's
punch-list round. The code change may be as small as none; the record must not
ship as written. Also raises `FU-6`.

---

### `FU-1` — the oracle cannot tell "the tool said something unparseable" from "the tag is absent"

The handoff's `F-4`, confirmed, and it is a **design gap** as much as a build one
— the spec did not anticipate it. Measured: five garbled tool readings, each
diffed against a Sensor whose optional tags are all `None`:

| tool reading | parsed as | diff |
|---|---|---|
| control — all `-` | `None` | `[]` |
| `BlackLevelRepeatDim="1"` | `None` | `[]` |
| `ActiveArea="0 0 5632"` | `None` | `[]` |
| `ActiveArea="0 0 5632 8392 9"` | `None` | `[]` |
| `DefaultCropOrigin="12"` | `None` | `[]` |
| `DefaultCropSize="8368 5584 9"` | `None` | `[]` |

**5/5 garbled readings are indistinguishable from a genuine `-`.**

The sharp form of it: **our side keeps this distinction and the oracle throws it
away.** `src/ifd.rs`'s `array::<N>()` (`DEC-012`) records a wrong-length tag in
`malformed_tags`; `ToolReading` has no equivalent field, so the comparator has
nothing to compare it against — which is precisely why `SB-1`'s guard is dead.

**Is `AC2` met as written? Yes — I am killing that half of the framing.** `AC2`
asks that a `-` read `None` on our side *and vice versa*, and that the oracle
catch a reader inventing values. Both hold: I perturbed our side to `Some(x)`
against a tool `None` and got a named mismatch every time (see the sweep below).
The gap `FU-1` names is the *converse* — a shape-odd tool value reclassified as
absence — and that is outside `AC2`'s wording. It is real; it is not an `AC2`
failure.

**Disposition: `spec: SPEC-NNN`** — one file, one fix (a tri-state on the tool
side: `Absent` / `Unparseable(raw)` / `Value`), plus comparing `malformed_tags`
against it. Take `FU-2` into the same spec.

---

### `FU-2` — a multi-valued tool reading is silently truncated to its first element

New, mine. `reading_from_fields`' `req()`/`opt()` helpers are
`values_for(..).and_then(|v| v.first().copied())` — everything after the first
value is discarded without a word:

```
bits="14 12 8" black="512 999" white="16383 1" orient="1 6"
  -> bits=14 black=Some(512) white=Some(16383) orient=Some(1)
  -> diff vs an honest single-valued Sensor = []
```

Different mechanism from `FU-1` (truncation, not conflation) and a different fix
(assert the length), so it gets its own id. **Latent, not live:** every tag on
today's seven-file monochrome corpus is single-valued, so this hides nothing
today. It goes live the moment a file has `SamplesPerPixel > 1` or
`BlackLevelRepeatDim ≠ [1,1]` — at which point `BlackLevel` is legally a 4-element
array and the oracle would report agreement from its head alone.

**Disposition: `spec: SPEC-NNN`** — same spec as `FU-1`.

---

### `FU-3` — seven corpus-gated tests, including a red-proof, pass having checked nothing

The handoff's `F-5`, confirmed and **broader than stated**. With
`IRRADIANCE_CORPUS_DIR=/nonexistent/…`:

```
test result: ok. 21 passed; 0 failed; ... finished in 0.06s     <- vs 1.09s with corpus
```

Not three tests — **all seven corpus-gated ones**, including
`oracle_goes_red_on_a_patched_tag_in_a_real_file`, one of the two proofs the
blocking constraint `oracle-must-be-shown-red` turns on. It reports green having
never patched a byte.

Two things are true and both matter:

- **The skip IS loud** and names every missing file, satisfying §12 bar 4 —
  `SKIP LEICA-Q2-MONO/L1021223.DNG — MISSING at …`. `CorpusFile::require` is
  doing its job.
- **It is loud on a channel `cargo test` discards.** I only see those lines under
  `--nocapture`. The default run says `21 passed` and nothing else. That is
  `SPEC-002/F2`'s measurement biting again, one spec later.

The build's own addition is the counter: `checked` is computed at
`tests/metadata_oracle.rs:94`, `:137`, `:194` and reported via `eprintln!`, never
asserted. `AC1` says "on all seven files"; nothing asserts seven. This is
`named-tests-can-pass-vacuously` occurring inside the spec whose whole job is to
stop things passing vacuously — which is the strongest possible argument for the
check that signal is already waiting on.

**Disposition: `signal: named-tests-can-pass-vacuously`** — add evidence. It is
`status: accepted` awaiting a real check, and the note there already names both
traps (sum across targets, run with the corpus). The cheap fix when someone
writes it: assert `checked == manifest.files.len()` whenever the corpus root
resolves to a directory that exists.

---

### `FU-4` — the tier-A fixture pair is never reconciled with reality

New, mine, and it is the spec's own thesis turned on the spec's own artifact.

`## Context` indicts the hand-transcribed table because *"it cannot notice that
our reader drifted, that a tool changed its answer, or that the transcription was
wrong on the day."* Tier A is two frozen literals — `FIXTURE_LINE` (66 committed
bytes) and `fixture_sensor()` — with exactly those three blind spots. Nothing
compares either to the real thing, **even on a machine that has the corpus and
both tools**.

I checked, since nothing else does. **Both halves are accurate today:**

```
(a) FIXTURE_LINE vs live exiftool 13.55 : MATCH
(b) fixture_sensor() vs the real reader : 0/13 fields diverged
```

So this is a rot risk, not a present defect — I am reporting it as the former, not
dressing it up as the latter. But it means the only half of the oracle CI can run
cannot detect reader drift at all, and the fix is one gated test.

**Disposition: `fixed`** — add a corpus+tools-gated test asserting
`FIXTURE_LINE == live exiftool` and `fixture_sensor()`'s eleven fields ==
`sensor_at(L1021223)`. Cheap, and it closes the loop the spec asked for.

---

### `FU-5` — `AC8`'s letter: eleven expected tag values are hand-typed outside `tests/oracle-fixtures/`

`AC8` says *"no expected tag value is a hand-typed literal anywhere outside
`tests/oracle-fixtures/`."* `fixture_sensor()` (`tests/metadata_oracle.rs:358-388`)
hand-types all eleven — `8424`, `5632`, `14`, `34892`, `512`, `16383`, `[1,1]`,
`{0,0,5632,8392}`, `{12,24}`, `{8368,5584}`, `1` — in `tests/metadata_oracle.rs`.

`tests/ifd_reader.rs` itself is **clean**: its `Expected` struct now carries only
`path`, `big_endian`, `ifds`, `sensor_index`, `opcode_lists`, `malformed` — exactly
the five structure columns the spec said to keep. `AC8`'s substance is met; its
letter is not, because the debt moved rather than vanished.

**Disposition: `closed`** — reason: tier A must run with **no corpus and no tool**,
so its "ours" side has no source but a literal; relocating it into
`tests/oracle-fixtures/` would satisfy the wording while changing nothing real.
The honest answer to the risk is `FU-4`'s reconciliation test. Recommend amending
`AC8`'s wording at ship rather than moving code.

---

### `FU-6` — `measurement-over-generalised` reaches N=4

`SB-1(c)` is a fourth instance of a signal that is **already at its bar**
(`bar: N=3 same-outcome`, `status: watch`, *"AT BAR (N=3) as of 2026-08-21 —
codify at the owning stage's close"*). The shape matches exactly: the comparator
was exercised against the one malformed tag the corpus happens to carry, and the
decision record generalised that to *"needs no update when a FUTURE file
exercises a different malformed tag."* Running something proved what was run, and
the record claimed the category.

**Disposition: `signal: measurement-over-generalised`** — add evidence (4).

---

### `FU-7` — there is no build reflection to check (§15 check 6)

Confirmed as the orchestrator disclosed. `HANDOFF-021`'s `## Handback` prose
section is the untouched template stub —

> *(Filled by the implementer. Mirror the `handback:` front-matter above, and
> answer the reflection questions in `AGENTS.md` §15 "When done".)*

— while `handoff.status` reads `completed`. §15 check 6 cannot be run: there is
nothing to judge as mailed-in or not. This is the same shape as `SPEC-007/FU-7`,
which is already evidence (2) in `cost-field-has-two-owners`: a completed status
sitting on top of an unfilled handback.

**Disposition: `signal: cost-field-has-two-owners`** — add evidence (3), noting
that the reflection half is broader than the cost field the signal is named for.
The code is entirely the build's; the commit, the handback and the reflection gap
are not, and `SB-1` is not chargeable to that gap.

---

## What I killed

The handoff asked for a sixth finding and offered two candidates. **Both are
wrong, and the build deserves the credit.**

**Suggestion 1 — "is `diff()` narrower than it looks?" KILLED.** I perturbed
every one of the eleven comparisons in turn, **from both sides** — 19 cases
perturbing `Sensor`, 12 perturbing `ToolReading`, including `Some→None` and
`None→Some` transitions on every optional field. Each perturbation was asserted
to have changed the struct. **All 31 produced exactly one mismatch, correctly
named.** There is no field that is compared but never populated; nothing compares
`None` to `None` forever. `diff()` is exactly as wide as it looks.

**Suggestion 2 — "is the dnglab uniqueness assertion real?" KILLED.** Against the
real 2709-byte `dnglab analyze --meta --json` output for `L1021223.DNG`: all six
keys verified to occur exactly once, then each duplicated in turn (plant asserted
to have changed the document and raised the count 1→2). **All six refused**, each
with a named error:

```
"rawWidth" occurs 2 time(s) in the JSON document (expected exactly 1) —
 the uniqueness this extraction depends on no longer holds
```

The build's claim holds exactly as written.

**And half of `F-4`'s framing — `AC2` is met as written.** See `FU-1`.

My sixth findings came from elsewhere: `FU-2` (truncation) and `FU-4` (the
fixture never reconciled).

---

## Discipline notes

- **Every mutation asserted applied and compiled** before any conclusion —
  five failures here historically, none this session. `shasum` before/after on
  each edit, `cargo build --all-features --tests` exit code checked, tree
  restored and re-hashed each time.
- **Tree is pristine.** `git diff 418be15 -- src/ tests/ Cargo.toml Cargo.lock`
  is empty. `git status --porcelain` shows only the pre-existing untracked
  `reports/daily/2026-08-21.md`, which the build commit deliberately left alone.
  All probe files deleted.
- **`AC7` verified structurally**: `git diff --stat 04aaf4b 418be15 -- src/
  Cargo.toml Cargo.lock fuzz/Cargo.toml` is empty. No new dependency, and
  `src/` was never touched — `library-not-application` and the spec's "every
  line lives under `tests/`" both hold.
- **`handoff.to_agent` corrected** per return criterion 7 — and it was already
  right. 123/123 usage objects read `message.model: claude-opus-5`. tier_map
  predicted opus for verify and opus ran, so `SPEC-007/FU-6` is **1 for 4** on
  this spec's cycles, not 0 for 4. Worth recording that the hint's first
  correct call is still a checked call.
- **`just handback-sync` NOT run**, per return criterion 8.

## Acceptance criteria

| AC | verdict |
|---|---|
| AC1 — exiftool on all seven, naming file/field/ours/theirs | ✅ naming verified by observation; ⚠ "all seven" not asserted (`FU-3`) |
| AC2 — absence compared, not skipped | ✅ met as written (`FU-1` is outside its wording) |
| AC3 — six unique dnglab scalars, count asserted | ✅ verified against planted duplicates |
| AC4.1 — `cropArea.p` arithmetic | ✅ |
| AC4.2 — PEF excluded by name and reason | ✅ |
| AC4.3 — malformed tag read three ways | ✅ all three asserted |
| AC5 — goes red, with controls, tier A in CI | ✅ both watched failing; ⚠ tier-B half vacuous without corpus (`FU-3`) |
| AC6 — missing tool skips loudly, naming it | ✅ `run_tool` classifies `NotFound`; ⚠ the skip *message* rides the swallowed stream (`FU-3`) |
| AC7 — no new dependency | ✅ byte-identical |
| AC8 — transcribed columns gone | ✅ in `ifd_reader.rs`; ⚠ letter violated in `metadata_oracle.rs` (`FU-5`) |
| AC9 — ten gates + `oracle-meta` + fuzz | ✅ all green, 51.2M execs, seeds unchanged |

**To clear the punch list:** fix `SB-1` — correct or supersede `DEC-013` so the
record matches what shipped, and decide there whether the guard stays (dead until
`FU-1` lands) or goes. Everything else is dispositioned above and none of it
holds the spec.

