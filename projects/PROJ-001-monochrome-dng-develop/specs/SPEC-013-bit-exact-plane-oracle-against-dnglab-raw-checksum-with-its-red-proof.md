---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-013
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
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

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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
