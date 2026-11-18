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
  id: HANDOFF-031
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # CONFIRMED at verify (2026-09-05), not corrected:
                                   #   message.model reads claude-opus-5 on all 136
                                   #   usage objects in this session's transcript, so
                                   #   tier_map.verify's hint was RIGHT. The 0-for-7
                                   #   record is the BUILD lane's; verify is now 2 for 2.
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  tokens_total: 8200000            # deduped by message.id, 62 distinct messages, all claude-opus-5
  estimated_usd: 20.50             # per-component: $15/M in, $75/M out, $1.50/M cache-read, $30/M cache-write-1h
  duration_minutes: 75
  branch: feat/spec-013-bit-exact-plane-oracle-red-proof
  pr: null                         # not opened, per this handoff's return criterion 7
  completed_at: 2026-09-05
  notes: "VERDICT: APPROVED at 88cc343 -- 4 follow-ups, 0 ship-blockers. Code at the branch tip 4a5ce43 is IDENTICAL to 88cc343 (the delta is one handoff doc), so every measurement is against the approved code; src/, Cargo.toml and Cargo.lock are 0 lines changed vs main, reproduced not inherited. RAN MYSELF: eleven gates + lint-ci all green, summed across all eight targets -- test 120 passed 0 failed (52 lib + 0 irr + 9 corpus_manifest + 12 ifd_reader + 30 metadata_oracle + 10 plane_oracle + 7 plane_unpack + 0 doc) with ZERO SKIP lines, so tier B genuinely executed; fmt; clippy 0.1.97; lint-ci FORCE-RELINTED under 0.1.98 (88d9e12ae1, CI's floating stable, version asserted, not taken from cache); lint-no-allow; lint-red-proof (control clean -> injection rejected 101 -> all five lints fired); typecheck; build --release; msrv 1.90.0; deny; deny-fuzz; fuzz ifd 11,549,344 runs and fuzz-plane 14,400,795 runs, 60s each, zero crashes, seed corpus byte-unchanged (32 files, md5 b97a26cf255bd87b22a235cbcdcaaa48 before and after) and zero artifacts. CI OBSERVED GREEN on the APPROVED SHA: run 33945319141, headSha 88cc343, ALL 9 JOBS including rust/test; also 9/9 on the tip 4a5ce43 (33951250138). validate 17 artifacts; cost-audit clean; decisions-audit 0 structural errors, the 4 scope warnings pre-date this spec and DEC-010/DEC-017 is nesting not conflict. RED-PROOF WATCHED FAILING, digests reproduced independently: honest=cb653b5bec24d166eef2fd258ee61ac4 mutant=59b032fe4320a27989ce61f3e3da7ff2 on L1021223.DNG, and the tree was byte-identical afterwards (git status empty, src/plane.rs md5 2b86d470b26ed0bd548380ac0a5943cf). SIX MUTATIONS, each asserted to change the file AND compile AND change the output, tree restored and md5-verified after every one: (M1) injection made a NO-OP -> the red-proof FAILS with its own anti-no-op message, so the third clause is live and the rebuild genuinely tracks the injected source. (M2) injection made non-compiling -> loud 'cargo build --release failed' panic, so the apparatus cannot no-op via a silent build failure; the temp dir is fresh per run so no stale artifact can be reused. (M3) negative control staged MUTATED -> FAILS, reporting the mutant digest, so the control is load-bearing. (M4) L1000622.DNG moved from DECODABLE into SKIPPED_COMPRESSED with a fabricated reason -> oracle coverage silently drops 4 files to 3 and all 10 tests still pass (FU-2). (M5) the red-proof's own fault injected into the REAL src/plane.rs with NO corpus -> hand_built_fixtures_plane_matches_its_known_md5 goes RED, so CI's tier-A half DOES catch a broken bit-packed unpacker -- this materially corrects HANDOFF-031's framing in the build's favour. (M6) 16-bit byte-aligned endianness swapped with NO corpus -> two plane_unpack tests go red, so that path is pinned corpus-free too. BEYOND THE HANDOFF'S CHECKS: AC3's locator exercised on a REAL 47-megapixel mismatch for the first time (it had only ever seen 5-element arrays) -- a mutated-crate probe dumped a real wrong plane, and locate_first_difference against dnglab's own --raw-pixel plane named 'index 0: ours=744 dnglab=746', with 31,594,155 of 47,443,968 samples differing (66.6%); assert_plane_matches's failure branch was fired end-to-end on the real 94.9 MB plane and its message is well-formed. AND THE STRONGEST RESULT, which no cycle had measured: our honest plane agrees with dnglab's SAMPLE-FOR-SAMPLE across all 47,443,968 samples (locate_first_difference -> None), a strictly stronger statement than the MD5 match and a second independent oracle route. MD5 cross-checked far beyond AC1's seven RFC vectors: 142 input lengths (0..130 plus every padding-cliff case 55/56/57/63/64/65 and multi-block) all identical to system md5, plus four real DNGs 36-86 MB each, plus irregular-chunk streaming matching one-shot on all four. PROVENANCE VERIFIED NOT ASSUMED: all 64 K constants independently reproduce from RFC 1321's own generating formula floor(abs(sin(i+1))*2^32), and SHIFT and the message-word index sequences match Sec 3.4's round tables, so class 1 -- specification is defensible from the artifact itself. FINDINGS, all follow-up, none ship-blocking: FU-1 the red-proof passes vacuously where CI runs it (corpus absent -> 10/10 pass in 0.01s, and CI runs cargo test WITHOUT --nocapture so the SKIP text is captured) -- but TWO mitigations I measured cut it down: CI's uncaptured corpus-status step prints '0/7 present ... tier-B tests will SKIP' so the vacuity is inferable rather than silent, and M5 shows CI's tier-A half actually catches a broken unpacker. CI therefore has PROTECTION but no PROOF of it. Cost to close, measured: a corpus-free red-proof over the hand-built fixture already in the file goes red (honest d1d83299c631541fac68da1051b19a23, mutant 6aa91ec5ca43d50e25e9d9013cae358e) in 1.47s for BOTH cold builds, reusing hand_built_fixture, stage_probe_crate, build_and_run_probe and FIXTURE_PLANE_MD5 -- so 'DEC-003 means CI can never run it' is true only of a CORPUS file. FU-2 compressed_files_are_skipped_by_name proves the UNION is complete, never the PARTITION (M4). FU-3 the red-proof covers one of DEC-008's two paths -- the injected fault leaves L1000622.DNG's digest byte-identical -- but DEC-017's Validation already anticipates this in writing and M6 shows the path is pinned corpus-free elsewhere. FU-4 doc drift, md5.rs:19 names PROBE_MD5_SOURCE, the constant is MD5_SOURCE. Did NOT run handback-sync, did NOT open the PR, committed nothing, merged nothing. tokens_total is a transcript sum DEDUPED BY message.id from this session's own JSONL (83ded79b-0b78-4b86-80ec-484720d47113.jsonl): 136 usage objects / 62 distinct ids, all claude-opus-5. Measured floor at time of writing 7,374,343 (input 124 / output 44,028 / cache_read 7,184,654 / cache_write_1h 145,537, 5-minute tier zero), priced PER-COMPONENT at opus rates ($15/$75/$1.50/$30 per M) = $18.45; rounded UP to 8,200,000 / $20.50 to cover the turns spent writing this handback, per HANDOFF-020's precedent."
  synced_at: 2026-09-05
