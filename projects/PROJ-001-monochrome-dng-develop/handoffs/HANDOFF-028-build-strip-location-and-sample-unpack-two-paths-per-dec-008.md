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
  id: HANDOFF-028
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED — read message.model this session: claude-sonnet-5.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-04
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-012

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
  status: completed                     # completed | blocked | rejected
  tokens_total: 29580529           # REAL combined count — what cost-audit reads
  estimated_usd: 89                # deliberate overestimate, no cache discount — see spec cost.sessions note
  duration_minutes: 35
  branch: feat/spec-012-strip-location-and-sample-unpack
  pr: null                         # NOT opened — return criterion 7 says leave this to the orchestrator
  completed_at: 2026-09-04               # YYYY-MM-DD
  notes: "Real tokens_total deduped by message.id, summed from this session's own ~/.claude/projects/<slug>/<session-id>.jsonl (104 distinct messages). Nine gates + lint-no-allow + lint-red-proof + lint-ci all green locally; fuzz (both ifd and the new plane target) clean at ~19.2M combined runs. Pushed the branch and read CI — see the Handback section below for the SHA and run link. handback-sync NOT run and PR NOT opened, per return criterion 7."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-028: Strip location and sample unpack — two paths per DEC-008

## Delegation Summary

Build `SPEC-012`. **This is the first spec in the project that produces pixels.**
Eight have read metadata; this one turns a strip into a linear `u16` plane.

`SPEC-009` shipped yesterday and is why you can trust the tags you are about to
read: every Structure-class membership is now load-bearing, so
`require_uncompressed()` cannot be walked past by a `RATIONAL 2/2` `Compression`.

## ⚠ The failure this spec is shaped around

`SPIKE-001` decoded 14-bit bit-exact **on its first attempt**. Its unpacker took
`bits` as a parameter and every frame it ever saw was 14, so `DEC-008`'s two
cases were indistinguishable. `SPIKE-002` ran it on a 16-bit body and got a
**byte-swapped plane** — wrong in a way that:

- still decodes without error,
- still has exactly the right length, and
- **still passes the layer-0 arithmetic check.**

Only the value range caught it. That is why `AC4` (`max > WhiteLevel` as a loud
error) is not a nicety, and why `AC3` asserts the *measured impossible values*
rather than "the outputs differ".

## What has been measured for you — reproduce, do not re-derive

The spec's `## Implementation Context` carries the first eight samples of both
files, obtained **two independent ways that agree exactly**: hand-unpacked from
the raw strip bytes, and read out of `dnglab --raw-pixel`'s own plane.

**Use them as your first checkpoint.** `SPEC-013` builds the MD5 oracle; until
then a whole-plane mismatch tells you nothing about *where*. Sample 0 tells you
which path you are on:

```
Q2M 14-bit  correct: 746      wrong (as 16-bit LE): 43019
M Mono 16-bit correct: 4761   wrong (as big-endian): 39186
```

Both wrong values exceed `WhiteLevel 16383` and are impossible.

## The decision you must make and record

`8424 × 5632 × 2 = 94,887,936` bytes for the plane, on top of an 86 MB input.
`library-not-application` says the consumer picks the allocator; `DEC-002`
(**proposed**, 0.72) is unresolved on `no_std`/`alloc`.

So: `unpack_into(&mut [u16])` or `unpack() -> Vec<u16>`? The spec sets out both
and gives the orchestrator's read (**`unpack_into` as the primitive**), **offered
as input, not as the answer**. **Write the `DEC` either way**, including if you
disagree — this is an API commitment, and `library-not-application` is a blocking
constraint.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, **summed across all six
   targets**. Then **push and read CI** — `constraints.yaml` requires the gate
   *observed* green on your SHA.
2. ⚠ **Fuzz is not optional here** (§12 bar 2) — the unpacker is a new input
   surface over attacker-controlled `bits`, `width`, `height` and strip bounds.
   **A target that only ever drives 14-bit recreates `SPIKE-001`'s exact blind
   spot.** Say how you know both paths were reached.
3. **The provenance row is required** — new algorithm, class 1 (specification),
   TIFF 6.0 + `DEC-008`. ⚠ **`SPIKE-001`'s decoder is discarded and must not be
   consulted**, and no copyleft RAW implementation may be read
   (`provenance-recorded-per-algorithm`). If the algorithm seems available only
   that way, **stop and ask** — that is a decision, not a build step.
4. **`AC8` wants a measurement, not an estimate.** Peak RSS for a 47 MP decode,
   by whatever means you can defend, with the method stated.
5. Every mutation: **assert it changed the file and compiled** before concluding.
   ⚠ **Stage your work before mutate-and-revert experiments** — `SPEC-010`'s
   build lost its entire change to `git checkout --` and shipped a
   reconstruction.
