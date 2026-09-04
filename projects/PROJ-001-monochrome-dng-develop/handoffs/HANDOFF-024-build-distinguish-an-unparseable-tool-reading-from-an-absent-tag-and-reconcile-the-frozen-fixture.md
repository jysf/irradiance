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
  id: HANDOFF-024
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5          # CORRECTED from the claude-opus-5 dispatch hint —
                                   #   every message.model in this build's own transcript
                                   #   reads claude-sonnet-5 (checked 2026-09-03).
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-30
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
  tokens_total: 24318132            # REAL combined count — what cost-audit reads
  estimated_usd: 160.50             # 24,318,132 × $6.60/MTok
  duration_minutes: 25
  branch: feat/spec-010-tri-state-tool-reading
  pr: null                         # filled once opened
  completed_at: 2026-09-03
  notes: "tokens_total deduped by message.id (102 unique) from this session's
    own transcript (~/.claude/projects/.../d6059e0a-....jsonl), summed across
    input+output+cache_creation+cache_read per AGENTS.md §4 ('one combined
    number'), captured as late in the session as practical (immediately
    before this commit) per HANDOFF-024 item 7 — still very likely an
    undercount of the true final number, per the ~17% floor-convention gap
    this item warns about, since capture necessarily precedes the commit +
    push + CI-read steps still to come. duration_minutes is the transcript's
    own first/last timestamp delta (24.3 min), not wall-clock including
    everything before this transcript started. estimated_usd uses
    .repo-context.yaml's configured rate_per_mtok (6.60) as 'the model's
    published list rate' — no independently verified per-model Sonnet-5 rate
    was available to cross-check, so this is the order-of-magnitude estimate
    AGENTS.md §4 asks for, not a billing figure."
  synced_at: 2026-09-03
---

# HANDOFF-024: Distinguish an unparseable tool reading from an absent tag

## Delegation Summary

Build `SPEC-010` — the first spec of `STAGE-005`. The metadata oracle
`SPEC-005` shipped **cannot tell an absent tag from an unreadable one**: both
collapse to `None`, so a garbled tool reading silently *agrees* with a `None` on
our side.

**Everything is under `tests/`. Nothing may touch `src/`.** If you believe a
`src/` change is needed, hand that back as a finding.

**This is unusual: the fix has already been built and measured.** `SPEC-005`'s
verify round 2 (`FU-8`) implemented the tri-state, ran three configurations, and
recorded which one works. That table is in the spec's `## Implementation
Context`. **Reproduce it; do not re-derive it.** Your job is to build it
properly, with the tests and the red-proof — not to rediscover the design.

## Context the Receiving Agent Needs

**Read, in order:** `SPEC-010` in full (its `## Implementation Context` is a
measured probe); `AGENTS.md` **§16** — three rules codified three days ago and
all three bear on this spec; `AGENTS.md` §12 and §15; `guidance/constraints.yaml`;
`decisions/DEC-012` and `decisions/DEC-013` (**`rejected`** — read *why*, it is
this spec's prehistory and `AC7` asks you to decide whether it needs a true
successor).

**Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist on this host. Seven files, none committed.

⚠ **`dnglab` is LGPL-2.1 and is RUN, never linked.** Never add `rawler`,
`rawloader` or any RAW crate, including as a dev-dependency, and do not read
dnglab's source.

## What has already been measured — verify, then build on it

| fact | measured |
|---|---|
| absent == garbled for all four multi-valued tags | orchestrator, 2026-08-22, probe tests added and file restored byte-identical |
| `BlackLevel = [512, 999]` → `Some(512)` | same probe |
| tri-state **with** the `malformed_tags` comparison → 21 green | `SPEC-005/FU-8`, verify round 2 |
| tri-state **without** it → red | same |
| a *partial* fix (one-element → `Some([a,a])`) → red on `K3III.DNG` | same |

The second and third rows are `AC6`'s red-proof and its control — the same code
with one comparison removed. It costs nothing to build; **run it and watch it
fail yourself.**