---

# HANDOFF-031: Verify SPEC-013 — the plane oracle and its red-proof, at `88cc343`

## Delegation Summary

Verify `SPEC-013` at **`88cc343`** on `feat/spec-013-bit-exact-plane-oracle-red-proof`
(pushed, not merged; `main` at `9f269ed`). **This is a strong build — verify it
on that basis.** The risk is not sloppiness; it is a well-made oracle with a
coverage gap.

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + CI green on three SHAs | ✅ `f162a39`, `1f1bbbc`, `905a68a`, `88cc343` |
| `src/`, `Cargo.toml`, `Cargo.lock` untouched | ✅ `git diff main...HEAD` is **0 lines** on all three |
| 120 tests, 0 failed | ✅ summed, corpus present |
| **the red-proof works** | ✅ **run by the orchestrator**, watched: `honest=cb653b5bec24d166eef2fd258ee61ac4 mutant=59b032fe4320a27989ce61f3e3da7ff2` |
| the red-proof leaves the tree untouched | ✅ `git status` empty and `git diff HEAD` empty **after** running it |

⚠ **Credit where it is due, and it is a design the reviewer should understand
before critiquing:** the red-proof mutates a **temp-dir copy** of the crate and
rebuilds *that*, so the working tree is never touched and the whole thing runs in
**10.5 s**. The design session's own probe rebuilt in place, took minutes, timed
out twice, and left a stale process holding a mutated `src/plane.rs`. This is
strictly better than what the spec asked for.

