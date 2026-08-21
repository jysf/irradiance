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
  id: HANDOFF-016
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-004

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
  tokens_total: 5991740            # REAL combined count — what cost-audit reads
  estimated_usd: 39.55             # tokens_total × your rate, or your harness's number
  duration_minutes: 45
  branch: feat/spec-004-tag-model
  pr: null
  completed_at: 2026-08-21         # YYYY-MM-DD
  notes: "VERDICT: APPROVED at 37204d0 — six follow-ups, NO ship-blockers. tokens_total is a transcript sum DEDUPED BY message.id and says so, matching HANDOFF-011/012/013/014's methodology: 124 usage objects, 49 distinct ids, raw 15,050,048 vs deduped 5,991,740 = 2.51x, 97.0% cache-read. It is a FLOOR — computed before the session closed. 2.51x is the EIGHTH measured factor and a NEW HIGH: the band was 1.61x/1.76x/1.82x/1.86x/1.95x/2.20x/2.25x and is now 1.61x-2.51x, a 1.56x spread, which strengthens rather than weakens the standing rule that no fixed correction may be applied to any raw figure. NOTE FOR FU-18: HANDOFF-015's tokens_total: null was AVOIDABLE. This session ran in the same top-level interactive mode and obtained a real number by reading its own transcript at ~/.claude/projects/<path-slug>/<session-id>.jsonl — the session id is discoverable from the scratchpad path the harness provides. The build's reasoning was honest and DEC-013-compliant (never invent a number) but its premise that no source existed was wrong."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-016: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-004` for the **verify** cycle, at
`37204d0`. Independent session.

⚠ **Read the first scrutiny item before anything else — the spec you are verifying
against was substantially wrong, and the build was right to deviate.**

## Context the Receiving Agent Needs

### ⚠ My spec's Goal was mostly already shipped, and the build caught it

`SPEC-004`'s Context claimed SPEC-003 "stops exactly where geometry begins."
**False.** `main` already carried `black_level`, `white_level`, `active_area`,
`orientation`, `opcode_lists` and `black_level_repeat_dim` as `Option<…>` fields
(`src/ifd.rs:442-460`), extracted via `scalar()`/`array()` with a `malformed`
accumulator — which also already satisfied my AC3 (absent ≠ zero) and part of
`DEC-012`.

I reached that conclusion by grepping `pub struct Sensor` and reading only its
first lines. The struct continued past where I stopped. **I did this in the very
handoff that warned the builder not to assert from an incomplete look.**

So the real remaining work was narrower than the spec says, and the build scoped
it correctly to three things. **Verify the deviation, not the spec's literal
wording** — and judge whether the narrowed scope is genuinely complete.

### Already reconciled by the orchestrator

- **All ten gates green.** `main` untouched; branch is one commit ahead; tree clean.
- **All five literally-named tests exist and pass** (`--list` confirms the names).
  ⚠ My own first check reported "0 passed" for four of them because I took the
  first target's line; they live in a different target. Sum across targets or use
  `--list`.
- **AC1 done:** `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize` are now
  named-field structs (`src/ifd.rs:421/435/445`), not bare `[u32; N]`.
- **FU-11 closed with a tri-state:** `SensorMatch { Yes | No | Unreadable(tag) }`
  (`src/ifd.rs:579-587`), and `Error::NoSensorIfdCandidatesMalformed` names what
  was unreadable instead of a bare `NoSensorIfd`.

### What deserves scrutiny

1. **Is the narrowed scope complete?** Given the spec's Goal was largely already
   met, did the build correctly identify what remained — or is something in the
   original list genuinely still missing? Check the tag list against
   `docs/measured-q2m-dng.md` yourself.
2. **The FU-11 tri-state is the substance.** `Unreadable(tag)` must produce
   *different, asserted* outcomes for a malformed tag on a **non-sensor** IFD
   versus on the **sensor** IFD. The design warned that silently skipping is wrong
   because it hides a real plane. Does `NoSensorIfdCandidatesMalformed` actually
   say *which* IFD and *which* tag, and is it reachable?
3. **`tokens_total: null`, with a written reason** — this ran as a top-level
   session with no `/cost` and no usage-object access. That is the correct
   behaviour under `DEC-013` (never invent a number), and it differs from earlier
   CLI sessions that could sum transcripts. Confirm the reason is recorded in both
   the handoff and the spec, and consider whether
   `.repo-context.yaml`'s `cost.metering_source` should now say `none` for this
   execution mode rather than leaving the gate asking.
4. **The build's own sharp observation:** `cargo test <name>` matching **zero**
   tests exits **0**. A spec that names tests therefore creates a silent-pass
   hazard if the names drift. Worth a signal, or a check?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the two malformed-tag fixtures yourself and confirm the outcomes genuinely
differ — that pair is the spec. ⚠ Traps: `cargo +1.90.0` fails (use `just msrv`);
`cargo fuzz` needs the rustup shim first on PATH; an unchecked index will not reach
the fuzzer (the lint policy rejects it at compile time).

**Label every finding ship-blocking or follow-up.** If APPROVED, set
`task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Running `handback-sync` (finding 15).