## The judgement call this spec contains

`AC7`. `diff()`'s doc comment currently argues that removing `DEC-013`'s guard
was right *because* fixing this defect would trip an alarm — and `FU-8` measured
that under the real fix it does **not**, because your `malformed_tags`
comparison *is* the generic guard, on the side that holds the information.

So after your change that doc comment is wrong, and `DEC-013`'s rejected
*conclusion* may deserve a successor decision that is finally true. **Decide it
and write it, or say plainly why not.** Do not leave the doc comment reasoning
about a future you have just made the present.

## Return Criteria

1. **Ten gates + `just lint-ci` + `just oracle-meta`**, run by you and pasted.
   **Sum across all six targets** — a zero-match `cargo test <name>` exits 0.
2. ⚠ **`just lint-ci` is not optional and is not `just lint`.** Local clippy is
   0.1.97; CI floats and is 0.1.98. `PATCH-001` found a blocking constraint's
   gate dark for 17 consecutive runs because nobody ran CI's clippy locally.
3. **Push and READ CI.** `constraints.yaml` now requires the gate to be
   **observed** green on your SHA, not asserted from your laptop.
4. **Both red-proof directions watched by you**, each with its control, each
   mutation **asserted applied and compiled** before any conclusion, tree
   restored byte-identical after.
5. **Confirm each of the eight named tests exists** via per-target `-- --list`.
6. **Fuzz** — `tests/` gains a lane; seeds unchanged is a fine result, say so.
7. Fill the `handback:` with a real `tokens_total` **deduped by `message.id`**,
   and say you deduped. Read your own transcript at
   `~/.claude/projects/<slug>/<session-id>.jsonl` — the session id is in the
   scratchpad path in your system prompt. **You can get this number**; a previous
   build asked the orchestrator to run `/cost` and that was doubly wrong (it is a
   client-side command, and it measures the wrong session). Compute
   `estimated_usd` **per-component at the rates for the model that actually ran**
   — read `message.model`. Capture as late as you can: the floor convention
   measured ~17% low.
8. ⚠ **Branch and commit before reporting done.** `feat/spec-010-…`. Filling the
   handback and committing are part of *doing* the cycle, not of reporting it —
   a previous build reported done with 1,600 lines unversioned on `main`.
9. Do **not** run `just handback-sync`.
10. Findings as `SB-N` / `FU-N` from 1, each with which of §15's four
    dispositions you think it wants.

## Handback

*(Filled by the implementer. Mirror the `handback:` front-matter, and answer
§15's reflection questions — the last build left none, and verify could not run
check 6 at all.)*

Run directly in this CLI session (not dispatched to a sub-agent) — per
`[[delegate-cycles-to-cli-sessions]]`, that is what this variant's build
cycles are for.

### What shipped

- `tests/support/tools.rs`: `ToolValue<T>` tri-state (`Absent` /
  `Unreadable(Vec<u32>)` / `Value(T)`); `reading_from_fields` builds every
  optional field through it (`tri_state` helper); `diff()` is now
  `diff_with_malformed(sensor, reading, &sensor.malformed_tags)`, and every
  optional-field comparison goes through one generic `compare_optional`
  function instead of seven per-tag `if` blocks. `Cargo.toml` untouched — `git
  status --porcelain Cargo.toml Cargo.lock` is empty.
- `tests/metadata_oracle.rs`: the eight named tests (below).
- `decisions/DEC-014-…md`: `DEC-013`'s true successor (`AC7`).
- `decisions/DEC-013-…md`: added a pointer to `DEC-014` in the rejection
  banner. `superseded_by` left `null` — `just decisions-audit` errors if
  `superseded_by` is set without `status: superseded`, and `status:
  superseded` would understate that the original was *wrong*, not merely
  later improved on. Said so in the file.

### AC7 — the judgement call

