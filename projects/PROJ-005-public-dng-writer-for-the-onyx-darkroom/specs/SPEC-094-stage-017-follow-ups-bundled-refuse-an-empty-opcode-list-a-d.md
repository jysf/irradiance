---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-094
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-005
  stage: STAGE-017
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-051, DEC-052, DEC-075, DEC-080, DEC-081]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-061, SPEC-081, SPEC-082, SPEC-074]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-061, SPEC-081, SPEC-082]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-017's <capability>". Optional; null is acceptable.
value_link: "Closes STAGE-017's three [XS] follow-ups so the stage can close and the writer can be released."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 8000000        # S; three small, independent items, one of them a new refusal
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-25
      notes: "main-loop, not separately metered (AGENTS.md section 4)"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-094: STAGE-017 follow-ups, bundled: refuse an empty opcode list, a dng_write fuzz CI job, writer doc links

## Context

All three STAGE-017 specs have shipped: SPEC-061 (the writer core), SPEC-081 (metadata) and
SPEC-082 (the preview). The stage backlog still holds three [XS] items, each found by a verify
or a design. They are bundled into one spec the way PROJ-004 bundled its follow-ups
(SPEC-071, SPEC-074): one build, one verify, one ship. The writer has not been published yet
(the release spec comes after the stage close), so changing a SPEC-061 behaviour here reaches
no user.

1. **Empty opcode list** (DEC-081 "Follow-up"). The writer writes `Some(&[])` for any opcode
   list as a count-0 `UNDEFINED` tag. DEC-081 §2 refuses the same shape for XMP (`EmptyXmp`),
   because irradiance's own reader treats a count-0 interpretation tag as malformed (SPEC-089).
   An opcode list with no bytes has no 4-byte count header either, so no reader can walk it.
2. **No CI fuzz job for `dng_write`** (SPEC-061 verify F3). Every other fuzz target has a 60 s
   `fuzz smoke` job in `.github/workflows/`. `dng_write` runs only locally
   (`just fuzz-dng-write`).
3. **Three broken private doc links** (SPEC-061 verify F4). Running
   `cargo doc --no-deps --document-private-items` with `RUSTDOCFLAGS='-D warnings'` and
   **without** `write` fails on three links:
   - `[`crate::dng::write::Framing`]` and `[`crate::dng::write::WriteInputError`]` in
     `src/write_types.rs`'s module docs (lines 8 to 9);
   - `[`dng::write`](crate::dng)` in `src/lib.rs` (line 147).

   They resolve only when `write` is on. Measured on `8787014`: the same command with
   `--all-features` does not fail on them. It does fail on 13 pre-existing `ljpeg` private
   links, which are out of scope.

## Goal

The writer refuses an empty opcode list with a typed reason under every framing. CI fuzzes
`dng_write` for 60 s like every other target. The three writer doc links no longer break a
no-feature private-items doc build.

## Design

### 1. Refuse an empty opcode list

- **New reason:** `WriteInputError::EmptyOpcodeList { list: u8 }`, where `list` is 1, 2 or 3.
  The enum is `#[non_exhaustive]` (DEC-052), so this is additive. Its `Display` is
  "OpcodeList{list} is empty". Classified `CallerError` like every `WriteInputError`
  (DEC-075 §7). No new `Error` variant.
- **Doc comment:** "An opcode list was given with no bytes. A count-0 tag is malformed, and
  there is no count header to walk (`DEC-081` follow-up). Leave the option `None` to write
  no list."
- **When:** `Some(bytes)` with `bytes.is_empty()`, for any of the three lists, under
  **every** framing, including FullSensor, which copies lists verbatim.
- **Where in the order:** first in the "opcode lists" step: lists 1, 2, 3 in order, then the
  existing `OpcodeList1` walk. So an empty `OpcodeList2` with a shifting `OpcodeList1` under
  StandardCrop is `EmptyOpcodeList { list: 2 }`, not `UnsupportedOpcodeForFraming`. The
  check runs in `plan`, so `written_len` and `write_into` agree (DEC-075 §5, onyx's
  preflight).
