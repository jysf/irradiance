---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-041
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-003, DEC-016, DEC-018, DEC-033]   # determinism rests on these
  constraints: [oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-027, SPEC-030, SPEC-031, SPEC-039]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-010's <capability>". Optional; null is acceptable.
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
  tokens_estimate: null
  sessions:
    - cycle: build
      handoff: HANDOFF-120
      agent: claude-opus-5
      interface: other
      tokens_total: 7213996
      estimated_usd: 5.16
      duration_minutes: 12
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5, transcript-summed (bd02789d). Committed hashes HELD on ubuntu-latest x86_64 and macos-latest arm64, dev and release profiles (CI run 42947290870). No kernel change, no new dep."
    - cycle: verify
      handoff: HANDOFF-122
      agent: claude-opus-5-5
      interface: other
      tokens_total: 2311488
      estimated_usd: 1.83
      duration_minutes: null
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 verify. Verdict APPROVED at f307abe (PR #181, superseded by the rebased #183), no code change. Tests 514/0, lint green, both CI determinism legs ran on their real arches (ubuntu x86_64 + macos arm64), the red-proof bites (a one-bit sample flip moves all three hashes), and the hashes reproduced locally. tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  totals:
    tokens_total: 9525484
    estimated_usd: 6.99
    session_count: 2
---

# SPEC-041: **Cross-arch determinism evidence**: tier-A synthetic Bayer

> **DESIGNED 2026-11-10, `cycle: build`.** No new DEC — the determinism claim rests on
> DEC-016 (zero-alloc), DEC-018 (intent casts, no silent truncation), DEC-033
> (committed-constant `f64`, basic ops only, no FMA/transcendental) and DEC-003
> (tier-B never in CI). Independent of the other STAGE-010 specs.

## Context

Source `feedback/2026-11-02-pyirradiance-requests.md` item 4. pyirradiance will not claim bit-identical output across OS/arch until verified. CI is `ubuntu-latest` only, and the Canon 5D III file is tier-B corpus that never reaches CI (DEC-003), so the ask as phrased cannot be met. Float exposure is read, not measured: `to_linear_f32_into` is a single IEEE division and demosaic is integer arithmetic.

## Goal

Commit golden hashes of `unpack_into`, `demosaic_into` and `to_linear_f32_into` output over tier-A synthetic Bayer fixtures, assert them on a linux x86_64 and macOS arm64 CI matrix, and name any float path that could diverge. The oracle must be shown red (a perturbed hash fails).

## Inputs

- **`tests/support/tiff.rs`** — the tier-A synthetic CFA/Bayer builders (SPEC-027) and the uncompressed plane builders; the fixtures are built here, not held files.
- **`src/plane.rs`** (`unpack_into`), **`src/demosaic.rs`** (`demosaic_into`), **`src/tone.rs`** (`to_linear_f32_into`) — the three outputs to hash.
- **`.github/workflows/ci.yml`** — the existing `ubuntu-latest` jobs; add the two-OS matrix job here.
- **DEC-018 / DEC-033** — why the maths is cross-arch stable (integer + committed-constant `f64` basic ops, no FMA, no transcendental); **DEC-003** — why tier-B stays out of CI.

## Outputs

- **Files created:** `tests/determinism.rs` — builds the tier-A synthetic Bayer fixtures, runs the three outputs, hashes each with a small in-test FNV-1a over the raw output bytes (`u16` as LE bytes; `f32` as `to_le_bytes`), and asserts each equals a committed `const` hash. No new dependency (the hash is a few lines in the test).
- **Files modified:** `.github/workflows/ci.yml` (a `determinism` job with `strategy.matrix.os: [ubuntu-latest, macos-latest]` — macOS runners are arm64 — running `cargo test --test determinism`); `CHANGELOG.md` (`[Unreleased] → Added`: committed determinism hashes + the two-OS matrix); `docs/determinism.md` **or** a section in an existing doc naming the float paths; `docs/provenance-ledger.md` (one row).
- **No new public API, no kernel change.**

## Acceptance Criteria

- [ ] `tests/determinism.rs` builds tier-A synthetic Bayer fixtures (at least one RGGB and one other phase; 8/12/14/16-bit where the output differs) and asserts committed `const` FNV-1a hashes of `unpack_into`, `demosaic_into` and `to_linear_f32_into` outputs.
- [ ] A CI job runs that test on **`ubuntu-latest` (x86_64) and `macos-latest` (arm64)**; a divergence on either arch fails the job (that cross-arch agreement is the evidence).
- [ ] **Oracle shown red** (`oracle-must-be-shown-red`, DEC-009 control first): a perturbed run — a one-bit change to a fixture sample, or a locally-mutated kernel — produces a different hash and fails the assertion, shown red before the committed hashes go green.
- [ ] The float paths that could in principle diverge are **named** in the doc: `to_linear_f32_into` is a single IEEE `f32` division (`sample / 65535.0`, correctly-rounded, identical on both arches — no FMA, no transcendental); the colour path (`src/color.rs`, `f64`, STAGE-008) is named as an **unmeasured** path (committed-constant `f64` basic ops per DEC-033, out of scope here). `unpack_into` and `demosaic_into` are integer arithmetic — no float.
- [ ] `just test` and `just lint` pass; no new dependency; `library-not-application` holds.
- [ ] If any output diverges between arches, the build does **not** paper over it — it stops and reports it as a new spec (Non-Goals), it does not change a kernel to force agreement.

## Failing Tests

- **`tests/determinism.rs`**
  - `"unpack_into_hash_matches_committed"` — the `u16` plane bytes of each fixture hash to the committed constant.
  - `"demosaic_into_hash_matches_committed"` — the interleaved RGB `u16` bytes hash to the committed constant.
  - `"to_linear_f32_into_hash_matches_committed"` — the `f32` LE bytes hash to the committed constant.
  - `"determinism_red_proof"` — a perturbed sample (or mutated local kernel) yields a hash **different** from the committed one (control run first, per DEC-009), proving the assertion bites.
- **`.github/workflows/ci.yml`** — the two-OS matrix job is the cross-arch half; it has no unit-test file of its own.

## Non-Goals

- No tier-B corpus file in CI (corpus-file hashes are recorded for local runs only)
- No colour path (`f64`, STAGE-008) unless it is named as an unmeasured float path
- No change to any kernel; if a divergence is found it becomes its own spec
- No extra OS/arch beyond the two requested

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities. Keep short — the full
context graph lives in the handoff file.

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