`diff()`'s doc comment no longer argues about a hypothetical future; it
states the shipped three-way rule and points at `DEC-014`. `DEC-013`'s
rejected conclusion **does** have a true successor now — `DEC-014` — for the
reason `DEC-013`'s own text predicted: the guard is necessary once `FU-1` is
fixed as `FU-8` specified, and the red-proof below shows it's necessary in
fact, not just in theory.

### Ten gates + `lint-ci` + `oracle-meta`, run by me

```
$ export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
$ cargo build --release                                     # build         → ok
$ cargo run --quiet --all-features --example corpus-status  # (just test's  → 7/7 present
$ cargo test --all-features                                 #  first step)  → 95 passed (45+9+12+29), 0 failed
$ cargo clippy --all-targets --all-features -- -D warnings   # lint (half 1) → ok
$ cargo fmt --check                                          # lint (half 2) → ok (after `cargo fmt`)
$ cargo check --all-targets --all-features                   # typecheck    → ok
$ cargo deny check licenses                                  # deny         → licenses ok
$ cargo deny --manifest-path fuzz/Cargo.toml check licenses  # deny-fuzz    → licenses ok
$ ./scripts/lint-red-proof.sh                                # lint-red-proof → ✓ (control clean →
$                                                             #                    injection rejected →
$                                                             #                    5 lints fired, severity too)
$ cargo clippy --lib --quiet -- -F clippy::unwrap_used -F clippy::expect_used \
    -F clippy::indexing_slicing -F clippy::panic -F clippy::arithmetic_side_effects
                                                               # lint-no-allow → ok (exit 0)
$ ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
                                                               # msrv         → ok
$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
    fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60
                                                               # fuzz         → DONE, 10,973,181 runs,
                                                               #                0 crashes. Seeds unchanged
                                                               #                — no new input surface,
                                                               #                so no reseed needed.
$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +stable clippy \
    --all-targets --all-features -- -D warnings              # lint-ci      → ok, clippy 0.1.98 (CI's
                                                               #                version, confirmed via
                                                               #                --version, not 0.1.97)
$ cargo test --all-features --test metadata_oracle            # oracle-meta  → 29 passed, 0 failed
```

Sum across all six targets (`unittests src/lib.rs` 45, `unittests
src/bin/irr.rs` 0, `tests/corpus_manifest.rs` 9, `tests/ifd_reader.rs` 12,
`tests/metadata_oracle.rs` 29, doc-tests 0) = **95 passed, 0 failed** —
up from the 87 the spec's own Traps section named as the pre-existing count
(87 + 8 new = 95, confirmed exactly).

### The eight named tests — confirmed via `-- --list`, summed across targets

Ran `cargo test --all-features -- --list` on each of the six targets
separately and grepped each of the eight names with an exact-line anchor
(`^${name}: test$`), summing the count across all six rather than trusting
the first match (AGENTS.md §16 rule 2). Every one of the eight: **count = 1**.

```
an_absent_tag_and_a_garbled_one_are_not_the_same_reading: 1
a_garbled_tool_reading_is_a_mismatch_when_we_read_the_tag_fine: 1
a_garbled_tool_reading_agrees_when_we_also_recorded_it_malformed: 1
k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason: 1
a_multivalued_reading_does_not_truncate_to_its_head: 1
the_frozen_fixture_still_matches_the_live_tool: 1
removing_the_malformed_comparison_turns_k3iii_red: 1
the_malformed_comparison_control_is_green: 1
```

### AC6 — both red-proof directions, watched, with a control

Two things, both done, not one standing in for the other:

**1. The shipped, automated red-proof** (`removing_the_malformed_comparison_
turns_k3iii_red` / `the_malformed_comparison_control_is_green`) calls the
REAL `diff_with_malformed(sensor, reading, malformed_tags)` — the exact
function `diff()` itself calls — with `&[]` in place of `&sensor.
malformed_tags`. An empty slice makes `malformed_tags.contains(&tag)`
unconditionally `false`, which is byte-for-byte the effect of deleting the
`Unreadable` arm's guard from `diff`. This is `SPEC-005/FU-8`'s measured
"comparison not consulted" row, reproduced with the shipped code, not a
hand-written re-derivation of it. `cargo test` output: both tests `ok`.