- **Not changed:**
  - A non-empty list is still copied verbatim.
  - Only `OpcodeList1` is walked, and only under a framing that cuts.
  - A 4-byte list declaring 0 opcodes is not refused here. It is walkable, so it is not this
    defect.
- **Docs:** add the reason to the module docs' "Checks and their order" ("the opcode lists
  (none empty, then …)"), add a CHANGELOG `[Unreleased]` line, and record the resolution in
  DEC-081's Follow-up bullet ("resolved by SPEC-094").

### 2. A `dng_write` fuzz smoke job in CI

Add `fuzz-dng-write` to `.github/workflows/` next to `fuzz-ifd`, `fuzz-plane` and
`fuzz-ljpeg`, in the same shape:
- named `rust / fuzz smoke — dng_write (60s)`, on `ubuntu-latest`, nightly toolchain;
- `cargo install cargo-fuzz --locked`, then `mkdir -p fuzz/corpus/dng_write`;
- run `cargo +nightly fuzz run dng_write fuzz/corpus/dng_write fuzz/seeds/dng_write -- -max_total_time=60`.

Its comment names SPEC-061 verify F3. Update every list that counts or names the CI jobs
(AGENTS.md, `guidance/toolchain-brief.md`, any gate list a handoff template reads). CI goes
from 22 jobs to 23.

### 3. The three doc links

Make the three links resolve in every feature set. Do not change the docs' meaning, and add
no `#[allow]`.
- **`src/write_types.rs` module docs:** write the two names as plain code
  (`` `dng::write::Framing` ``), since the module is private and the types have no public
  path without `write`. Keep `[`crate::Error`]` and its other links, which resolve.
- **`src/lib.rs:147`:** either the same plain-code form, or a `cfg_attr(feature = "write",
  doc = ...)` pair. Choose the one that keeps the public no-feature docs reading correctly,
  and say which in the handback.
- **Check:** `doc-check` (public docs, `-D warnings`) must still pass. The private-items
  command above, without features, must list no `crate::dng` link (the 13 `ljpeg` errors
  remain).

## Inputs

- **Files to read:**
  - `src/dng/write.rs`: `plan`, the opcode-list step, and the module docs.
  - `src/write_types.rs` and `src/lib.rs` around line 147.
  - `tests/dng_write.rs` (`every_refusal_is_typed`, `checks_run_in_the_documented_order`).
  - `tests/support/dng_write_fuzz.rs` and `examples/fuzz-seeds.rs`.
  - `.github/workflows/` (the fuzz jobs).
- **Decisions:** DEC-075 §5, §7 and §9; DEC-081 §2 and its Follow-up; DEC-080; DEC-051 and
  DEC-052.

## Outputs

- **Files modified:**
  - `src/write_types.rs`: the new reason, its `Display`, and the module-doc links.
  - `src/dng/write.rs`: the check and the module docs.
  - `src/lib.rs`: one doc link.
  - `.github/workflows/<ci>.yml`: the new job.
  - Tests, fuzz support and seeds; CHANGELOG; DEC-081; AGENTS.md (CI job list, if any).
- **New public items:** `WriteInputError::EmptyOpcodeList { list: u8 }`, behind `write`.
- **No new options or flags.**

## Acceptance Criteria

- [ ] **AC1 Refusal.** `Some(&[])` for each of `opcode_list_1`, `opcode_list_2` and
  `opcode_list_3` is `InvalidWriteInput(EmptyOpcodeList { list: n })`, with the right `n`,
  under each of the three framings, from both `written_len` and `write_into`.
- [ ] **AC2 Order.** An empty list is refused before the `OpcodeList1` walk: with an empty
  `OpcodeList2` and a shifting `OpcodeList1` at StandardCrop the error is
  `EmptyOpcodeList { list: 2 }`. With an empty XMP as well, `EmptyXmp` comes first. With
  both an empty list 1 and an empty list 3, the error is `{ list: 1 }`.
- [ ] **AC3 Nothing else moves.** Every existing writer test passes unchanged, including
  SPEC-082's three pinned MD5s. A 4-byte count-0 list is still accepted and copied
  verbatim.
