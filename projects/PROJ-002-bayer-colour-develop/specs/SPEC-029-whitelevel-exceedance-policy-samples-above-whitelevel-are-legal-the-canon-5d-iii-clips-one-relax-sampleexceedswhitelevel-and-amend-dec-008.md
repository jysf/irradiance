---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-029
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-024]             # exercised on the Canon Bayer plane the CFA gate now accepts

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: claude-opus-4-8

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Unblocks the Canon Bayer plane for SPEC-025's oracle — a sample above WhiteLevel must not fail unpack; amends DEC-008's mono-era assertion."

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
  tokens_estimate: null
  sessions:
    - cycle: build
      agent: claude-opus-4-8
      interface: other
      tokens_total: 31829740
      estimated_usd: 23.45
      duration_minutes: 20
      recorded_at: 2026-10-30
      notes: "both Failing Tests + both corpus Bayer DNGs (Canon single-strip, Nikon 12-strip) now unpack; Error::SampleExceedsWhiteLevel REMOVED (not strict-mode); DEC-029 written amending DEC-008's permanent-invariant clause; tests in tests/plane_unpack.rs (the unpack_into home, AGENTS.md §12), not the spec's literal `src/plane.rs` wording — same divergence HANDOFF-066 recorded; tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated"
  totals:
    tokens_total: 31829740
    estimated_usd: 23.45
    session_count: 1
---

# SPEC-029: WhiteLevel-exceedance policy: samples above `WhiteLevel` are legal (the Canon 5D-III clips one); relax `SampleExceedsWhiteLevel` and amend DEC-008

> **DESIGN — `cycle: design`.** Approach + failing tests below; a build session
> makes the Failing Tests pass without redesigning. Blocks SPEC-025 (the oracle).

## Context

`plane::unpack_into` asserts every sample is `≤ WhiteLevel` and errors
`SampleExceedsWhiteLevel` otherwise — DEC-008's "max > WhiteLevel is impossible,
the free assertion" (`src/plane.rs:321`), true for the mono corpus. The Canon
5D-III Bayer DNG carries a sample `15464` above its `WhiteLevel 15000`, which is
**legal** — real sensors record specular highlights above the nominal white — so
the assertion rejects a valid file. This spec amends DEC-008 and relaxes the policy
so an above-WhiteLevel sample no longer fails unpack. Prerequisite for the bit-exact
oracle (SPEC-025); the plane must preserve the raw values (clamping is develop's
job, not unpack's) or the `dnglab --raw-checksum` can't match.

## Goal

Amend DEC-008 and relax `plane::unpack_into` so a sample above `WhiteLevel` no
longer errors — the Canon 5D-III plane unpacks, with raw sample values **preserved,
not clamped**. Keep a meaningful bound (samples still fit the declared bit depth).
A new DEC records the amendment (reconcile its number — DEC-029 is informally
reserved for the crustyimg LUT boundary).

## Inputs

- **Files to read:**
  - `src/plane.rs:317-331` — the `max > WhiteLevel` assertion
    (`SampleExceedsWhiteLevel`) inside `unpack_into`.
  - **DEC-008** (`decisions/DEC-008-...md`) — its Consequences call this assertion
    "a permanent invariant" that "earns its keep" (it caught SPIKE-002's 16-bit
    byte-swap: 39186 vs WhiteLevel 16383). This spec amends that clause.
  - `Error::SampleExceedsWhiteLevel` (`src/lib.rs`); `Sensor.white_level`.
- **Corpus:** `CANON-EOS-5D-MARK-III/5G4A9394.DNG` — a sample `15464` > `WhiteLevel
  15000` (unpack rejects it today); `LEICA-Q2-MONO` / `LEICA-M-MONOCHROM` (within
  WhiteLevel) for the regression.

## Outputs

- **Files modified:** `src/plane.rs` (relax the assertion); `src/lib.rs` (the error
  variant's fate); **a new DEC** amending DEC-008.
- **Relax `unpack_into`:** a sample above `WhiteLevel` is **no longer an error**. The
  plane **preserves raw sample values — no clamping** (clamping is develop's job;
  clamping here would change bytes and break SPEC-025's `dnglab --raw-checksum`).
  Recommended: drop the `max > WhiteLevel` loop (`:317-331`); `Error::
  SampleExceedsWhiteLevel` is then unused — remove it, or keep it only behind a
  future opt-in strict mode (build's call; the **default must accept above-white**).
- **New DEC (amends DEC-008):** samples may legitimately exceed `WhiteLevel`
  (specular highlights), so `max ≤ WhiteLevel` is **not** an unpack invariant;
  `WhiteLevel` is a develop-time saturation parameter. The byte-swap detection
  DEC-008 pinned on this assertion now rests on the **bit-exact oracle (SPEC-025)**
  plus the layer-0 packed-bits check — record that trade-off honestly (the free
  in-library guard for the byte-aligned path is reduced). Reconcile the DEC number —
  **DEC-029 is informally reserved for the crustyimg LUT boundary**, so take the next
  genuinely free one.

## Acceptance Criteria

- [ ] A synthetic plane with a sample above `WhiteLevel` unpacks **Ok** (no
  `SampleExceedsWhiteLevel`), and that raw value is present unchanged in `dst` (not clamped).
- [ ] Mono planes within WhiteLevel (Q2M, M Monochrom) unpack **byte-identically** to
  before (regression).
- [ ] (corpus) `CANON-EOS-5D-MARK-III/5G4A9394.DNG` unpacks with no
  `SampleExceedsWhiteLevel`, its above-white sample preserved.
- [ ] A new DEC exists amending DEC-008 (status accepted; records the reversed
  consequence and names SPEC-025 as the byte-swap guard).
- [ ] DEC-008's two-path byte-alignment rule is untouched — only the post-unpack
  assertion changes.

## Failing Tests

- **`src/plane.rs` tests**
  - `a_sample_above_white_level_unpacks_and_is_preserved` — build a plane whose max
    sample exceeds `WhiteLevel`; `unpack_into` returns `Ok` and `dst` holds the raw
    above-white value (not clamped to WhiteLevel).
  - `a_plane_within_white_level_is_unchanged` — regression; existing Q2M / M
    Monochrom unpack output byte-identical.
- **Corpus (gated):** the Canon 5D-III DNG unpacks without error.
- **Guard note:** DEC-008's byte-swap guard moves to SPEC-025's oracle; if the error
  variant is kept for a strict mode, test that mode still fires; if removed, the spec
  records the oracle as the replacement guard.

## Non-Goals

- Clamping / normalizing samples to WhiteLevel — that is develop (STAGE-007+) and
  would break the bit-exact plane
- Multi-strip assembly — SPEC-028
- The bit-exact oracle — SPEC-025
- Removing the layer0 packed-bits check (`width*height*bits == strip bits`)

## Notes for the Implementer

- **Do NOT clamp.** The bit-exact plane must carry raw values (including above-white),
  or SPEC-025's `dnglab` checksum can't match. Clamp/scale to WhiteLevel is develop's
  job (STAGE-007+).
- **The DEC is the weighty part** — it reverses DEC-008's "permanent invariant"
  Consequence. Write it as an amendment (amend that one clause; DEC-008's two-path
  byte-alignment rule stays). Cite the Canon sample (15464 > 15000) as the falsifier
  and name SPEC-025 as the replacement byte-swap guard.
- Independent of SPEC-028 (strip layout) — both block SPEC-025; either can build first.
- Keep `unpack_into` panic-free and the two DEC-008 unpack paths unchanged — only the
  post-unpack WhiteLevel assertion changes.

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
