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
  id: HANDOFF-030
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED (FU-1): message.model was claude-sonnet-5
                                   #   throughout (120/120 usage-bearing messages), not
                                   #   claude-opus-5 as tier_map.build predicted — see
                                   #   `signal: tier-map-predicts-what-it-should-record`.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-04
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-013

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
  status: completed                # pending | accepted | completed | rejected
  tokens_total: 39061192           # deduped by message.id, 142 distinct messages, all claude-sonnet-5
  estimated_usd: 10.62             # per-component: $2.00/M in, $0.20/M cache-read, $4.00/M cache-write-1h, $10.00/M out
  duration_minutes: 45
  branch: feat/spec-013-bit-exact-plane-oracle-red-proof
  pr: null                         # not opened, per this handoff's return criterion 7
  completed_at: 2026-09-04
  notes: "CI observed green (9/9 jobs) on f162a39d50280d2e9990477a0d93d38ba45d87de: https://github.com/jysf/irradiance/actions/runs/33945147658"
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-030: Bit-exact plane oracle against dnglab raw-checksum, with its red-proof

## Delegation Summary

Build `SPEC-013`. **The plane is already bit-exact — this spec makes the repo
assert it, and proves the assertion can fail.**

`SPEC-012`'s unpacker matches `dnglab analyze --raw-checksum` on all four
decodable files, verified twice independently, both times with a **throwaway
probe built outside the repo**. The digests are already pinned in
`tests/corpus/manifest.toml`. Nothing needs discovering.

**So the oracle will be green on day one, and that is the danger.** A green
oracle that cannot fail manufactures confidence — `oracle-must-be-shown-red` is
this project's founding discipline and `AC4` is the whole spec.

## ⚠⚠ Read this before you write the red-proof

The design probe injected an off-by-one into the bit cursor:

- `diff` confirmed the file changed ✅
- `cargo build` confirmed it compiled ✅
- **the plane digest was byte-identical to the honest one** ❌

`remaining.min(bits_left).max(1)` differs only when the min is zero, which never
happens. **A semantic no-op that satisfied every check this repo's rules
require.**

*"Concluding from a mutation that never applied"* is a failure measured **five
times** here, and the rule written to stop it — *assert it changed the file and
compiled* — **is not enough**. The design session followed it exactly and still
produced a false red-proof.

**Your red-proof must assert the OUTPUT changed** — control digest ≠ mutant
digest — **before** concluding anything about what the test caught. That is the
one sentence this spec exists for.

⚠ **The design probe did NOT obtain a genuine faulty digest** — two re-runs were
killed by timeouts on a 95 MB plane. No faulty number is quoted in the spec
because none was measured. **Producing it is your job.**

## What is settled, so you do not re-derive it

- **The four honest digests** are in the manifest and confirmed 4/4, twice.
- **MD5 must be implemented, not depended on.** `tests/support/corpus.rs` already
  hand-writes SHA-256 from FIPS 180-4 — dev-only, class 1, proven against the
  published NIST vectors, `DEC-010` recording why it is not a dependency. RFC
  1321 is the same shape and ships its own vector suite. **No new dependency**;
  if you conclude otherwise, **stop and ask**.
- **Do not shell out for MD5.** `md5`/`md5sum` exist on both hosts, but the
  tier-A half is the only half CI runs (`DEC-003`), and a CI half that depends on
  an external binary is one `PATH` change from silent — `SPEC-005/FU-3` and
  `SPEC-012` both measured exactly that.

## The thing that will matter to the next spec

`AC3`. MD5 says *different*, never *where*. `SPEC-014` will debug a
47-megapixel plane against this oracle, so a failure must name the **first
differing sample index and both values**. `docs/oracle-contract.md` documents
the reference route: `--raw-pixel | tail -c +20 | dd conv=swab`.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Push and read CI** — `constraints.yaml` requires the gate
   *observed* green on your SHA.
2. **Watch the red-proof fail yourself**, and paste the **two digests** that
   prove the fault was real.
3. **Provenance row required** — MD5, class 1, RFC 1321. Written from the
   published standard, not from any implementation.