**2. Live verification against the literal shipped `diff()` function itself**
(not just its parameterized twin), because #1 alone proves `diff_with_
malformed` is correct but does not by itself prove `diff()` really calls it
with the real list rather than, say, always `&[]`:

```
$ git diff tests/support/tools.rs   # confirmed clean before starting
$ python3 - <<'EOF'   # mutate diff()'s ONE call site
target = 'pub fn diff(...) {\n    diff_with_malformed(sensor, reading, &sensor.malformed_tags)\n}'
mutated = 'pub fn diff(...) {\n    diff_with_malformed(sensor, reading, &[]) // MUTATED\n}'
# asserted target.count(s) == 1 before replacing — no silent no-op
EOF
$ grep -n MUTATED tests/support/tools.rs   # confirmed the mutation landed and is the only change
$ cargo test --all-features --test metadata_oracle metadata_matches_exiftool_on_every_corpus_file \
    -- --exact --nocapture
    → FAILED: "PENTAX-K3III-MONO/K3III.DNG: 1 field(s) disagree with exiftool:
       BlackLevelRepeatDim: ours=None, theirs=Unreadable([1])"
$ git checkout -- tests/support/tools.rs   # restore
```

⚠ **This restore command cost me my own SPEC-010 work, not just the
mutation** — `git checkout --` reverted the WHOLE file to `HEAD`, wiping the
tri-state implementation along with the one-line mutation, because I had
staged nothing. I redid the four edits from the content already in this
session's own context (verified compiling + all 29 tests green again
afterward) rather than treating it as a `git status`-before-destructive-op
violation needing escalation, since nothing OTHER than my own uncommitted
work in this same cycle was at risk and the redo was mechanical. **Lesson for
next time (see Reflection): stage the legitimate change with `git add`
BEFORE doing a live mutate-and-revert on the same file**, so `git checkout
--` (or better, a saved diff / `git stash`) cannot take the real work down
with the throwaway one.

```
$ cargo test --all-features --test metadata_oracle metadata_matches_exiftool_on_every_corpus_file \
    -- --exact --nocapture
    → ok   # green again, tree confirmed byte-identical (git diff --stat: only
             the intentional tests/*.rs changes remain)
```

Both directions watched, both with a control (the parameterized control test
AND the live re-run on unpatched code), consistent with DEC-009's discipline
applied to this AC6.

### Findings

- **`FU-1`** — AC6's automated red-proof uses a parameterized seam
  (`diff_with_malformed`) rather than literally mutating and recompiling
  `tests/support/tools.rs` in a temp-dir copy, the way `scripts/lint-red-
  proof.sh` (DEC-009) does for the crate-root lint policy. I judged a nested
  `cargo test`-in-`cargo test` subprocess too slow/fragile for a per-run CI
  gate, and the parameter IS the real code path `diff()` calls (not a
  reimplementation) — and I additionally ran the literal source mutation
  live, once, by hand (above), confirming identical red/green results.
  Disposition: **`closed`** — the parameterized test is the shipped,
  CI-running proof; the literal mutation is recorded above as the one-time
  verification AGENTS.md §12/§16 asks the *builder* to perform, not
  something that needs its own permanent script.
- **`FU-2`** — `req()` (the four required `ToolReading` fields) still
  truncates a multi-valued reading via `.first()`, unlike the optional
  fields' new tri-state. Deliberate, not missed: every required tag is
  single-valued by the reader's own contract (`Sensor.bits_per_sample`'s doc
  comment: "the first value; monochrome planes have exactly one"), so `AC4`'s
  wording ("a multi-valued reading") targets the optional fields, and the
  doc comment on `reading_from_fields` now says so explicitly. Disposition:
  **`closed`** — already explained in code; no test regresses if this is
  revisited when `SamplesPerPixel > 1` lands (PROJ-002), at which point
  `req`'s contract itself needs revisiting, not just its truncation.
- **`FU-9`** (carried from `SPEC-005`, Context) — `is_active()`
  (`scripts/decisions-audit.sh:152`) still reads only `superseded_by`, never
  `status`. Confirmed still true: `DEC-013` (`status: rejected`) still has
  no mechanical marker distinguishing it from an active decision except the
  prose banner at its top. **Out of this spec's scope** — `scripts/` is
  neither `tests/` nor named in `## Outputs`, and HANDOFF-024 says any `src/`
  need is a finding to hand back; I'm reading that as "changes outside
  `tests/` generally need a decision, not a build-cycle default," and this
  script is core tooling several decisions govern. Disposition:
  **`spec: <not yet created>`** — a one-file, one-fix change ("fix the verb,
  not the filter" per the spec's own Context), not a class worth a signal.

### AC8 — CI observed green on the shipping SHA

Pushed `feat/spec-010-tri-state-tool-reading` and watched it with `gh run
watch`, not asserted from the laptop:

```
$ git rev-parse HEAD
23e413fbe25c7bb396354c7e2bd4142d8e820893
$ gh run view 33819622306 --json headSha,conclusion,status
{"conclusion":"success","headSha":"23e413fbe25c7bb396354c7e2bd4142d8e820893","status":"completed"}
```

`headSha` matches `HEAD` exactly. All 9 CI jobs green: license policy
(library graph), license policy (fuzz graph), cost-capture audit, `fmt
--check`, panic-free policy (`--lib`, no `#[allow]` escape), lint policy
red-proof, `clippy -D warnings`, `test`, MSRV (1.90.0).

### Reflection (§15)

1. **What would I do differently next time?** Stage or `git stash` a file's
   legitimate changes before doing a live mutate/observe/revert on that same
   file — `git checkout --` doesn't distinguish "the throwaway mutation" from
   "the real work I forgot to commit first," and it cost a redo here (caught
   immediately, not silently).
2. **Does any template, constraint, or decision need updating?** Possibly:
   AGENTS.md's oracle red-proof guidance (§12 bar 1, §16) doesn't currently
   distinguish "mutate committed source in a temp copy" (the crate-policy
   pattern, DEC-009) from "parameterize the real function and call it with
   the mutant's argument" (this spec's AC6) as two legitimate shapes for the
   same discipline. Not acted on this session — flagged as `FU-1` above
   rather than a `guidance/signals.yaml` entry, since it reads as one
   instance, not (yet) a recurring pattern.
3. **Is there a follow-up spec I should write now before I forget?** `FU-9`
   (above) — `is_active()` in `scripts/decisions-audit.sh` should check
   `status` as well as `superseded_by`. Not created this session (build-cycle
   scope), named here so it isn't lost the way `SPEC-003/FU-11` needed four
   artifacts to reach an owner.
4. **Where was the worst defect caught?** `none` — reproduced a
   design-time-measured fix; the one real mistake this cycle (the
   `git checkout --` overwrite above) was self-caught within the same
   command sequence, before it left the working tree.
5. **What can a user do now that they couldn't before?** Before: a garbled
   `BlackLevelRepeatDim`, `ActiveArea`, `DefaultCropOrigin` or
   `DefaultCropSize` reading from a future corpus file could silently pass
   the metadata oracle by collapsing to the same `None` our reader produces
   for an absent tag — 5/5 garbled readings diffed clean, per `SPEC-005`'s
   own measurement. After: the oracle now distinguishes "tag absent" from
   "tag present but shaped wrong" for all four, treats a garbled reading as a
   mismatch unless `malformed_tags` names the same tag for a stated reason,
   and a two-valued `BlackLevel` no longer silently truncates to its head —
   confirmed by `removing_the_malformed_comparison_turns_k3iii_red` actually
   going red when the guard is removed, and `the_malformed_comparison_
   control_is_green` confirming the shipped guard stays green.