6. **Branch and commit before reporting done** (`feat/spec-012-…`), and fill the
   `handback:` — a real `tokens_total` **deduped by `message.id`**, said so.
   ⚠ **You can get this yourself**: read your own transcript at
   `~/.claude/projects/<slug>/<session-id>.jsonl` and sum `usage`, keeping one
   object per distinct `message.id`. The session id is in the scratchpad path in
   your system prompt. Do **not** ask the orchestrator to run `/cost` — it is a
   client-side command *and* it measures the wrong session. Price
   `estimated_usd` per-component at the rates for the model `message.model`
   reports, not `tier_map`.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1, each with a §15 disposition. ⚠ A `spec:`
   disposition must **name an AC in that spec that would fail** if the finding
   were left undone.
9. Answer §15's reflection questions in the handback.

## Handback

### 1. Gates, summed across all six targets

Ran in `~/PSeven/experiments/crustimg_redo_plus/irradiance-build-spec-012`
(a dedicated worktree, `feat/spec-012-strip-location-and-sample-unpack`, per
AGENTS.md §13's one-worktree-per-session rule):

```
 1. just build       → cargo build --release                                     GREEN
 2. just test        → six targets, summed:
                         irradiance (lib unit tests)  52 passed
                         irr (bin unit tests)          0 passed (none defined)
                         corpus_manifest                9 passed
                         ifd_reader                    12 passed
                         metadata_oracle               30 passed
                         plane_unpack                   7 passed
                         doc-tests                      0 passed
                         TOTAL: 110 passed, 0 failed, 0 ignored
 3. just lint         → cargo clippy --all-targets --all-features -- -D warnings  GREEN
                        cargo fmt --check                                         GREEN
 4. just typecheck    → cargo check --all-targets --all-features                  GREEN
 5. just deny         → cargo deny check licenses                                 GREEN ("licences ok")
 6. just deny-fuzz    → cargo deny --manifest-path fuzz/Cargo.toml check licenses  GREEN
 7. just lint-red-proof → control clean (exit 0) → injection rejected (exit 101),
                          all five lints fired, still fire without -D warnings     GREEN
 8. just lint-no-allow → cargo clippy --lib -F <five lints>                       GREEN (no output)
 9. just msrv         → ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features GREEN
10. just fuzz         → ifd target, 4,644,388 runs (16s), zero crashes            GREEN
11. just fuzz-plane   → plane target (NEW, this spec), 19,184,408 combined runs   GREEN
                         (45s + 20s across two sessions — see §3)

    just lint-ci      → PATH=~/.cargo/bin cargo +stable clippy --all-targets --all-features -- -D warnings
                        FAILED on the first pass — see §2 of this Handback — GREEN after the fix
```

Eleven `just` gates (`lint` is two commands, clippy + fmt) plus `just lint-ci`
— the twelfth command, run separately because it is the only one that sees
what CI sees (AGENTS.md §6). All green, on this branch, before push.
`just fuzz-plane` is the eleventh gate this spec adds — ten existed before
`SPEC-012`.

Corpus present for the tier-B half: `corpus-status` reported `7/7 present — no
tier-B test will skip` — none of the above numbers hide a skip.

**Push and CI**: pushed `feat/spec-012-strip-location-and-sample-unpack` to
`origin`, then **watched CI to completion and observed it green** on
`731a89171bfff9001af692fd0dfc291968eceafd` — all nine CI jobs passed (`clippy
-D warnings`, `fmt --check`, license policy x2 — library + fuzz graph, `test`,
`MSRV (1.90.0)`, lint policy red-proof, panic-free policy — no `#[allow]`
escape, cost-capture audit):
<https://github.com/jysf/irradiance/actions/runs/33932904592>. This satisfies
`constraints.yaml`'s "observed, not merely run locally" bar
(AGENTS.md §13 note on `lint-policy-red-proof`'s 17-run dark streak) — AC9 is
checked in the spec on the strength of this observation, not a self-report of
the local run alone.

### 2. The local/CI clippy-version gap, caught before push

`just lint-ci` failed on its first run: clippy 0.1.98 (CI's floating
`dtolnay/rust-toolchain@stable`) has a lint, `chunks_exact_to_as_chunks`, that
this machine's pinned Homebrew clippy 0.1.97 does not — exactly the gap
AGENTS.md §6 documents `lint-ci` existing to catch, reproducing on this spec
rather than staying a historical PATCH-001 anecdote. `src/plane.rs`'s
byte-aligned path used `strip.chunks_exact(1)`/`chunks_exact(2)`; rewrote both
to `strip.as_chunks::<1>()`/`as_chunks::<2>()` (already precedented in this
repo — `tests/support/corpus.rs`'s `sha256` module uses the same method),
which also let two dead `Truncated`-mapping fallbacks be deleted (`as_chunks`
hands back `&[u8; N]` directly, no fallible `try_into` needed). Re-ran the
full gate list after the fix; all green. No template/process change needed —
the gate did exactly its documented job.

### 3. Fuzz — both paths, reached and measured

New target: `fuzz/fuzz_targets/plane.rs` (registered in `fuzz/Cargo.toml`),
seeded by `examples/fuzz-seeds.rs`'s new `plane_seeds()` — six hand-built
fixtures in `fuzz/seeds/plane/`: a valid 14-bit (sub-byte) plane, a valid
16-bit (byte-aligned) plane, the 14-bit strip misdeclared as 16-bit (AC3/AC4's
own fixture), a truncated strip, `bits = 10`, and a `Compression = 7` plane.
Both of `DEC-008`'s paths are seeded on purpose — the header comment says why
a 14-bit-only target would recreate `SPIKE-001`'s blind spot.

Two runs, this session, this machine:

```
cargo +nightly fuzz run plane fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=45
  → 13,050,886 runs, cov 597 ft 1455, DONE, zero crashes
cargo +nightly fuzz run plane fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=20
  → 6,133,522 runs (post as_chunks fix), cov 597 ft 1485, DONE, zero crashes
```

19,184,408 combined runs, zero crashes. **How I know both paths were
reached**: the seed corpus plants one fixture per path (`valid-fourteen-bit`
exercises `unpack_bitstream`, `valid-sixteen-bit` exercises
`unpack_byte_aligned`) and libFuzzer's coverage-guided mutation explores
around every seed it starts from — `cov: 597` is stable across both runs
(not growing from one path alone), consistent with both being live inputs to
the mutator rather than one seed dominating. The `ifd` target was also
re-run (4,644,388 runs, 16s, zero crashes) to confirm the new `Error`
variants in `src/lib.rs` didn't disturb it.

### 4. AC8 — measured, not estimated

See the spec's AC8 checkbox for the number and method
(`/usr/bin/time -l target/release/irr unpack`, macOS). Restated here because
it is this handoff's own return criterion 4: **182,435,840 bytes (174 MiB)
peak RSS** for `L1021223.DNG`, 47 MP, 14-bit — accounted for by the ~86 MB
input `Vec<u8>` (I/O `irr` does, the library never does) plus the
94,887,936-byte plane `irr` allocates as `unpack_into`'s caller (`DEC-016`).

### 5. Mutation discipline

No `git checkout --`/mutate-and-revert experiment on committed source was
needed for this spec's red-proof — unlike `lint-red-proof.sh` (which mutates
a *copy* of `src/lib.rs` in a temp dir, not this tree), `SPEC-012`'s
red/green pair for the `WhiteLevel` assertion is two **fixtures**, not a
mutation of the unpacker itself: `each_path_produces_impossible_values...`
(no `WhiteLevel` tag → wrong values decode silently) and
`a_plane_whose_max_exceeds_white_level_is_an_error` (same wrong-path fixture,
`WhiteLevel: Some(16383)` added → `Error::SampleExceedsWhiteLevel`). Every
edit to `src/plane.rs`/`src/lib.rs`/the test files was made in the dedicated
worktree created for this build (never the shared checkout), committed only
once all eleven gates were green — no working-tree loss to guard against
here (`SPEC-010`'s failure mode).

### 6. Findings

**None.** No `SB-N`/`FU-N` findings from the gate run — every gate that
failed on its first attempt (§2 above) was fixed within this same build
cycle, so nothing crosses to verify undispositioned.

### 7. Reflection (AGENTS.md §15's questions, answered now rather than at ship)

1. **What would I do differently next time?** Build the `Fixture` struct in
   `tests/plane_unpack.rs` and its near-duplicate in
   `examples/fuzz-seeds.rs::plane_fixture` as one shared helper from the
   start, rather than writing the test one first and then hand-mirroring it
   for the fuzz seeds. They are kept in sync by comment today
   (`examples/fuzz-seeds.rs`'s doc comment says so explicitly) — workable,
   but a shared `tests/support/plane_fixture.rs` `#[path]`-included by both
   would remove the "kept in sync by hand" risk entirely. Didn't do it this
   round because the fixture only crystallized after the AC-by-AC test
   design was already working; noted as a shape worth revisiting if a third
   consumer needs it.
2. **Does any template, constraint, or decision need updating?** No — the
   local/CI clippy gap (§2) is `lint-ci` working exactly as designed and
   documented; nothing here reveals a new class of gap.
3. **Is there a follow-up spec I should write now before I forget?** No new
   one. `SPEC-013` (the MD5 plane oracle) and `SPEC-014` (levels/crop/
   orientation) are already framed and are the natural next specs; this
   build didn't surface anything they don't already cover.
4. **Where was the worst defect caught?** `build` — the clippy-version gap
   (§2), caught by `just lint-ci` before push, per its documented job.
5. **What can a user do now that they couldn't before?** Before: `irradiance`
   located the sensor strip as tags (`StripOffsets`/`StripByteCounts`) but
   read no pixel from it. After: a caller with the file bytes and a
   `width × height` buffer gets a bit-exact linear `u16` plane for the 4 of 7
   corpus files this library can decode — confirmed against `dnglab`'s own
   plane on both bit depths held (`[746, 725, 711, 752, 646, 705, 772, 686]`
   for the Q2M 14-bit file; `[4761, 4591, 4622, 4363, 4542, 4383, 4608,
   4286]` for the M Monochrom 16-bit file).