**And the build did the thing the spec was written to force.** Its *first*
candidate fault changed the file, compiled, and produced `Error::Truncated`
rather than a wrong digest — because the strip is packed with **zero slack**
(`width × height × bits == StripByteCounts × 8` exactly), so any constant
additive shift runs one bit past the buffer on the final sample. It rejected
that fault and recorded why in `DEC-017`. That is `AC4`'s third clause working
on its first use.

## ⚠ The finding to confirm or kill — the orchestrator's, measured

**The red-proof passes vacuously where CI runs it.**

```
corpus present : an_injected_unpacker_fault_turns_the_oracle_red ... ok   (10.50s)
corpus absent  : all 10 tests "pass"                                      (0.00s)
```

`AC5` asked for a tier-A half and **got a real one** — the RFC vectors, the
streaming check, a hand-built fixture plane with a known digest, the locator, and
two PGM-parser tests all do genuine work with no corpus and no tools. That
criterion is met.

**But the red-proof is not in it.** The half CI can see contains no proof that
the oracle can fail. `constraints.yaml` was amended at STAGE-001's close to say
*a job that exists and has never passed is a deleted job*; the sibling case is a
**red-proof that exists and never runs**, and this project has now met that shape
four times (`SPEC-005/FU-3`, `SPEC-010/F-b`, `SPEC-012`, here).

Judge it. It is arguably **not** a defect — the red-proof genuinely works for
anyone holding the corpus, and `DEC-003` means CI can never hash a real plane.
But if it is acceptable, say so **with the reason**, because the alternative is
that a tier-A red-proof over the hand-built fixture is cheap and nobody thought
to ask for it.

## Your own checks

1. **Does the rebuild actually rebuild?** `DEC-017`'s mechanism copies, mutates,
   and rebuilds in release mode. If that rebuild silently failed or a stale
   artifact were reused, the test would compare a digest against itself. **Break
   the rebuild deliberately and confirm the test notices** — a red-proof whose
   apparatus can no-op is the exact defect `SPEC-013` exists to prevent, one
   level up.
2. **Is `the_honest_tree_is_the_negative_control` load-bearing?** Mutate it and
   see what dies. A control that cannot fail is not a control.
3. **Is `a_mismatch_names_the_first_differing_sample` exercised on a REAL
   mismatch**, or only a synthetic one? `AC3` exists because `SPEC-014` will
   debug 47 megapixels against this.
4. **MD5 beyond the RFC vectors.** Seven published vectors are the floor.
   Cross-check the implementation against the system `md5`/`md5sum` on something
   large and irregular — the corpus planes are right there.
5. **`compressed_files_are_skipped_by_name`** — does it assert the *reason*, or
   just that three files were skipped?

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Observe CI green on the SHA you approve.**
2. **Watch the red-proof fail yourself** (§15 check 9) and paste **both digests**.
3. **Fuzz** (§15 check 10) — `tests/` gained a lane; seeds unchanged is a fine
   result, say so.
