---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-015
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   ⚠ RAISED from the stage backlog's [S]. That S predates the
                                   #   design probe, which found the oracle needs THREE layers
                                   #   (property, per-pixel, tier-A red-proof), not one assertion.
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-004, DEC-005, DEC-016, DEC-018, DEC-019]
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input, library-not-application, test-before-implementation]
  related_specs: [SPEC-012, SPEC-013, SPEC-014]

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
value_link: "closes STAGE-002 by making SPEC-014's levels and geometry independently
  checkable — the one surface DEC-004 measured both existing oracles blind to"

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
  tokens_estimate: 60000000
  # Calibration basis, stated so the next estimate can be judged rather than
  # guessed: SPEC-013 (an oracle, expected S) cost ~47M across build+verify;
  # SPEC-014 (expected L) estimated 26M and cost 88,845,024 — a 3.42x miss, the
  # largest recorded, and it came from cycles the estimate did not model (a
  # verify that raised six follow-ups, then a fourth delegated round to close
  # them). This spec is expected M with its design probe already done, so the
  # build should be shorter than SPEC-014's — but it is an ORACLE with a
  # mandatory red-proof, and SPEC-013 shows that class attracts punch lists.
  # 60M assumes build + verify + one punch-list round. If it lands near 45M the
  # estimator was pessimistic; near 100M and the 3.42x pattern is systemic, not
  # a SPEC-014 accident.
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-05
      notes: "main-loop, not separately metered (AGENTS.md §4). The design probe RAN, against three real frames (111,529,040 pixels), and it settled the spec's central question — whether an analytic oracle must reimplement the transform (weak independence) or can assert PROPERTIES of it (strong). Measured: (1) the shipped output is within 0.499968 LSB of the exact real-valued affine map on every one of 111.5M pixels, zero at or above 0.5, so the tolerance is pre-registered from evidence rather than guessed; (2) 45.0-50.1% of pixels differ from a TRUNCATED map, so an oracle written with floor fails half of every frame — DEC-018's warning, now measured on real data; (3) histogram(output) == histogram(normalized crop window) held EXACTLY on all three frames including the Orientation 6 one, WITHOUT reimplementing the eight-case orientation table anywhere; and (4) that property catches both faults the existing oracles miss — FU-3's orientation identity fault (15,425,929 pixels wrong, 33%) and SPIKE-001's BlackLevel+64 (36,824,570 wrong, 78.8%), the fault SSIMULACRA2 scores 95.62 (passing) on and the plane checksum is bit-identical on. Probe cost 2.6s for all three frames in release. Probe crate lived in the scratchpad and was never committed."

    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 56224398
      estimated_usd: 24.15
      duration_minutes: 65
      recorded_at: 2026-09-05
      notes: "tokens_total/estimated_usd are per-component (input $3, output $15, 1h cache-write $6, cache-read $0.30 per MTok - published Sonnet rates), summed over this session's own transcript (69b4c29b-d5cc-4fd8-8ef7-d2da3fdf661c.jsonl, identified by scratchpad-dir uuid, not content match - signal orchestrator-transcript-looks-like-a-prior-attempt), deduped by message.id, rounded up 20% per this handoff's own instruction. Raw measured combined was 46,853,665 (~$20.13 per-component); AGENTS.md section 4's flat-rate fallback on the same raw total would read ~$140 - signal flat-rate-overstates-cached-sessions gained this session as its 5th data point (~7x), so the per-component figure is reported, not the flat one. [SPEC-015/FU-4: handback-sync.sh transcribed only the FIRST physical line of this note into the spec, leaving an unterminated double-quoted scalar and making the whole front matter unparseable from c57f88d; restored in full here as one line.]"
    - cycle: verify
      agent: claude-opus-5[1m]
      interface: claude-code
      tokens_total: 20515070
      estimated_usd: 45.60
      duration_minutes: 95
      recorded_at: 2026-09-05
      notes: "VERDICT APPROVED at a3f0063 (CI 9/9, run 34003871323); 8 follow-ups FU-4..FU-11, 0 ship-blockers. Cost is a transcript sum deduped by message.id from THIS session's own JSONL (d56874fe-79ae-4cbf-b1b9-c0e078c2dc7b.jsonl, identified by the scratchpad-dir uuid, not by content match): 184 usage objects / 103 unique ids, all message.model=claude-opus-5; raw combined 17,095,892 (input 206 / output 77,549 / cache-read 16,784,762 = 98.2% / cache-write-1h 233,375 / cache-write-5m 0), priced PER-COMPONENT at published Opus rates ($15/$75/$30-1h/$1.50-read) = $38.00, then BOTH figures rounded up 20% per this handoff's point 7 to cover the turns spent writing this handback. ⚠ THIS notes field is deliberately ONE LINE: the build's multi-line scalar is what handback-sync truncated into an unterminated quote in the spec's front matter — see FU-4, which must be fixed BEFORE this entry is synced."
  totals:
    tokens_total: 76739468
    estimated_usd: 69.75
    session_count: 3
