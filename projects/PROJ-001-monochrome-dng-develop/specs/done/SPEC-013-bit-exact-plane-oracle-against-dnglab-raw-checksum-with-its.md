---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-013
  type: story                      # epic | story | task | bug | chore
  cycle: ship                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-sonnet-5  # CORRECTED at build (2026-09-04): tier_map.build says
                             # claude-opus-5, but this cycle actually ran on
                             # claude-sonnet-5 in a dispatched CLI session — the
                             # "0 for 6" hint this field carried was accurate.
  created_at: 2026-09-04

references:
  decisions: [DEC-003, DEC-008, DEC-010, DEC-016]                    # [DEC-NNN, DEC-MMM]
  constraints: [oracle-must-be-shown-red, provenance-recorded-per-algorithm, no-new-top-level-deps-without-decision, library-not-application]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-002, SPEC-012]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: null

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
  tokens_estimate: 18000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-04
      notes: "main-loop, not separately metered (AGENTS.md §4). Design probe produced ONE finding worth more than the number it was chasing: an injected off-by-one CHANGED THE FILE and COMPILED and was a SEMANTIC NO-OP — the plane digest came back byte-identical. remaining.min(bits_left).max(1) differs only when the min is zero, which never happens. That means the rule this repo wrote after five occurrences of 'concluding from a mutation that never applied' — assert it changed the file and compiled — IS NOT SUFFICIENT, and the design session followed it exactly while producing a false red-proof. AC4 therefore requires a third clause: assert the OUTPUT changed, control digest != mutant digest, before concluding anything. ⚠ NOT MEASURED: a genuine faulty digest. Two re-runs were killed by session timeouts on a 95 MB plane; no faulty number is quoted in the spec because none was obtained, and producing it is the build's job. Also settled: MD5 must be implemented from RFC 1321 rather than depended on or shelled out to, following sha256's FIPS 180-4 precedent exactly, because the tier-A half is the only half CI runs."

    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 39061192
      estimated_usd: 10.62
      duration_minutes: 45
      recorded_at: 2026-09-05
      notes: "CI observed green (9/9 jobs) on f162a39d50280d2e9990477a0d93d38ba45d87de: https://github.com/jysf/irradiance/actions/runs/33945147658"
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 8200000
      estimated_usd: 20.50
      duration_minutes: 75
      recorded_at: 2026-09-05
      notes: "VERDICT: APPROVED at 88cc343 -- 4 follow-ups, 0 ship-blockers. Code at the branch tip 4a5ce43 is IDENTICAL to 88cc343 (the delta is one handoff doc), so every measurement is against the approved code; src/, Cargo.toml and Cargo.lock are 0 lines changed vs main, reproduced not inherited. RAN MYSELF: eleven gates + lint-ci all green, summed across all eight targets -- test 120 passed 0 failed (52 lib + 0 irr + 9 corpus_manifest + 12 ifd_reader + 30 metadata_oracle + 10 plane_oracle + 7 plane_unpack + 0 doc) with ZERO SKIP lines, so tier B genuinely executed; fmt; clippy 0.1.97; lint-ci FORCE-RELINTED under 0.1.98 (88d9e12ae1, CI's floating stable, version asserted, not taken from cache); lint-no-allow; lint-red-proof (control clean -> injection rejected 101 -> all five lints fired); typecheck; build --release; msrv 1.90.0; deny; deny-fuzz; fuzz ifd 11,549,344 runs and fuzz-plane 14,400,795 runs, 60s each, zero crashes, seed corpus byte-unchanged (32 files, md5 b97a26cf255bd87b22a235cbcdcaaa48 before and after) and zero artifacts. CI OBSERVED GREEN on the APPROVED SHA: run 33945319141, headSha 88cc343, ALL 9 JOBS including rust/test; also 9/9 on the tip 4a5ce43 (33951250138). validate 17 artifacts; cost-audit clean; decisions-audit 0 structural errors, the 4 scope warnings pre-date this spec and DEC-010/DEC-017 is nesting not conflict. RED-PROOF WATCHED FAILING, digests reproduced independently: honest=cb653b5bec24d166eef2fd258ee61ac4 mutant=59b032fe4320a27989ce61f3e3da7ff2 on L1021223.DNG, and the tree was byte-identical afterwards (git status empty, src/plane.rs md5 2b86d470b26ed0bd548380ac0a5943cf). SIX MUTATIONS, each asserted to change the file AND compile AND change the output, tree restored and md5-verified after every one: (M1) injection made a NO-OP -> the red-proof FAILS with its own anti-no-op message, so the third clause is live and the rebuild genuinely tracks the injected source. (M2) injection made non-compiling -> loud 'cargo build --release failed' panic, so the apparatus cannot no-op via a silent build failure; the temp dir is fresh per run so no stale artifact can be reused. (M3) negative control staged MUTATED -> FAILS, reporting the mutant digest, so the control is load-bearing. (M4) L1000622.DNG moved from DECODABLE into SKIPPED_COMPRESSED with a fabricated reason -> oracle coverage silently drops 4 files to 3 and all 10 tests still pass (FU-2). (M5) the red-proof's own fault injected into the REAL src/plane.rs with NO corpus -> hand_built_fixtures_plane_matches_its_known_md5 goes RED, so CI's tier-A half DOES catch a broken bit-packed unpacker -- this materially corrects HANDOFF-031's framing in the build's favour. (M6) 16-bit byte-aligned endianness swapped with NO corpus -> two plane_unpack tests go red, so that path is pinned corpus-free too. BEYOND THE HANDOFF'S CHECKS: AC3's locator exercised on a REAL 47-megapixel mismatch for the first time (it had only ever seen 5-element arrays) -- a mutated-crate probe dumped a real wrong plane, and locate_first_difference against dnglab's own --raw-pixel plane named 'index 0: ours=744 dnglab=746', with 31,594,155 of 47,443,968 samples differing (66.6%); assert_plane_matches's failure branch was fired end-to-end on the real 94.9 MB plane and its message is well-formed. AND THE STRONGEST RESULT, which no cycle had measured: our honest plane agrees with dnglab's SAMPLE-FOR-SAMPLE across all 47,443,968 samples (locate_first_difference -> None), a strictly stronger statement than the MD5 match and a second independent oracle route. MD5 cross-checked far beyond AC1's seven RFC vectors: 142 input lengths (0..130 plus every padding-cliff case 55/56/57/63/64/65 and multi-block) all identical to system md5, plus four real DNGs 36-86 MB each, plus irregular-chunk streaming matching one-shot on all four. PROVENANCE VERIFIED NOT ASSUMED: all 64 K constants independently reproduce from RFC 1321's own generating formula floor(abs(sin(i+1))*2^32), and SHIFT and the message-word index sequences match Sec 3.4's round tables, so class 1 -- specification is defensible from the artifact itself. FINDINGS, all follow-up, none ship-blocking: FU-1 the red-proof passes vacuously where CI runs it (corpus absent -> 10/10 pass in 0.01s, and CI runs cargo test WITHOUT --nocapture so the SKIP text is captured) -- but TWO mitigations I measured cut it down: CI's uncaptured corpus-status step prints '0/7 present ... tier-B tests will SKIP' so the vacuity is inferable rather than silent, and M5 shows CI's tier-A half actually catches a broken unpacker. CI therefore has PROTECTION but no PROOF of it. Cost to close, measured: a corpus-free red-proof over the hand-built fixture already in the file goes red (honest d1d83299c631541fac68da1051b19a23, mutant 6aa91ec5ca43d50e25e9d9013cae358e) in 1.47s for BOTH cold builds, reusing hand_built_fixture, stage_probe_crate, build_and_run_probe and FIXTURE_PLANE_MD5 -- so 'DEC-003 means CI can never run it' is true only of a CORPUS file. FU-2 compressed_files_are_skipped_by_name proves the UNION is complete, never the PARTITION (M4). FU-3 the red-proof covers one of DEC-008's two paths -- the injected fault leaves L1000622.DNG's digest byte-identical -- but DEC-017's Validation already anticipates this in writing and M6 shows the path is pinned corpus-free elsewhere. FU-4 doc drift, md5.rs:19 names PROBE_MD5_SOURCE, the constant is MD5_SOURCE. Did NOT run handback-sync, did NOT open the PR, committed nothing, merged nothing. tokens_total is a transcript sum DEDUPED BY message.id from this session's own JSONL (83ded79b-0b78-4b86-80ec-484720d47113.jsonl): 136 usage objects / 62 distinct ids, all claude-opus-5. Measured floor at time of writing 7,374,343 (input 124 / output 44,028 / cache_read 7,184,654 / cache_write_1h 145,537, 5-minute tier zero), priced PER-COMPONENT at opus rates ($15/$75/$1.50/$30 per M) = $18.45; rounded UP to 8,200,000 / $20.50 to cover the turns spent writing this handback, per HANDOFF-020's precedent."
    - cycle: ship
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-05
      notes: "main-loop, not separately metered (AGENTS.md §4). Ship: ran the red-proof myself before accepting it (honest/mutant digests reproduced, tree byte-identical after), then closed FU-1, FU-2 and FU-4, red-proofing each fix in its FAILING direction — the corpus-free red-proof takes the suite from 10 tests/0.01s to 12/0.91s with no corpus, and FU-2's fix fails on the reviewer's exact mutation. ⚠ My first FU-2 mutation did NOT compile (fixed-size arrays) and was caught by the assert-it-compiled clause before any conclusion — the second save from that clause in this spec."
  totals:
    tokens_total: 47261192
    estimated_usd: 31.12
    session_count: 4