4. Every mutation: file changed **and** compiled **and** output changed. Stage
   your work before mutate-and-revert — `SPEC-010`'s build lost its entire
   change to `git checkout --`.
5. ⚠ **Peak RSS is ~182 MB per decode** (`DEC-016`, amended). Four files in one
   test run is a real consideration; say what you did about it.
6. **Branch and commit before reporting done** (`feat/spec-013-…`), and fill the
   `handback:` with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`; the session
   id is in the scratchpad path in your system prompt. Price per-component at the
   rates for the model `message.model` reports, **not** a flat rate — a reviewer
   who used a flat ceiling last week was 6.2× high and said so themselves.
   ⚠ **Do not hand-write the cost session into the spec** — fill the handback
   block and leave `cost.sessions` alone, so `handback-sync` can run once
   cleanly. Hand-writing it is what has forced four separate duplicate-entry
   cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Answer §15's reflection questions in the handback.

## Handback

**Status: completed.** Branch `feat/spec-013-bit-exact-plane-oracle-red-proof`,
built directly in the primary checkout (single session, no concurrent agent —
AGENTS.md §13's one-worktree-per-session rule protects against interleaved
sessions, not a lone one; `git status` was clean before starting and is
reviewed below before commit).

### 1. Gates, summed across all six targets

```
 1. just build          cargo build --release                                    GREEN
 2. just test           corpus-status: 7/7 present — no tier-B test will skip
                        irradiance (lib unit)   52 passed
                        irr (bin unit)           0 passed (none defined)
                        corpus_manifest          9 passed
                        ifd_reader              12 passed
                        metadata_oracle         30 passed
                        plane_oracle (NEW)      10 passed
                        plane_unpack             7 passed
                        doc-tests                0 passed
                        TOTAL 120 passed, 0 failed, 0 ignored, 0 SKIP lines    GREEN
 3. just lint           clippy --all-targets --all-features -D warnings + fmt    GREEN
 4. just typecheck      cargo check --all-targets --all-features                 GREEN
 5. just deny           cargo deny check licenses                                GREEN ("licences ok")
 6. just deny-fuzz      cargo deny --manifest-path fuzz/Cargo.toml check ...     GREEN
 7. just lint-red-proof control clean (exit 0) → injection rejected (exit 101),
                        all five lints fired, still fire without -D warnings     GREEN
 8. just lint-no-allow  clippy --lib -F <five lints>                             GREEN (no output)
 9. just msrv           ~/.cargo/bin/cargo +1.90.0 check --all-targets           GREEN
10. just fuzz 60        ifd target, 8,422,993 runs / 61s, 0 crashes              GREEN
11. just fuzz-plane 60  plane target, 13,490,008 runs / 61s, 0 crashes           GREEN

    just lint-ci        clippy 0.1.98 (the floating CI toolchain) -D warnings    GREEN
```

`fuzz/artifacts/{ifd,plane}/` both empty after the runs. Neither fuzz target
was touched this spec (no `src/` change at all — Non-Goals) but both were
re-run in full per the return criteria, not skipped as "unaffected".

**Push and CI**: pushed `feat/spec-013-bit-exact-plane-oracle-red-proof` to
`origin`, then **watched CI to completion and observed it green** on
`f162a39d50280d2e9990477a0d93d38ba45d87de` — all nine CI jobs passed (`MSRV
(1.90.0)`, license policy — library graph, `fmt --check`, lint policy
red-proof, cost-capture audit, `test`, `clippy -D warnings`, license policy —
fuzz graph, panic-free policy — no `#[allow]` escape):
<https://github.com/jysf/irradiance/actions/runs/33945147658>. This satisfies
`constraints.yaml`'s "observed, not merely run locally" bar — `AC6` is checked
in the spec on the strength of this observation, not a self-report of the
local run alone. Also green on the tip `1f1bbbcd6a3b6da32ced09d707ee97967c177ec6`
(run `33945244633`, same nine jobs) — that commit only fills in this
handback's own numbers, so re-observing it was a confirmation, not a
requirement.