4. **Provenance (§15 check 11):** MD5 row, class 1, RFC 1321, written from the
   standard and not from an implementation. Confirm it.
5. Every mutation: file changed **and** compiled **and** *output changed*. Stage
   your work before mutate-and-revert.
6. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports. ⚠ **Do not hand-write `cost.sessions`** — fill the
   handback block only, so `handback-sync` runs once cleanly. Hand-writing it has
   caused four duplicate-entry cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

**✅ APPROVED at `88cc343`.** 4 follow-ups, 0 ship-blockers.

The build is what the handoff said it was: a well-made oracle. Every claim in the
reconciliation table reproduced, and nothing was taken on report. `src/`,
`Cargo.toml` and `Cargo.lock` are **0 lines changed** against `main`. Eleven gates
+ `just lint-ci` green, 120 tests / 0 failed summed across all eight targets with
**zero SKIP lines**. CI observed green **9/9 on `88cc343` itself** (run
`33945319141`), and 9/9 on the tip `4a5ce43`, whose only delta is this document.

**Red-proof watched failing, digests reproduced independently:**

```
RED-PROOF (LEICA-Q2-MONO/L1021223.DNG):
  honest=cb653b5bec24d166eef2fd258ee61ac4  mutant=59b032fe4320a27989ce61f3e3da7ff2
```

`git status` empty and `src/plane.rs` byte-identical afterwards.

### The four checks the handoff asked for

1. **Does the rebuild actually rebuild?** Yes, proven three ways. Making the
   injection a **no-op** turns the red-proof red with its own anti-no-op message
   — so `assert_ne!` is live and the mutant digest genuinely tracks the injected
   source. Making the injection **not compile** produces a loud
   `cargo build --release failed` panic. And the temp dir is created fresh per
   run with no `target/`, so no stale artifact can be reused. **The apparatus
   cannot no-op.**
2. **Is the negative control load-bearing?** Yes. Staging it *mutated* fails it,
   and it reports the mutant digest — it distinguishes "failed for my reason"
   from "failed for any reason", which is what `DEC-009` asks of a control.
3. **Is the locator exercised on a REAL mismatch?** It was **not** — it had only
   ever seen 5-element arrays. **So I exercised it.** A mutated-crate probe
   dumped a genuinely wrong 47-megapixel plane; `locate_first_difference` against
   dnglab's own `--raw-pixel` plane returned `index 0: ours=744 dnglab=746`, with
   **31,594,155 of 47,443,968 samples differing (66.6%)**. `assert_plane_matches`'s
   failure branch was also fired end-to-end on the real 94.9 MB plane; its message
   is well-formed. **AC3 is met.** And the by-product is the strongest result of
   this review, which no cycle had measured: our honest plane agrees with
   dnglab's **sample-for-sample across all 47,443,968 samples** — strictly
   stronger than the MD5 match, via a second independent oracle route.
4. **MD5 beyond the RFC vectors.** Cross-checked against system `md5`: **142
   input lengths** (0..130 plus every padding-cliff case 55/56/57/63/64/65 and
   multi-block) all identical, **four real DNGs of 36–86 MB** all identical, and
   irregular-chunk streaming matching one-shot on all four. Separately, all **64
   `K` constants independently reproduce** from RFC 1321's own generating formula
   `floor(abs(sin(i+1)) × 2³²)`, and `SHIFT` and the message-word index sequences
   match §3.4's round tables — so the provenance row's **class 1 — specification**
   is verifiable from the artifact itself, not merely asserted.

### On the finding the handoff asked me to confirm or kill

**Confirmed as a follow-up (`FU-1`), and narrowed.** The vacuity is real: corpus
absent → 10/10 pass in 0.01 s, and CI runs `cargo test` **without** `--nocapture`,
so the in-harness SKIP text is captured and invisible. But two mitigations I
measured cut the finding down, and both belong in the record:

- CI's **uncaptured** `corpus-status` step runs ahead of the tests and prints
  `0/7 present … tier-B tests will SKIP`. The vacuity is **inferable, not
  silent** — `SPEC-002` built exactly this surface for exactly this reason.
- **CI's tier-A half actually catches a broken unpacker.** Injecting the
  red-proof's own fault into the real `src/plane.rs` with **no corpus** turns
  `hand_built_fixtures_plane_matches_its_known_md5` **red**. This corrects the
  handoff's framing in the build's favour: the half CI sees is not inert.

So the accurate statement is not "CI cannot see the oracle fail" but **CI has
protection without proof of that protection** — and proof is precisely what
`oracle-must-be-shown-red` exists for.

**And the handoff's own hypothesis is right, measured:** a corpus-free red-proof
over the hand-built fixture *already in the file* goes red — honest
`d1d83299c631541fac68da1051b19a23`, mutant `6aa91ec5ca43d50e25e9d9013cae358e` —
in **1.47 s for both cold builds**, reusing `hand_built_fixture`,
`stage_probe_crate`, `build_and_run_probe` and `FIXTURE_PLANE_MD5`. So
"`DEC-003` means CI can never run it" is true only of a **corpus** file. Nobody
thought to ask, and it costs about fifteen lines.

### Findings

| id | finding | label | recommended disposition |
|---|---|---|---|
| `FU-1` | The red-proof never executes where CI runs it — corpus absent, 10/10 pass in 0.01 s, SKIP text captured. Mitigated by the uncaptured `corpus-status` step and by the tier-A fixture test genuinely catching a broken unpacker (measured), so CI has protection but no proof of it. A corpus-free red-proof over the existing hand-built fixture goes red in **1.47 s**. Fifth instance of `named-tests-can-pass-vacuously`. | follow-up | `fixed` at ship — ~15 lines reusing four functions already present. If ship declines, `signal: named-tests-can-pass-vacuously` with this evidence; **not** `closed`, whose trigger would be someone remembering. |
| `FU-2` | `compressed_files_are_skipped_by_name` proves the **union** is complete, never the **partition**. Moving `L1000622.DNG` from `DECODABLE` into `SKIPPED_COMPRESSED` with a fabricated reason drops oracle coverage 4 files → 3 with the whole suite still green. The test asserts the reason is *non-empty*, never that it is *true* — and the file that can be dropped is the only one exercising `unpack_byte_aligned`. | follow-up | `fixed` at ship if cheap (assert each `SKIPPED_COMPRESSED` entry actually fails to decode, tier B), else `spec:` — but note no existing AC fails without it, so `signal:` is the honest fallback. |
| `FU-3` | The red-proof covers one of `DEC-008`'s two paths. Measured: the injected fault leaves `L1000622.DNG`'s digest **byte-identical** (`b0f602b9…` both), so it says nothing about `unpack_byte_aligned`. Well mitigated — `DEC-017`'s Validation anticipates this in writing, and swapping that path's endianness turns two `plane_unpack` tests red **with no corpus**. | follow-up | `closed` — the close's trigger is a test that already fails (`each_path_produces_impossible_values_on_the_others_data`), which `AGENTS.md` names as a good close. Or `spec:` if `SPEC-014` wants a second fault. |
| `FU-4` | Doc drift: `tests/support/md5.rs:19` names the constant `PROBE_MD5_SOURCE`; it is `MD5_SOURCE` (`tests/plane_oracle.rs:633`). | follow-up | `fixed` — one word. |

### Method note

Six mutations, each asserted to **change the file** *and* **compile** *and*
**change the output**, with the tree restored and md5-verified after every one
(`src/plane.rs` `2b86d470b26ed0bd548380ac0a5943cf`, `tests/plane_oracle.rs`
`b073286507f21ff062ffbec81aacac1e`). Peak RSS was not a problem: the tier-B
oracle decodes the four files sequentially, one plane buffer live at a time.
`handback-sync` **not** run, PR **not** opened, nothing committed, `cost.sessions`
**not** hand-written — the `handback:` block above is filled and left for
`handback-sync`.