shipped_at: 2026-09-05
---

# SPEC-013: Bit-exact plane oracle against dnglab raw-checksum, with its red-proof

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-013]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

**The plane is already bit-exact. This spec makes that a fact the repo asserts
rather than one two sessions discovered by hand.**

`SPEC-012` shipped an unpacker whose whole-plane MD5 matches
`dnglab analyze --raw-checksum` on all four decodable corpus files — verified
**twice, independently**, by the orchestrator before verify ran and by the
reviewer during it. Both used a **throwaway probe built outside the repo**,
because `SPEC-012` deliberately scoped the oracle here.

That is the debt this spec pays: the strongest evidence the project has lived
outside the tree twice, and `SPEC-012`'s own reflection names it as the thing to
do differently. The digests are already pinned in `tests/corpus/manifest.toml`
(since `SPEC-002`) — nothing needs discovering, only wiring and red-proofing.

## Goal

Compare our unpacked plane's MD5 against the manifest's pinned `raw_checksum` on
every run, **prove the comparison can go red**, and make a mismatch **locatable**
rather than merely detectable.

## Inputs

- `src/plane.rs` (`unpack_into`), `tests/support/corpus.rs` — especially its
  hand-written `sha256`, which is the precedent this spec follows
- `docs/oracle-contract.md` — the plane contract, and the `--raw-pixel` route
  that makes a mismatch locatable