---

# SPEC-015: Analytic levels and geometry oracle

## Context

`SPEC-014` shipped level normalization and the `ActiveArea` → `DefaultCrop` →
`Orientation` geometry, and asserted **its own arithmetic**. That was the only
thing available: `SPEC-013`'s `--raw-checksum` attaches to the *uncropped,
un-normalised* plane by contract, so it cannot see any of `SPEC-014`'s work, and
`DEC-004` settled that no comparison oracle ever will — `SPIKE-001` measured the
plane checksum **bit-identical** under a `BlackLevel + 64` fault, and the develop
oracle (SSIMULACRA2, `DEC-005`) scoring **95.62 — passing** on the same fault,
blind up to **+256 (50 %)**.

So the develop path currently has **no independent check at all**, and
`SPEC-014` proved twice that this is not theoretical:

- **`AC4`** — no decodable file has a non-zero `ActiveArea` origin, so an
  implementation ignoring it passes every corpus test. Mutation left **140 of
  141** green.
- **`FU-3`** — a real orientation fault (identity at the mapper's call site)
  left **141 of 141** green. It is now closed by one tier-A fixture, on a
  6-pixel image.

Both holes were found by someone thinking to run a mutation. Neither was found
by a gate. This spec is the gate.

`SPEC-015` closes `STAGE-002`.

## Goal

Check `develop_into`'s output against expectations derived **independently of
how `develop_into` computes them** — so that the two can disagree — and prove
the check can go red.

## The design decision this spec rests on

⚠ **An analytic oracle that reimplements the transform is a mirror, not an
oracle.** Written by the same project from the same reading of the same spec, a
second copy of the eight-case orientation table fails and succeeds for the same
reasons as the first. `DEC-004` already names this limit: it verifies *"the
arithmetic we chose, not that our choice matches Adobe's intent"*.

**The design probe found a route that avoids it** (`## Implementation Context`).
Three layers, in increasing strength and decreasing independence — and the spec
requires all three, because each covers what the others cannot:

| layer | what it asserts | independence |
|---|---|---|
| **L1 — properties** | facts that follow from what the transform *means*, not how it is computed | **strong** — reimplements nothing |
| **L2 — per-pixel bound** | every output pixel is within a pre-registered tolerance of the *exact real-valued* affine map | **strong** — never names a rounding rule |
| **L3 — red-proof** | the whole thing goes red on a deliberate fault, in CI | n/a — it is the proof the rest can fail |

**L2 is the move that keeps this honest.** `develop_into` rounds to nearest
(`DEC-018`, `FU-4`), and an oracle told to expect round-to-nearest is reading
the implementation's own decision record. Asserting instead that the output is
within **< 0.5 LSB of the exact real number** is a statement about the affine
map itself: it is satisfied by *any* correct rounding rule and violated by every
incorrect map. **Separate what is forced from what is chosen** — the endpoints
are forced and get exact assertions; the interior rounding is chosen and gets a
bound. `FU-4`'s existing test already pins the choice; this spec must not
duplicate it.

## Inputs

- `src/develop.rs` — `develop_into`, `output_dimensions` (the code under test;
  **do not modify it**)
- `src/plane.rs` — `unpack_into` (`DEC-016`'s caller-owned-buffer shape)
- `src/ifd.rs` — `Sensor`'s `black_level`, `white_level`, `active_area`,
  `default_crop_origin`, `default_crop_size`, `orientation`
- `tests/develop.rs`, `src/develop.rs`'s unit tests — **read them to avoid
  duplicating them.** `SPEC-014` already asserts endpoints on real tags, the
  crop dimensions, `Orientation` 1 and 6 dimensions, the `ActiveArea`-relative
  origin, and (`FU-4`) the rounding rule. This spec adds what none of them do:
  a check over **every pixel of a real frame**, derived independently.
- `DEC-004` (analytic, never by comparison), `DEC-005` (why SSIMULACRA2 cannot
  do this), `DEC-018` (the rounding rule and its 50 % trap), `DEC-019`
- `docs/oracle-contract.md` — the three layers and where this one attaches

## Outputs

- **`tests/develop_oracle.rs`** — a new tier-A + tier-B test target. Tier A must
  carry real assertions, including the red-proof (see `AC6`).
- Reusable oracle helpers. Put them where `tests/support/` already lives, next
  to `corpus.rs` / `md5.rs`; do **not** add them to the library — this is test
  support, not a product surface (`library-not-application`).
- **A `DEC-*` if and only if** you make a non-obvious choice — e.g. the
  property set, or where the helpers live. Not required if nothing surprises
  you; a decision record for "I did what the spec said" is noise.
- A **provenance row** — the oracle derives expected values from DNG 1.7's own
  definitions, so **class 1 — specification**, same as `src/develop.rs`'s row.
  ⚠ It is a *separate row*: it is a separate implementation, and the ledger
  tracks implementations, not features.

## Acceptance Criteria

- [x] **AC1 — L2, the per-pixel bound, on every decodable frame.** For each of
      the three decodable files, every output pixel is within **< 0.5 LSB** of
      the exact real-valued affine map
      `(clamp(raw, B, W) − B) × 65535 / (W − B)`, computed in `f64` from tags
      **read from the file**. ⚠ **Pre-registered tolerance and its falsifier,
      stated here before the build measures anything**
      (`pre-register-the-tolerance`): the bound is **`< 0.5`**, and the
      falsifier is **a single pixel at `≥ 0.5`**. Measured headroom at design:
      max deviation **0.499968** over 111,529,040 pixels, zero at or above 0.5.
      If the build measures a max at or above 0.5, that is a **finding, not a
      threshold to relax** — say so and stop.
      `every_pixel_is_within_half_an_lsb_of_the_exact_affine_map` (tier B).
- [x] **AC2 — L2 must never be satisfiable by truncation.** Assert in the same
      test that the shipped output **differs** from the truncated map on a large
      fraction of pixels, so a future "simplification" cannot pass AC1 by
      accident. Measured: **45.0–50.1 %** across the three frames. Assert a
      floor of **> 40 %**, not the exact figure — the fraction is data-dependent
      and pinning it exactly would make the test brittle for the wrong reason.
      Same test as AC1. ⚠ **`SPEC-015/FU-8`** — the floor's real margin is not
      "5 points": in-range disagreement is structurally ~0.5006/0.5006/0.5001
      regardless of image content, and only CLIPPED pixels (which land on
      exact integers, where `round == floor`) pull the total down. **A correct
      implementation falls under the 40 % floor once the clipped share exceeds
      20.09 %** — measured break-even, not the 5-point margin the raw 45.0 %
      figure suggests. `L1000622.DNG` is already at 10.05 % clipped, half way
      there. This fails loudly and in the safe direction (a false red on a new
      corpus file, never a false green), so it is diagnosis for the next
      false-red investigator, not a reason to change the floor.
- [x] **AC3 — L1, the permutation property, WITHOUT reimplementing the
      orientation table.** `develop_into` applies a *permutation* of the
      normalized crop window, so `histogram(output)` must equal
      `histogram(normalize(crop window))` — computed in raster order with **no
      orientation applied at all**. Holds exactly on all three frames including
      the `Orientation 6` one (measured). ⚠ **The test must not contain the
      eight-case table in any form.** If your implementation of this criterion
      needs to know what `Orientation 6` does, you have written a mirror.
      `the_developed_histogram_is_the_normalized_crop_windows` (tier B).
- [x] **AC4 — L1, the injectivity property.** `normalize` is strictly monotonic
      and injective on `[BlackLevel, WhiteLevel]` whenever `W − B ≤ 65535`, so
      the output carries exactly as many distinct levels as the crop window has
      distinct raw values. Measured: **15,872** distinct levels on
      `L1026016.DNG` (`= 16383 − 512 + 1`) and **16,164** on `L1000622.DNG`
      (`= 16383 − 220 + 1`) — both the full in-range domain, exactly.
      Assert monotonicity over the whole domain in tier A and the distinct-level
      identity in tier B.
      `normalization_is_strictly_monotonic_and_injective` (tier A) /
      `distinct_output_levels_equal_distinct_input_levels` (tier B).
- [x] **AC5 — the oracle catches both faults the existing oracles miss.** Not a
      claim — a test. Both are measured and both must be exercised:
      **(a)** a levels fault — `BlackLevel + 64`, which leaves `--raw-checksum`
      **bit-identical** and scores SSIMULACRA2 **95.62, passing**; measured to
      corrupt **36,824,570 of 46,726,912** pixels (78.8 %) as seen by this
      oracle. **(b)** an orientation fault — identity at `crop_source_coords`'
      call site, the exact fault that left `SPEC-014` 141/141 green; measured to
      corrupt **15,425,929** pixels (33 %).
      `the_oracle_is_red_on_a_levels_fault` /
      `the_oracle_is_red_on_an_orientation_fault`.
- [x] **AC6 — the red-proof runs where CI can see it (`oracle-must-be-shown-red`).**
      ⚠ `SPEC-013/FU-1` is the precedent and the warning: its red-proof works,
      and it is **invisible to CI**, because it needs the corpus. **AC5's two
      faults must go red with `IRRADIANCE_CORPUS_DIR` unset**, over a hand-built
      fixture — the shape `SPEC-014/FU-3` used, and the shape `SPEC-013`'s
      reviewer measured as costing 1.47 s. State the mechanism you chose and why.
      Verified by running the tier-A subset with no corpus and watching it fail.
- [x] **AC7 — no library code changes.** `src/develop.rs`, `src/plane.rs` and
      `src/ifd.rs` are **0 lines changed** against `main`. This spec adds a
      check; if the check fails, that is a finding to report, not a reason to
      edit the code under test. ⚠ If the oracle genuinely finds a defect, **stop
      and say so** — that is the single most valuable outcome this spec can have,
      and it must not be quietly absorbed by adjusting either side.
- [x] **AC8 — cost.** The oracle runs over ~111.5 M pixels. Design measured
      **2.6 s** for all three frames in `--release`; a debug `cargo test` will be
      slower. Report the measured wall-clock of the tier-B tests. If it exceeds
      **60 s**, say so and propose a subsample rather than silently shipping a
      slow suite — pre-registered, same rule as AC1.
- [x] **AC9 — eleven gates + `just lint-ci`**, CI **observed** green on the
      shipping SHA.

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum across all.

- `every_pixel_is_within_half_an_lsb_of_the_exact_affine_map` — AC1 + AC2, tier B
- `the_developed_histogram_is_the_normalized_crop_windows` — AC3, tier B
- `normalization_is_strictly_monotonic_and_injective` — AC4, **tier A**
- `distinct_output_levels_equal_distinct_input_levels` — AC4, tier B
- `the_oracle_is_red_on_a_levels_fault` — AC5(a) + AC6, **tier A**
- `the_oracle_is_red_on_an_orientation_fault` — AC5(b) + AC6, **tier A**

## Non-Goals

- **Changing anything in `src/`.** See AC7.
- **A second copy of the orientation table.** That is the failure mode this
  spec is designed around; see AC3.
- **Re-asserting what `SPEC-014` already asserts** — endpoints on real tags,
  crop dimensions, `Orientation` 1 and 6 dimensions, the `ActiveArea`-relative
  origin, `FU-4`'s rounding pin. Read those tests; do not duplicate them.
- **SSIMULACRA2, `dnglab --srgb`, or any perceptual comparison.** `DEC-004` and
  `DEC-005` closed that route with measurements; reopening it is a new decision.
- **Fuzzing.** This adds no parser and no new input surface — it consumes the
  same `Sensor` the existing targets already fuzz. §12 bar 2 does not fire.
  Say so in the handback rather than adding a target nobody asked for.
- **Demosaic, colour, tone, opcodes** — `STAGE-003` / PROJ-002.

## Implementation Context

> **Measured 2026-09-05** by a throwaway probe crate in the scratchpad, against
> all three decodable corpus files — 111,529,040 pixels. Reproduce rather than
> re-derive; the numbers below are what the acceptance criteria are calibrated
> against.

### The tolerance, pre-registered with its evidence

| file | raw → out | orientation | max \|shipped − exact\| | pixels ≥ 0.5 |
|---|---|---|---|---|
| `L1021223.DNG` | 8424×5632 → 8368×5584 | 1 | **0.499968** | **0** |
| `L1026016.DNG` | 8424×5632 → 5584×8368 | **6** | **0.499968** | **0** |
| `L1000622.DNG` | 5216×3472 → 5212×3468 | 1 | **0.499969** | **0** |

Headroom to the bound is **0.000032**. That is not slack to be spent: it is what
correct round-to-nearest looks like. Any *other* correct rounding rule also lands
under 0.5, which is exactly why the bound is stated this way rather than as
equality against a formula.

### The trap, measured on real frames

Pixels differing from a **truncated** map: **50.1 %** (`L1021223`), **49.1 %**
(`L1026016`), **45.0 %** (`L1000622`). `DEC-018` warns about this in the
abstract; these are the numbers. **An oracle that derives expected values with
`floor` fails on roughly half of every frame** — and the tempting repair is to
"fix" the oracle to match the implementation, which silently destroys its
independence. AC1's `< 0.5` bound is immune to the whole question, and AC2 exists
to make sure nobody quietly satisfies AC1 by truncating anyway.

### The property route works, and it reimplements nothing

`histogram(develop_into output) == histogram(normalize(crop window))`, the window
taken in raster order with **no orientation applied**:

```
L1026016.DNG (Orientation 6): true — distinct levels 15872 expected, 15872 got
L1000622.DNG (Orientation 1): true — distinct levels 16164 expected, 16164 got
```

15872 = `16383 − 512 + 1`; 16164 = `16383 − 220 + 1`. Both are the **entire**
in-range domain, so normalization is injective on real data, not just in theory
(AC4).

### And it catches both faults the existing oracles are blind to

Measured by injecting each fault into `src/develop.rs`, rebuilding, running the
probe, then restoring byte-identically:

| fault | plane checksum | develop oracle | **this oracle** |
|---|---|---|---|
| `BlackLevel + 64` | **bit-identical** (SPIKE-001) | **95.62 — passes** (DEC-004) | **red** — 36,824,570 / 46,726,912 pixels wrong (**78.8 %**) |
| orientation identity at the call site | unaffected — attaches before | untested | **red** — 15,425,929 pixels wrong (**33 %**) |

The second row is `SPEC-014/FU-3`, the fault that left **141 of 141** tests
green. This oracle sees it on 15.4 million pixels.

### Cost

Probe ran all three frames in **2.6 s** wall-clock, `--release`, single-threaded,
including file read and unpack. Peak memory is `SPEC-014`'s measured ≈275.9 MB
plus one expected-value buffer; `f64` accumulation can be done per-pixel without
materialising a second full plane, and AC8 asks you to.

### Traps

- ⚠ **Do not write the eight-case orientation table.** AC3 exists because of it.
- ⚠ **Do not derive expected values from `DEC-018`'s rounding rule.** AC1's
  bound is deliberately rule-agnostic; reading the rule makes the oracle a
  mirror of the decision record.
- ⚠ **The red-proof must run with the corpus absent** (AC6). `SPEC-013/FU-1` is
  a working red-proof CI has never once executed.
- ⚠ **If the oracle finds a real defect, stop and report it** (AC7). Do not
  adjust either side to make it pass.
- `just lint-ci`, not `just lint`, and **read CI**.
- A tier-B test passes whether or not the corpus is present; only `just test`
  names what is missing.

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