## Return Criteria — how to hand back

1. Paste the ten gates and the two malformed-tag fixtures.
2. Fill `## Completion` and `handback:`. `tokens_total`: deduplicate by
   `message.id` and say so, **or** `null` with a written reason — never a guess.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship` if approved).
4. Commit on `feat/spec-004-tag-model`; do not merge.

## Verdict

✅ **APPROVED at `37204d0`** (reviewed at HEAD `08fe18f`). **Six follow-ups, no
ship-blockers.** `task.cycle` set to `ship`.

The build's deviation is the right call and the narrowed scope is complete
against the spec's own acceptance criteria. I checked the tag list against
`docs/measured-q2m-dng.md` line by line myself: every tag that document records —
`PhotometricInterpretation`, `SamplesPerPixel`, `Compression`, `BitsPerSample`,
`BlackLevel`/`WhiteLevel`, `ImageWidth`/`ImageLength`, `RowsPerStrip`,
`StripByteCounts`, `ActiveArea`, `DefaultCropOrigin`/`Size`, `OpcodeList1`/`3`,
`Orientation` — is extracted and pinned, plus `StripOffsets`, `NewSubfileType`,
`OpcodeList2` and `BlackLevelRepeatDim` beyond it. The three-stage geometry the
document names (`ActiveArea` → `DefaultCrop*` → `Orientation`) is complete. Nothing
in the original list is missing.

## Ten gates — all re-run by me, 2026-08-21, `IRRADIANCE_CORPUS_DIR` on the real 7-file corpus

1. **build** — `cargo build --release` → exit 0.
2. **test** — `cargo test --all-features` → **52 passed, 0 failed**, summed across
   all five targets: **31** lib + **0** `src/bin/irr.rs` + **9** `corpus_manifest.rs`
   + **12** `ifd_reader.rs` + **0** doc-tests. ⚠ The handoff's warning is real and I
   hit the inverse of it: my first run was piped through `tail -40`, which cut the
   lib target's 31 off the top. Read every `Running` line or the number is wrong.
3. **lint (clippy)** — `cargo clippy --all-targets --all-features -- -D warnings` → exit 0, clean.
3b. **lint (fmt)** — `cargo fmt --check` → clean.
4. **typecheck** — `cargo check --all-targets --all-features` → clean.
5. **licences (lib)** — `cargo deny check licenses` → `licenses ok`; only the
   pre-existing `license-not-encountered` warnings (BSD-3-Clause/Zlib/Unicode-3.0).
6. **licences (fuzz)** — `cargo deny --manifest-path fuzz/Cargo.toml check licenses` → `licenses ok`.
7. **msrv** — `just msrv` → exit 0 against the pinned 1.90.0 floor. (Trap confirmed
   as documented: the recipe is required; bare `cargo +1.90.0` does not resolve.)
8. **lint-red-proof** — `just lint-red-proof` → exit 0: control clean → injection
   rejected (exit 101) → all five lints fired at 4 distinct injected lines in
   `src/lib.rs:56-81`, and still fire without CI's `-D warnings`.
9. **lint-no-allow** — `just lint-no-allow` → exit 0, no `#[allow]` escape.
10. **fuzz** — `PATH="$HOME/.cargo/bin:$PATH" cargo +nightly fuzz run ifd
    fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60` → **15,649,000 runs in
    61 s**, `Done`, zero artifacts. A second clean run after my red-proof gave
    **10,365,858 runs in 46 s**, zero artifacts.