- `tests/corpus/manifest.toml` — `[file.oracle] raw_checksum`, already pinned
- `DEC-010` (why a hash is implemented, not depended on), `DEC-016`

## Outputs

- **MD5 in `tests/support/`** — dev-only, never in the library
- `tests/plane_oracle.rs` — the oracle and both red-proof halves
- **A provenance-ledger row** for MD5: class **1 — specification**, RFC 1321
- `docs/oracle-contract.md` gains the "this is now a test" note

## Acceptance Criteria

- [x] **AC1 — MD5 is implemented from RFC 1321 and proven against its own
      published test vectors** (the RFC ships a suite: `""` →
      `d41d8cd98f00b204e9800998ecf8427e`, `"abc"` →
      `900150983cd24fb0d6963f7d28e17f72`, and five more). Dev-only, in
      `tests/support/`. ⚠ **This follows `sha256`'s precedent exactly** — written
      from the published standard, not from any implementation, class 1, with
      `DEC-010` already recording why a hash is not a dependency. **No new
      dependency** was added. Confirmed: `tests/support/md5.rs` (dev-only,
      never in `src/`), all seven RFC 1321 Appendix A.5 vectors pass in
      `md5_matches_the_rfc_1321_test_vectors` (tier A), plus
      `md5_streaming_matches_one_shot` mirroring `sha256`'s own
      split-across-a-block-boundary discipline.
- [x] **AC2 — the oracle runs on all four decodable files** and compares
      `md5(plane)` to the manifest's `raw_checksum`. The three compressed files
      are **skipped by name with a stated reason**, not silently. Confirmed:
      `plane_md5_matches_the_pinned_raw_checksum` (tier B) decodes and hashes
      all four `DECODABLE` entries against their pinned `raw_checksum` — **all
      four match, on this machine, this corpus**:
      `L1021223.DNG cb653b5bec24d166eef2fd258ee61ac4`,
      `L1026016.DNG 3f1851259f3119c0a2fa98d84065f2af`,
      `L1026192.DNG c7348179f042d9597be7829d03fa5d8a`,
      `L1000622.DNG b0f602b90db91f981bbd6802fd0e6edf`.
      `compressed_files_are_skipped_by_name` (tier A) asserts `DECODABLE` +
      `SKIPPED_COMPRESSED` (each carrying its own reason: JPEG/SOF-3 or vendor
      PEF) together account for all 7 `[[file]]` entries, so a manifest entry
      falling through both lists fails loudly rather than vanishing.