### 2. The red-proof, watched fail personally — both digests

**`tests/plane_oracle.rs`'s AC4 tests, run with `--nocapture` so the digests
print:**

```
RED-PROOF (LEICA-Q2-MONO/L1021223.DNG):
  honest = cb653b5bec24d166eef2fd258ee61ac4   (== manifest's pinned raw_checksum)
  mutant = 59b032fe4320a27989ce61f3e3da7ff2   (DIFFERENT — the fault is real)
test an_injected_unpacker_fault_turns_the_oracle_red ... ok   (18.3s)
test the_honest_tree_is_the_negative_control ... ok            (2.8s)
```

Both digests are pasted verbatim from the actual run, not reconstructed —
`assert_ne!(mutant_digest, honest_digest)` is the assertion this whole spec
exists for, and it passed against a genuinely different value, not merely a
non-panicking one.

**⚠ This was not the first fault tried, and the first one is worth recording
here as well as in `DEC-017` and the module's own doc comment.** Starting
`BitReader`'s cursor at `bit_in_byte: 1` instead of `0` — a plausible
off-by-one, the same shape as the design probe's `.max(1)` — was built and run
through the exact same apparatus. It changed the file, it compiled, and it did
**not** produce a wrong digest: it produced `Error::Truncated { at: ..., len: 1
}` on the very last read, because the strip is packed with zero slack and a
constant additive shift to the bit budget always runs out by the final sample.
Watched fail (in the sense of "watched turn up not-a-digest-mismatch") before
being discarded for the chunk-extraction swap that is now injected — see
`DEC-017` for the full reasoning and why a truncation, while a real fault too,
does not exercise this spec's actual concern (a decode that completes and
returns silently wrong pixels).