Also green: `just validate` (6 artifacts), `just cost-audit`, `just decisions-audit
--changed main` (names DEC-008 and DEC-012 on `src/ifd.rs` — the deferral wired at
DEC-012's `affected_scope` fires as designed), `just decisions-index --check`.
`cargo run --example fuzz-seeds` regenerates the 25 seeds **byte-identically**
(`git status` clean afterwards). `main` is untouched: `main` is still at `00e0472`
and the branch is two commits ahead.

## Deliverable 3 — the two malformed-tag fixtures, run by me

Not just as tests: I ran both through the shipped `irr ifd` binary, so the
difference is visible at the behavioural surface and not only inside an assertion
(AGENTS.md §12 behavioural pre-flight). **Same malformed tag — `PhotometricInterpretation`
forced to TIFF field type 250 — on different IFDs. The outcomes differ, and both
are asserted.**

```
$ irr ifd fuzz/seeds/ifd/malformed-photometric-on-thumbnail.tiff
ifds            2
  #0 @8 depth 0 chain — 3 entries, next 0        <- the malformed tag is here
  #1 @200 depth 1 sub of #0 — 9 entries, next 0
sensor_matches  [1]
sensor_ifd      #1
dimensions      4 x 2
...
exit=0                     <- the plane survives a bad tag on an unrelated IFD

$ irr ifd fuzz/seeds/ifd/malformed-photometric-on-only-candidate.tiff
ifds            1
  #0 @8 depth 0 chain — 9 entries, next 0
sensor_matches  []
sensor          <none: no IFD matched the sensor-plane rule, and 1 candidate(s)
                 could not be evaluated because an identifying tag was malformed:
                 [(0, 262)]>
exit=1                     <- and it says WHICH IFD (0) and WHICH TAG (262)
```

Both `#[test]`s pass: `malformed_on_thumbnail_does_not_lose_the_plane` and
`malformed_on_the_sensor_ifd_is_reported_not_hidden` (`tests/ifd_reader.rs`).
`NoSensorIfdCandidatesMalformed` (`src/lib.rs:166`, Display at `:227`) does name
both, in the error itself rather than in a log. FU-11 is genuinely closed for the
class it names, and `SensorMatch` (`src/ifd.rs:579`) is the right shape — a
`Result` could not represent "skip and keep scanning" without a side channel.

## The five literally-named tests — summed across targets, not read off the first

⚠ Four live in `tests/ifd_reader.rs`; `malformed_tag_costs_only_that_tag` is a **lib
unit test** (`src/ifd.rs:1508`), which is exactly why a first-target read reports
zero. Each filter selects **one real test that passes**:

```
tag_model_matches_exiftool                      -> test tag_model_matches_exiftool ... ok
orientation_is_per_frame                        -> test orientation_is_per_frame ... ok
absent_tag_is_absent_not_zero                   -> test absent_tag_is_absent_not_zero ... ok
malformed_tag_costs_only_that_tag               -> test ifd::tests::malformed_tag_costs_only_that_tag ... ok
malformed_on_thumbnail_does_not_lose_the_plane  -> test malformed_on_thumbnail_does_not_lose_the_plane ... ok
```

## Oracle red-proof (§15 check 9) — run by me, not taken on report

`tag_model_matches_exiftool` is the oracle. I swapped `top`/`left` in the
`ActiveArea` mapping (`src/ifd.rs:1043`) and watched it fail:

```
assertion `left == right` failed: PENTAX-K3III-MONO/K3III.DNG: ActiveArea
  left: Some(ActiveArea { top: 26, left: 34, bottom: 4194, right: 6250 })
 right: Some(ActiveArea { top: 34, left: 26, bottom: 4194, right: 6250 })
test result: FAILED. 0 passed; 1 failed
```

Worth recording: the fault is invisible on the Q2 Monochrom, whose `ActiveArea` is
`0 0 5632 8392` — `top` and `left` are both 0, so a swap changes nothing. The
Pentax's asymmetric `26/34` is the only thing in the corpus that can see this
particular fault. A single-body oracle would have gone green on a real transposition.

## Fuzz red-proof at the NEW code (§15 check 10) — the new paths are genuinely covered

The target reaches the new work (`fuzz/fuzz_targets/ifd.rs` calls
`sensor_candidates()` and `sensor()`), but reaching is not covering, so I proved it.
Planted a lint-clean fault in `scan_sensor` (`src/ifd.rs:949`) — `self.ifds.split_at(usize::MAX)`
when `unreadable.len() >= 2`, i.e. two IFDs each with an unreadable identifying tag,
a shape **no committed seed has**, so the fuzzer had to synthesise it.

- **Negative control first, with the fault live:** `cargo test --all-features` →
  **52 passed, exit 0**; `just lint` → exit 0; `just lint-no-allow` → exit 0. The
  fuzzer is the only thing that can see it.
- **Red:** libFuzzer found it, `deadly signal`, **exit 77**, `crash-6a0da6a3cd4b48df`,
  a 314-byte input it built by mutating the thumbnail seed into a container with a
  *second* unreadable IFD.
- **Restore:** fault removed, artifact deleted, `src/ifd.rs` sha256 back to
  `496c2baadf5814de2efa52c8b42af8419f0f6fcb1aa4e42b46ca922d86a3e104` (identical to
  the pre-fault digest), `grep 'DELIBERATE FAULT'` = 0, `git status` clean, and the
  clean re-run above at 10,365,858 execs with zero artifacts.

## Punch list — six follow-ups, none ship-blocking

### FU-16 (follow-up, highest value) — `sensor()` still loses the plane to a malformed tag on a NON-sensor IFD, via `Orientation`

`src/ifd.rs:1011-1017`. `sensor()` reads `TAG_ORIENTATION` from **IFD0** with a bare
`?`. That is a cross-IFD read on the one tag that requires one — and it is FU-11's
exact failure shape surviving at the site FU-11 did not name. Reproduced with a
fixture I built by re-pointing the thumbnail fixture's malformed entry at tag 274:

```
ifds            2
sensor_matches  [1]        <- the plane WAS located; the tri-state worked
sensor          <none: tag 274 has unreadable field type 250>
exit=1                     <- and then it was thrown away
```

**Not a regression** — I checked `git show main:src/ifd.rs`, the code is identical
on `main`, so this is inherited from SPEC-003, not introduced here. And DEC-012's
*table* sanctions it: interpret-phase, "fatal to that call only". But DEC-012's
*principle* — "a malformedness that changes only what a known-optional field says
costs that field alone" — points the other way, and `Sensor::orientation` is
`Option<u32>`. **The table and the principle disagree for optional scalar tags, and
the code follows the table.** That is the thing worth deciding, not the one line.

### FU-17 (follow-up) — a DNG-legal `RATIONAL` `DefaultCropSize`/`DefaultCropOrigin`/`BlackLevel` makes the whole file unreadable

`src/ifd.rs:788-800`. `uints()` rejects `TYPE_RATIONAL` with `UnexpectedFieldType`,
and `array()?` / `scalar()?` propagate it straight out of `sensor()`. DNG 1.7 permits
`RATIONAL` for exactly the tags AC1 introduced typed extraction for. Built a
spec-legal fixture (single IFD, `DefaultCropSize` as `RATIONAL` `8368/1, 5584/1`):

```
sensor_matches  [0]        <- located
sensor          <none: tag 50720 has unreadable field type 5>
exit=1                     <- discarded, and "unreadable" is untrue: type 5 is legal here
```

The build flagged `RATIONAL` as a follow-up but framed it as "unimplemented in
`uints()` … will surface the moment one does". The blast radius is larger than that
framing: it is **fatal to the file**, not a missing field, and the message calls a
spec-legal field type "unreadable". No corpus file exercises it, which is why it is
a follow-up and not a blocker — but it is the same one question as FU-16 and the two
should be decided together. Worth a spec.

### FU-18 (follow-up) — do **not** set `.repo-context.yaml`'s `cost.metering_source` to `none`; the answer is per-session, and the number was obtainable

Answering the handoff's scrutiny item 3 directly: **no.** `metering_source`
(`.repo-context.yaml:56`) is a **repo-global** switch, and DEC-013's own Context
lists flipping it as bad outcome #2 — *"the cost data is lost permanently, for
cycles that do have a number available."* SPEC-003 produced four real
dedup-by-`message.id` figures from delegated cycles; setting the global to `none`
would retroactively excuse those too. The gate is silent right now only because
`scripts/cost-audit.sh:51-57` gates **shipped** specs — SPEC-004 will fail it at ship.

And the premise is wrong anyway: **I obtained a real number from the same top-level
interactive execution mode the build reported as unmeterable** — 5,991,740 deduped,
by reading this session's own transcript at
`~/.claude/projects/<path-slug>/<session-id>.jsonl`, where the session id is
discoverable from the scratchpad path the harness provides. The build's reasoning
was honest and DEC-013-compliant (never invent a number); its premise that no source
existed was simply not checked. So the gate should keep asking.

The real fix is already half-written: the open signal `token-counts-not-comparable`
(`guidance/signals.yaml:108`) proposes a per-session `basis:` field
(`agent-result | transcript-sum | slash-cost`). Add `none` as a fourth basis and
have `cost-audit` accept a null only when that session's basis says so. Per-session,
not repo-global. Disposition at project close.

### FU-19 (follow-up) — `cargo test <name>` matching zero tests exits 0: make it a **check**, and a signal second

The build's observation is correct and I reproduced it:

```
$ cargo test --all-features a_test_name_that_does_not_exist_anywhere
test result: ok. 0 passed; ... 31 filtered out
test result: ok. 0 passed; ... 0 filtered out
test result: ok. 0 passed; ... 9 filtered out
test result: ok. 0 passed; ... 12 filtered out
exit=0
```

Every spec in this repo names its tests in `## Failing Tests`, and §15 verify check 2
is *"Failing tests from spec now pass"* — which a zero-match green satisfies
vacuously. **Both, but the check first.** A signal alone reproduces
`brag-step-skipped-at-ship` exactly: a step nothing surfaces. The cheap version is a
`just failing-tests SPEC-NNN` recipe that reads the spec's `## Failing Tests` block
and asserts each filter reports ≥1 passed. Two notes for whoever writes it: it must
**sum across targets** (one of these five lives in the lib target), and it must run
with the corpus present — `tag_model_matches_exiftool` and `orientation_is_per_frame`
also pass vacuously on a bare runner, because the skip-when-absent harness `continue`s.

### FU-20 (follow-up, minor) — `NoSensorIfdCandidatesMalformed` can name IFDs that were never candidates

`src/ifd.rs:916-933`. `is_sensor_ifd` reads all three identifying tags before
combining them, so an IFD with a **readable** `NewSubfileType == 1` — a preview,
definitively disqualified — still returns `Unreadable` if its `Photometric` is
malformed. The shipped `malformed_photometric_on_thumbnail` fixture is exactly that
shape. Harmless today: it only enriches the failure-path error, and over-reporting
"identity unknown" is the conservative direction. But it makes the error's own word
*"candidate"* untrue. Short-circuiting on a readable disqualifying `subfile_type`
would fix it.

### FU-21 (follow-up, minor) — `cost.totals` is `0 / 0 / session_count: 0` with sessions recorded

SPEC-004 front-matter. Totals are stamped at ship, so this is a ship-cycle item
rather than a defect now — flagged only so it is not inherited silently, which is
what happened at SPEC-003's FU-13 (*"a wrong-but-plausible number is the exact
failure mode this round was about"*). With this cycle there are two sessions and one
real number.

## Checks that passed and are worth stating

- **§15 check 5 (non-trivial implementer decisions need a `DEC-*`):** the build
  emitted none, and I agree. `SensorMatch`, the infallible `sensor_candidates()` and
  `Error::NoSensorIfdCandidatesMalformed` were all **prescribed** by FU-11's own text
  ("skipped and recorded", "the error must say why") and DEC-012's rule — the shape
  was specified, not chosen, and no new debatable fork was opened. Note it does make
  two **breaking public API changes** (`is_sensor_ifd`, `sensor_candidates`); fine at
  0.1.0 under DEC-007, with no consumers, and disclosed in HANDOFF-015's deviations.
- **§15 check 11 (provenance):** no new algorithm, so no new row. The existing
  `src/ifd.rs` row was **extended** rather than duplicated, same class 1 —
  specification, and states plainly that no implementation was consulted. Honest.
- **§15 check 12 (dependencies):** none added. `Cargo.toml` is not in the diff and
  both `deny` graphs are unchanged.
- **§15 check 7 (`cost.sessions` for prior cycles):** the design cycle has no entry,
  which is correct — `scripts/cost-audit.sh:12-14` meters build and verify only.
- **DEC-012's deferred pointer comments** are both present and both accurate:
  `src/ifd.rs:741` above `sub_ifd_offsets_of_last()` (walk-phase, strict) and `:876`
  above `array()` (interpret-phase, tolerant). That was the one thing DEC-012 asked
  SPEC-004's first edit to do.
- **AC3 on a real file, not only a synthetic:** `absent_tag_is_absent_not_zero` is a
  tier-A synthetic that runs on a bare CI runner, *and* the oracle table pins
  `active_area: None` on both M Monochrom bodies (`tests/ifd_reader.rs:159,183`). The
  synthetic proves absent ≠ present-and-zero; the corpus proves it happens for real.
- **`references.decisions`** is now `[DEC-008, DEC-012]`, closing SPEC-003's FU-12.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-004-tag-model` (local commit; not pushed, no PR — "commit; do not merge")
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** **Yes — all seven**, read as the handoff directs
  (against the deviation, not the spec's stale literal wording). AC1 typed structs
  (`src/ifd.rs:421/435/445`); AC2 `orientation_is_per_frame`, two real Q2M frames
  disagreeing 1 vs 6; AC3 absent ≠ present-and-zero, synthetic *and* both M
  Monochrom bodies pinned `None`; AC4 DEC-012 implemented with both deferred
  pointer comments; AC5 FU-11 closed and both fixtures re-run by me at the CLI;
  AC6 `exiftool` table over all 7 files with an `EXPECTED_FILES` drift guard, and
  I watched it go red; AC7 the fuzz target reaches the new paths and I proved it
  by planting a fault there and making libFuzzer find it.
- **For `verify`:** ✅ **APPROVED at `37204d0`** — six follow-ups (FU-16…FU-21),
  **no ship-blockers**.

### Cost self-report

- **Tokens (total):** **5,991,740** — a transcript sum **deduplicated by
  `message.id`**, the same methodology as HANDOFF-011/012/013/014. 124 usage
  objects, 49 distinct ids, raw 15,050,048 vs deduped 5,991,740 = **2.51×**
  inflation, **97.0%** cache-read. It is a **FLOOR** — computed before the session
  closed. This is the **eighth** measured factor and a **new high**: the band was
  1.61× / 1.76× / 1.82× / 1.86× / 1.95× / 2.20× / 2.25× and is now **1.61×–2.51×**,
  a 1.56× spread over eight observations. That strengthens rather than weakens the
  standing rule — **no fixed correction may be applied to any raw figure**,
  including SPEC-001's `cost.totals` of 51,979,929, which must be re-summed with
  dedup and not divided.
- **Estimated USD:** 39.55 (5,991,740 × 6.60 USD/Mtok, `.repo-context.yaml`'s DEC-002 basis)
- **Duration (minutes):** ~45, estimated from the shape of the work (three fuzz runs
  totalling ~170 s, two full corpus test runs at ~13 s each, ~20 cargo invocations,
  plus reading time) — not measured against a wall clock.
- **Source of the number:** transcript sum, deduped by `message.id` — see FU-18.
  **The build reported `null` for this same execution mode and did not need to.**

### Drift and new artifacts

- **New decisions emitted:** none. Everything I found is either a follow-up on
  existing records (FU-16 needs DEC-012 amended or superseded, not a new fork
  opened by a reviewer) or process debt belonging to an already-open signal.
- **Deviations from spec:** none by me. The build's five deviations are all
  disclosed in HANDOFF-015 and I checked each: the `SensorMatch` return type, the
  infallible `sensor_candidates()`, the new `Error` variant, the three corrected
  doc comments, and the two test renames. All four of the first are prescribed by
  FU-11 and DEC-012 rather than chosen; the doc-comment corrections are accurate
  (every AC1 tag is SHORT/LONG, so the wider type model was never this spec's
  scope); the renames preserve coverage and are what makes the spec's literal
  `## Failing Tests` commands select anything at all.
- **Follow-up work identified:** FU-16 through FU-21 above. **FU-16 and FU-17 are
  one spec, not two** — both are "a locatable plane discarded because one optional
  tag on some IFD could not be read", which is the question DEC-012 exists to
  answer and currently answers two ways. FU-18 and FU-19 are process debt for
  `guidance/signals.yaml`; FU-19 wants a `just` recipe, not just a note.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing, and the handoff deserves credit for the reason: it told me the spec
   was wrong, *why* it was wrong, and that I should verify the deviation instead.
   That inverted the usual verify failure mode, where the reviewer re-derives the
   architect's stale premise from the same incomplete look. The one thing I would
   have wanted stated is which *class* of finding counts as ship-blocking when the
   spec's literal ACs are all met but a neighbouring instance of the same defect
   survives — I resolved it by weighing "is it a regression?" (no, `main` carries
   FU-16's code identically) and "does a written record sanction it?" (yes, DEC-012's
   table), and landed on follow-up. Someone else could reasonably have blocked.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-013 should have been in the handoff's references, not just gestured at in
   scrutiny item 3. Its Context already contains the answer to the question the item
   asks — it names `metering_source: none` as one of the three bad outcomes the
   decision exists to avoid — so the question was pre-decided and reading the DEC
   was what settled it in one pass.

3. **If you did this task again, what would you do differently?**
   — Read my own transcript for the token count at the *start* rather than the end.
   I nearly reported `null` by inheriting the build's premise instead of testing it,
   which would have been the same defect this whole round is about: an assertion
   taken from a neighbouring document rather than measured. It cost one command to
   check. Relatedly: I would run `cargo test` without piping through `tail` from the
   first invocation — the handoff warned about under-counting by reading the first
   target, and I under-counted by discarding it, which is the same error mirrored.