- [ ] **AC4 Fuzz.** A `refuse-empty-opcode-list` seed reaches the new reason, and
  `fuzz_seeds_reach_their_named_outcome` passes. Every seed that previously reached `Ok`
  with an empty list is renamed or re-pointed, and the handback lists each one.
  `just fuzz-dng-write` runs 60 s with no crash.
- [ ] **AC5 CI job.** `rust / fuzz smoke — dng_write (60s)` exists and passes on the PR, so
  23 of 23 checks pass.
- [ ] **AC6 Doc links.** `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
  --document-private-items` without features reports no unresolved `crate::dng` link. With
  `--all-features` it reports none of the three either. `just doc-check` passes.
- [ ] **AC7 Gates.** `just test` (corpus and `dng_validate` set), `lint-ci`,
  `lint-no-allow`, `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`,
  `lint-fuzz-red-proof`, `msrv`, `validate`, `gate-audit`, `cost-audit` and
  `semver-check`.

## Failing Tests

- **`tests/dng_write.rs`**
  - `empty_opcode_list_is_refused_under_every_framing`: AC1, the 3 lists × 3 framings × both
    entry points.
  - `every_refusal_is_typed`: extended with the new reason.
  - `checks_run_in_the_documented_order`: extended with AC2's three pairs.
  - `count_zero_opcode_list_header_is_still_written_verbatim`: AC3's 4-byte list
    (`00 00 00 00`) at FullSensor and StandardCrop, with the tag bytes read back exactly.
- **`tests/dng_write_fuzz` seeds:** `fuzz_seeds_reach_their_named_outcome` with the new
  seed.
- **The doc links and the CI job** have no unit test. Their evidence is the command output
  in AC6 and the PR's check list in AC5.

**Rule-clause map:**

| Clause | Test | What it must show |
|---|---|---|
| empty means refused, each list | `empty_opcode_list_…` | the error at each `n`, so a mutant that checks only list 1 fails |
| every framing, FullSensor included | `empty_opcode_list_…` | a mutant that checks only under cutting framings fails at FullSensor |
| both entry points | `empty_opcode_list_…` and `every_refusal_is_typed` | a mutant that checks only in `serialize` fails `written_len` |
| before the walk | `checks_run_…` (list 2 with a shifting list 1) | a mutant that checks after the walk returns `UnsupportedOpcodeForFraming` |
| list order 1, 2, 3 | `checks_run_…` (lists 1 and 3 empty) | a mutant that walks 3, 2, 1 returns `{ list: 3 }` |
| only zero bytes | `count_zero_opcode_list_…` | a mutant that refuses `len < 4`, or a declared count of 0, fails |

**Distinguishing fixtures.**
- The list-order case needs **two** empty lists. With one, both orders agree.
- The before-the-walk case needs a list 1 that the walk would actually refuse, at a framing
  that cuts. Use the `refuse-list-1-shifting` shape with a non-zero active-area origin.

**Moved assertions.**
- Grep `tests/` and `tests/support/` for every place an opcode list can be built with zero
  bytes. That includes the fuzz builder's length byte, where a length of 0 gives
  `Some(&[])`.
- Any assertion or seed that expects `Ok` from one becomes the new refusal. The build lists
  each one in the handback.
- At design, no test in `tests/dng_write*.rs` passes `Some(&[])` for an opcode list. The
  only `Some(&[])` is XMP's, at `tests/dng_write.rs:933`.

## Non-Goals

- The 13 `ljpeg` private doc links, or a CI gate on `--document-private-items`. Note it at
  the stage close if it is worth a future item.
- Walking `OpcodeList2` or `OpcodeList3`, or refusing a count-0 header. Any change to which
  opcodes are allowed (DEC-075 §5).
- Any reader change.

## Notes for the Implementer

- Keep the three items as separate commits. The verify can then mutate item 1 without
  touching the others.
- The fuzz smoke job should run for 60 s, as every other job does. Don't shorten it to save
  CI time.
- If the `lint-fuzz-red-proof` script copies files by list, check that adding a seed or job
  does not need a change there. SPEC-061's red-proof missed a file this way.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