- [x] **AC3 — a mismatch is LOCATABLE.** On failure the oracle reports the
      **first differing sample index and both values**, not "digests differ".
      MD5 says *different*, never *where*, and `SPEC-014` will debug a 47 MP
      plane against this. `docs/oracle-contract.md` documents the reference
      route: `--raw-pixel | tail -c +20 | dd conv=swab`. Confirmed:
      `locate_first_difference` (pure, tier A,
      `a_mismatch_names_the_first_differing_sample`) finds the first
      disagreeing sample and both values in a same-shaped pair of planes;
      `parse_raw_pixel_pgm` (tier A, `dnglab_raw_pixel_pgm_parses` /
      `dnglab_raw_pixel_pgm_rejects_malformed_input`) reproduces the reference
      route above, verified against the doc's own endianness proof (`02 EA` →
      746). `assert_plane_matches` wires both into the tier-B oracle's failure
      path, so a real mismatch (none observed — the oracle is green on all
      four files) would name the sample rather than print two opaque hex
      strings.
- [x] **AC4 — the red-proof: an injected fault in the unpacker turns the oracle
      red**, with the honest tree as the negative control.
      ⚠⚠ **The injected fault's output WAS asserted to change, not merely the
      file** — `an_injected_unpacker_fault_turns_the_oracle_red` asserts
      `mutant_digest != honest_digest` directly, every run. **Measured, not
      assumed — the two digests, on `L1021223.DNG`:**
      honest `cb653b5bec24d166eef2fd258ee61ac4`, mutant
      `59b032fe4320a27989ce61f3e3da7ff2`. Watched fail personally before this
      fault was chosen: a first attempt (starting `BitReader`'s cursor at bit
      1) changed the file and compiled but produced `Error::Truncated`, not a
      wrong digest — recorded in `DEC-017` and the module's own doc comment so
      it is not re-discovered. `the_honest_tree_is_the_negative_control`
      confirms the unmutated copy-and-rebuild apparatus reproduces the pinned
      digest, so the red result above is attributable to the injection, not
      the harness (`DEC-017`, mirroring `lint-red-proof.sh`'s control
      discipline, `DEC-009`).
- [x] **AC5 — a tier-A half runs in CI**, with no corpus and no tools: the RFC
      vectors, plus a hand-built fixture whose plane and digest are both known,
      plus the locator from `AC3`. `DEC-003` means CI can never run the tier-B
      half, so the tier-A half is the only half CI sees. Confirmed: six tier-A
      tests need neither corpus nor a tool —
      `md5_matches_the_rfc_1321_test_vectors`, `md5_streaming_matches_one_shot`,
      `a_mismatch_names_the_first_differing_sample`, `dnglab_raw_pixel_pgm_parses`,
      `dnglab_raw_pixel_pgm_rejects_malformed_input`,
      `compressed_files_are_skipped_by_name` — plus
      `hand_built_fixtures_plane_matches_its_known_md5`, a hand-built 4×2,
      14-bit fixture (`L1021223.DNG`'s own measured strip head, reused from
      `SPEC-012`) whose plane `[746, 725, 711, 752, 646, 705, 772, 686]` and
      whose MD5 `d1d83299c631541fac68da1051b19a23` (computed independently
      with Python's `hashlib.md5` at design time, not with this spec's own
      MD5) are both pinned.
- [x] **AC6 — eleven gates + `just lint-ci`**, and **CI observed green** on the
      shipping SHA. See Handback for the full local list and the CI run.

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each exists per-target and sum
across all targets.

- `md5_matches_the_rfc_1321_test_vectors` — AC1, tier A
- `plane_md5_matches_the_pinned_raw_checksum` — AC2, tier B
- `a_mismatch_names_the_first_differing_sample` — AC3, tier A
- `an_injected_unpacker_fault_turns_the_oracle_red` — AC4
- `the_honest_tree_is_the_negative_control` — AC4's control
- `compressed_files_are_skipped_by_name` — AC2

