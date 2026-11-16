---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-071
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  id: PROJ-004
  stage: STAGE-021
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-063, DEC-064, DEC-016]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-068, SPEC-069, SPEC-070]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-021's <capability>". Optional; null is acceptable.
value_link: "closes STAGE-020's four [XS] follow-ups so the release is cut from a clean tree"

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
  tokens_estimate: 12000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-071: STAGE-020 follow-ups, bundled: a Compression 7 allocation pin, one assert message, `just fuzz-ljpeg`, and the timing recipes in AGENTS.md

## Context

The first spec of STAGE-021. STAGE-020 left four [XS] follow-ups in the PROJ-004 brief.
Each is small; they are bundled so they cost one build and one verify. None changes
library behaviour.

## Goal

Close all four, with every existing test, pin and oracle green and unedited except the one
assert message (item 2).

## Design

### 1. An absolute Compression 7 allocation pin (`tests/plane_units_alloc.rs`)

Today `units_and_unit_decode_allocate_nothing` checks Compression 7 fixtures only as "the
units allocate exactly what `unpack_into` allocates". Both go through the same per-strip or
per-tile kernel, so a new allocation in that kernel moves both counts and the check stays
green.

**The design probe measured the absolute count** at `4664773`, with a temporary print in
that test (not committed). For both tier-A fixtures and every `max_band_rows` (1, 3,
`u32::MAX`):

| Fixture | Strips or tiles | Allocations (units) | Allocations (`unpack_into`) |
|---|---|---|---|
| `c7 multi-strip` (6×7, 12-bit, 4 strips of 2 rows) | 4 | 40 | 40 |
| `c7 tiles` (21×13, 12-bit, 12 tiles of 8×4) | 12 | 120 | 120 |

That is **exactly 10 allocations per strip or tile**, all from `ljpeg::parse_headers`'s
Huffman-table parse.

- **Add a named constant**, for example `const C7_ALLOCATIONS_PER_CHUNK: usize = 10;`,
  with a comment stating where the 10 come from. Read `parse_headers` / `parse_dht` and
  list the allocations: the `dc_tables` `Vec` and its growth, each `huffval` `Vec`,
  `component_tables`, and the clones. Say that it was measured on these one-component
  fixtures at `4664773`.
- **Assert, for each compressed fixture,** `decode_allocs == C7_ALLOCATIONS_PER_CHUNK ×
  chunk_count`, where `chunk_count` comes from the sensor's strip or tile count, not from
  a literal. **Keep** the existing old-equals-new assertion beside it.
- The allocation count depends on the header (DHT count, `Nf`). The pin is for these
  fixtures; the comment says so. It does not pin held files.

### 2. One assert message (`tests/plane_oracle.rs`)

In `inject_chunk_extraction_fault`, the `assert_eq!` failure message still says
"BitReader::read's chunk extraction", but since SPEC-069 the needle is the 14-bit group
kernel's first two shifts (DEC-064 §2). Reword **only the message string**, for example:
"(the 14-bit group kernel's first two shifts in unpack_bitstream, DEC-064)". The needle,
the replacement, the count and every assertion stay as they are.

### 3. `just fuzz-ljpeg` (`app.just`)

A recipe mirroring `fuzz-plane` exactly:

```text
fuzz-ljpeg seconds="60":
    mkdir -p fuzz/corpus/ljpeg
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ljpeg \
        fuzz/corpus/ljpeg fuzz/seeds/ljpeg -- -max_total_time={{seconds}}
```

- Its comment names SPEC-038 (the target) and the same `+toolchain` trap.
- `fuzz/fuzz_targets/ljpeg.rs`'s header may add one line pointing at the recipe. The
  command already there stays.
- **The SPEC-070 build found that** the header command fails on a fresh checkout, because
  `fuzz/corpus/ljpeg` (gitignored) does not exist. The `mkdir -p` is the fix.

### 4. AGENTS.md §6

The §6 rule is that every recipe's commands appear in the block. Today four recipes are
missing; add each, with a short comment in the block's style:
- `orient-timing` (SPEC-067);
- `unpack-timing` (SPEC-069);
- `ljpeg-timing` (SPEC-070);
- `fuzz-ljpeg` (this spec).

For the three timing recipes the comment says: release build, corpus-gated (SKIP line
and exit 0 when the file is absent), median of 7, no CI gate. Add a sentence to the
section's prose history naming the SPEC that added each recipe.

## Inputs

- `tests/plane_units_alloc.rs` (all of it), `src/ljpeg.rs` (`parse_headers`, `parse_dht`).
- `tests/plane_oracle.rs` (`inject_chunk_extraction_fault`).
- `app.just` (the fuzz and timing recipes), `fuzz/fuzz_targets/ljpeg.rs` (header).
- AGENTS.md §6.
- Decisions: DEC-063 §2 (units allocate nothing new), DEC-064 §2, DEC-016.

## Outputs

- **Files modified:** `tests/plane_units_alloc.rs`, `tests/plane_oracle.rs` (one string),
  `app.just`, AGENTS.md, and optionally `fuzz/fuzz_targets/ljpeg.rs` (one comment line).
- **No `src/` change, no public item, no CHANGELOG entry.** It is test and tooling only,
  so `[Unreleased]` is not touched.

## Acceptance Criteria

- [ ] **AC1. The pin bites.**
  - `units_and_unit_decode_allocate_nothing` asserts the absolute count for both
    compressed fixtures, and passes.
  - Red-proof, on an archived copy of the tree: add one allocation per call in the
    shared per-chunk kernel (for example a `std::hint::black_box(Vec::<u8>::with_capacity(1))`
    in `plane::decode_unit`'s Compression 7 arm). The new absolute assertion goes red,
    while the old-equals-new assertion alone would stay green. Show both.
- [ ] **AC2. The message only.** `git diff` on `tests/plane_oracle.rs` touches one
  string literal. `an_injected_unpacker_fault_turns_the_oracle_red` and its control stay
  green.
- [ ] **AC3. `just fuzz-ljpeg 60`** runs clean from a checkout with no
  `fuzz/corpus/ljpeg` directory. Delete it first and show that it is created.
- [ ] **AC4. AGENTS.md §6** lists every recipe in `app.just`. Show a list of
  `app.just` recipe names against the block's comment headers, with none missing.
- [ ] **AC5. No regression.** Every gate is green with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`

  No `src/` file differs from `main`.

## Failing Tests

AC1's new assertion is the only test change, and it passes on today's code by design: it
pins today's count. The test-first evidence is AC1's red-proof.

## Non-Goals

- Pinning allocation counts on held (tier-B) files.
- Reducing the 10 allocations.
- The test lane (SPEC-072).
- Any `src/` change.

## Notes for the Implementer

- **Build this before SPEC-072.** Both edit AGENTS.md §6; SPEC-072 is cut after this
  merges.
- **The race fix (PR 243) made the tally per thread.** Keep it that way. The new
  assertion reads the same `count()`.
- **`just test` today takes about 37 min in debug.** Run it in the background. SPEC-072
  is what makes it fast.

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