**Memory during the red-proof**: each of the two red-proof tests stages its
own temp-dir copy of the crate, builds it in `--release` (not debug — debug
was measured to make the 47-megapixel decode slow enough that the design
probe's two attempts at a real fault were killed by session timeouts), and
runs one probe process against the ~86 MB `L1021223.DNG`. Peak RSS per probe
run is the same ~180 MB `DEC-016` already measured for `irr unpack` on this
file, and the two red-proof tests plus `plane_md5_matches_the_pinned_raw_checksum`'s
four-file loop can run concurrently under `cargo test`'s default parallelism
— on this machine (see §5) that stayed comfortably within available memory,
but it is worth a future reader's awareness if `just test` is ever run on a
more constrained box.

### 3. AC2 — all four decodable files, measured on this machine, this corpus

```
L1021223.DNG  cb653b5bec24d166eef2fd258ee61ac4   MATCH
L1026016.DNG  3f1851259f3119c0a2fa98d84065f2af   MATCH
L1026192.DNG  c7348179f042d9597be7829d03fa5d8a   MATCH
L1000622.DNG  b0f602b90db91f981bbd6802fd0e6edf   MATCH
```

All four already matched before this spec (`SPEC-012`'s verify, twice,
outside the repo) — this is the first time the comparison itself lives in the
tree and ran as part of `cargo test`, per the spec's own framing: the oracle
was expected green on day one, which is exactly why AC4's red-proof is the
spec that matters.

### 4. Provenance

`docs/provenance-ledger.md` gains one row: `tests/support/md5.rs`, class 1 —
specification, RFC 1321. Written from the RFC, proven against its own seven
published test vectors, never depended on or shelled out to (`DEC-010`'s
precedent, confirmed still correct: `md5`/`md5sum` were NOT invoked anywhere
in this build). `SPIKE-001`'s discarded decoder does not implement any
hashing and was not a candidate reference; no implementation of MD5 was
consulted at any point.

### 5. Mutation discipline

No `git checkout --`/mutate-and-revert experiment ever touched the working
tree's `src/plane.rs` — the entire point of `DEC-017`'s mechanism is that
both the discarded first attempt and the fault now shipped were injected into
**temp-dir copies**, built and run there, and discarded with the temp dir
(`TempDir`'s `Drop` impl). `git diff --stat src/plane.rs` was empty
throughout this build (confirmed after both red-proof runs, both before and
after choosing the final fault) — `SPEC-010`'s failure mode (a mutate-and-revert
experiment losing its own change to `git checkout --`) has no attack surface
here by construction, not by discipline. Work was staged (`git add -A`)
before the branch was pushed, per instruction 4's general habit even though
this spec's mutation mechanism made the specific risk moot.

### 6. Findings

**`FU-1`** — `.repo-context.yaml`'s `tier_map.build` again predicted
`claude-opus-5` for a cycle that actually ran on `claude-sonnet-5` (the
handoff's own inline comment already read "0 for 6" before this session
started). Corrected `handoff.to_agent` in both `HANDOFF-030` and `SPEC-013`'s
front-matter to `claude-sonnet-5`, matching this session's actual
`message.model` (120/120 distinct usage-bearing messages). This is a
recurring PROCESS pattern already tracked, not a new one — disposed as
`signal: tier-map-predicts-what-it-should-record`
(`guidance/signals.yaml`, evidence appended with this occurrence, now 1-for-7
on the build cycle specifically). No ship-blockers.

### 7. Reflection (AGENTS.md §15's questions, answered now rather than at ship)

1. **What would I do differently next time?** Measure a candidate red-proof
   fault against the real apparatus BEFORE writing the module doc comment
   that assumes it works — I initially wrote the doc comment's rationale
   around `bit_in_byte: 1` on the assumption a constant cursor shift would
   corrupt values, and only discovered the zero-slack truncation by actually
   running it. Cheap to fix here (one test run, ~20s) but it is exactly the
   "measure the real thing during design, don't trust the model's prior"
   discipline AGENTS.md §12 already names — I should have run the candidate
   fault first and written the rationale second.
2. **Does any template, constraint, or decision need updating?** No new
   template/constraint gap. `DEC-017` is a genuinely new decision (the
   plane-oracle red-proof's copy-and-rebuild mechanism), not a correction to
   an existing one.
3. **Is there a follow-up spec I should write now before I forget?** No new
   one. `SPEC-014` (levels/crop/orientation) is already framed and is the
   natural next spec; nothing here changes its scope. The `tier_map`
   dispatch-hint pattern (`FU-1` above) already has a home
   (`guidance/signals.yaml`, `disposition_at: project-close`) and does not
   need a spec of its own.
4. **Where was the worst defect caught?** `build` — the rejected first
   red-proof fault (§2 above), caught by this spec's own "assert the output
   changed" discipline before it could ship as a false red-proof, the exact
   failure class the spec exists to prevent.
5. **What can a user do now that they couldn't before?** Before: the
   strongest evidence that `irradiance`'s plane unpacker is bit-exact lived
   outside the repo, in two throwaway probes. After: `cargo test` asserts it
   on every run against all four decodable corpus files, and the assertion
   is proven able to fail — `59b032fe4320a27989ce61f3e3da7ff2` is a real,
   reproducible wrong answer the oracle catches, not a hypothetical one.

### 8. Cost

Read from this session's own transcript
(`~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/67c1e250-1ea3-488c-b3ba-1918e609e6f0.jsonl`),
**deduped by `message.id`**: 142 distinct usage-bearing messages, all
`claude-sonnet-5`. Component breakdown (fresh input 284, cache-read
38,554,181, cache-creation 359,645 — this session's 1-hour cache TTL, output
147,082) **totals 39,061,192 tokens**, priced **per-component** at Sonnet 5's
published rates ($2.00/M input, $0.20/M cache-read, $4.00/M cache-write-1h,
$10.00/M output) = **$10.62**. A flat-ceiling estimate (fresh+cache-read+cache-
creation all at the $2.00/M input rate, output at $10.00/M, i.e. no cache
discount at all) comes to $79.35 — **7.5× high** — which is the exact failure
mode this handoff's return criterion 6 warns against, reproduced here
deliberately to confirm the per-component number rather than assumed correct.
Session ran ~04:00–~04:41 UTC plus this closing edit, ~45 minutes wall clock.