Added at build, beyond this minimum (all in `tests/plane_oracle.rs`):
`md5_streaming_matches_one_shot` (AC1, tier A — mirrors `sha256`'s own
split-across-a-block-boundary discipline), `dnglab_raw_pixel_pgm_parses` /
`dnglab_raw_pixel_pgm_rejects_malformed_input` (AC3, tier A — the reference
route's parser, unit-tested directly), `hand_built_fixtures_plane_matches_its_known_md5`
(AC5, tier A — the hand-built fixture named in AC5's text but not in this
list's original six).

## Non-Goals

- **Levels, crop, orientation** — `SPEC-014`. This oracle attaches to the
  **uncropped, un-normalised** plane, which is what `--raw-checksum` compares.
- **Re-deriving the plane contract.** `docs/oracle-contract.md` verified it on
  two frames; it is settled.
- **Any `src/` change.** `SPEC-012`'s unpacker is correct and proven; this spec
  only observes it.
- **Shelling out for MD5.** `md5`/`md5sum` exist on both hosts, but the tier-A
  half must be self-contained, and `sha256`'s precedent already settled this.

## Implementation Context

> **Measured 2026-09-04.** The honest digests are not in question — they are in
> the manifest and were confirmed 4/4 twice. What follows is what the *design
> probe* found, including a mistake it made.

### The baseline, already established

| file | shape | `raw_checksum` |
|---|---|---|
| `L1021223.DNG` | 8424×5632, 14-bit | `cb653b5bec24d166eef2fd258ee61ac4` |
| `L1026016.DNG` | 8424×5632, 14-bit | `3f1851259f31…` |
| `L1026192.DNG` | 8424×5632, 14-bit | `c7348179f042…` |
| `L1000622.DNG` | 5216×3472, 16-bit | `b0f602b90db91f981bbd6802fd0e6edf` |

All four already match, and all four are already pinned. **The oracle is
expected to be green on day one** — which is exactly why `AC4` is the whole
spec: a green oracle that cannot fail manufactures confidence.

### ⚠⚠ The warning this spec exists for — and the design probe walked into it

The design probe injected an "off-by-one" into the bit cursor:
`remaining.min(bits_left)` → `remaining.min(bits_left).max(1)`.

- `diff` confirmed **the file changed**. ✅
- `cargo build` confirmed **it compiled**. ✅
- The resulting plane digest was **byte-identical to the honest one.** ❌

`.max(1)` differs only when the min is zero, which never happens here. **It was
a semantic no-op that satisfied every check this repo's rules require.**

This matters because *"concluding from a mutation that never applied"* is a
failure this project has measured **five separate times**, and the rule written
to stop it — *assert the mutation changed the file and compiled* — **is not
sufficient**. The design session followed that rule exactly and still produced a
false red-proof.

**So `AC4` requires a third clause, and it is this spec's most important
sentence:**

> A red-proof must assert that the **output changed** — control digest ≠ mutant
> digest — **before** concluding anything about what the test caught.

⚠ **The design probe did NOT obtain a genuine faulty digest.** Two attempts to
re-run with a real fault were killed by session timeouts on a 95 MB plane, and
the number is not quoted here because it was not measured. **Producing it is
your job**, and the clause above is how you will know you have it.

### MD5, and why implement rather than depend

`tests/support/corpus.rs` already hand-writes **SHA-256 from FIPS 180-4** —
dev-only, class 1 provenance, proven against the published NIST vectors, with
`DEC-010` recording why it is not a dependency. MD5 from RFC 1321 is the same
shape (~60 lines) and RFC 1321 ships its own vector suite.

`md5` and `md5sum` both exist on this host and on CI's ubuntu image, so shelling
out would work — but the tier-A half is the only half CI runs, and an oracle
whose CI half depends on an external binary is one `PATH` change from silent.
`SPEC-005/FU-3` and `SPEC-012` both measured what a tool-gated test does when the
tool is absent: it passes, quietly.

### Traps

- ⚠ **A test that skips is a test that passed.** Tier-B tests pass whether or not
  the corpus is present. `just test` names what is missing; a bare `cargo test`
  does not.
- `just lint-ci`, not `just lint`, and **read CI**.
- The plane is 94.9 MB and `unpack_into` needs the **whole file addressable**
  (`DEC-016`, amended at `SPEC-012`'s ship). Peak RSS ≈ 182 MB per decode; four
  files in one test run is a real consideration.


## Follow-ups

| id | finding | disposition |
|---|---|---|
| `FU-1` | the red-proof — the **only** proof this oracle can fail — skips on every CI runner, so the half CI sees carries no evidence of it | `fixed` at ship. A corpus-free red-proof + control over the hand-built fixture, reusing the same mutated-copy apparatus. Measured: **10 tests / 0.01 s → 12 tests / 0.91 s** with no corpus, printing `honest=d1d83299… mutant=6aa91ec5…`. ⚠ Verify **corrected the framing in the build's favour** and the correction is the accurate statement: CI's tier-A half *does* catch a broken unpacker — what it lacked was not protection but **proof of that protection** |
| `FU-2` | `compressed_files_are_skipped_by_name` proves the **union**, not the **partition** — moving a decodable file into `SKIPPED_COMPRESSED` with a fabricated reason drops coverage 4→3 with the suite green | `fixed` at ship — asserted against the files themselves via `require_uncompressed()`, plus a coverage count. **Red-proofed with the reviewer's exact mutation**, which now fails naming the file. ⚠ This one is the orchestrator's to have missed: the spec's `AC2` said *"skipped by name with a stated reason"* and got exactly that — a reason that is **stated** is not a reason that is **true** |
| `FU-3` | the injected fault leaves `L1000622.DNG`'s digest byte-identical, so `unpack_byte_aligned` has no **oracle** red-proof | `closed` — verify judged it well mitigated and the orchestrator agrees: `DEC-017` anticipates it **in writing**, and the byte-aligned path is pinned corpus-free by `plane_unpack.rs`. The trigger is mechanical rather than memory: a fault reaching that path would fail `plane_unpack.rs` before this oracle could observe it |
| `FU-4` | `md5.rs`'s doc comment names `PROBE_MD5_SOURCE`; the constant is `MD5_SOURCE` | `fixed` at ship |

**4 follow-ups · 3 `fixed` · 1 `closed` · 0 ship-blockers.**

⚠ **The strongest result in this spec was a by-product of a verify check.**
Testing whether the locator had ever seen real data, the reviewer ran it against
`dnglab --raw-pixel` — establishing that our honest plane agrees with dnglab
**sample-for-sample across all 47,443,968 samples**, which is strictly stronger
than the MD5 match this spec was built to assert.

## Reflection

**1. What would I do differently next time?**

**Write `AC2` to demand a true claim, not a stated one.** It said the compressed
files must be *"skipped by name with a stated reason"*, and the build delivered
exactly that — a list, a name, a non-empty string. `FU-2` is the gap between
**stated** and **true**: a fabricated reason passed every assertion while
silently dropping a quarter of the oracle's coverage. The criterion got what it
asked for and not what it meant.

That is the same family as `SPEC-010`'s `AC4` being narrower than the finding it
carried, and it suggests the sharper habit: **write the criterion as the
sentence a lie would have to survive.**

**2. Does any template, constraint, or decision need updating?**

- **The third clause earned its place twice more.** `AC4` required asserting the
  *output* changed, not just the file. The build's **first** candidate fault
  changed the file, compiled, and produced `Error::Truncated` rather than a wrong
  digest — rejected and recorded in `DEC-017`. Then, closing `FU-2`, the
  orchestrator's first mutation **did not compile** (the lists are fixed-size
  arrays) and was caught before any conclusion. Two independent saves, in one
  spec, from a clause that did not exist a week ago.
- **`DEC-017`'s apparatus is the reusable artefact here.** Mutating a temp-dir
  copy and rebuilding *that* runs in ~1.5 s, never touches the working tree, and
  cannot silently no-op. The design session's own probe rebuilt in place, timed
  out twice, and left a stale process holding a mutated `src/plane.rs`. Any
  future red-proof over expensive output should copy this shape.
- **`tier_map`'s build hint is 0 for 7** while the verify hint is 2 for 2. That
  asymmetry is now large enough to be the finding: builds are dispatched to a
  session whose model the orchestrator does not pick, so the map is recording a
  preference in a field read as an observation — exactly what
  `tier-map-predicts-what-it-should-record` says, with seven data points.

**3. Is there a follow-up spec to write now?**

**No.** Three follow-ups are fixed and one is closed with a mechanical trigger.

Worth recording as the project-level fact: **`irradiance`'s decoded plane now
agrees with `dnglab` sample-for-sample across all 47,443,968 samples of a Q2
Monochrom frame**, and the repo asserts the digest on every run with a red-proof
that runs *in CI*. The oracle no longer merely exists — it is known to be able to
fail, on the half that always runs.


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
