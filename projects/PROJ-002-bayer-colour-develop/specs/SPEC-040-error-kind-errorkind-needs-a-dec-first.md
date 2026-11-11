---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-040
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
  decisions: [DEC-051]             # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, library-not-application]
  related_specs: [SPEC-039]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-039]                     # e.g. [SPEC-002]

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
      handoff: HANDOFF-119
      agent: claude-opus-5
      interface: other
      tokens_total: 5756260
      estimated_usd: 4.20
      duration_minutes: 25
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input 82 + cache write 122,174 + cache read 5,608,811 + output 25,193, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns. synced_at left null for handback-sync."
    - cycle: verify
      handoff: HANDOFF-121
      agent: claude-opus-5-5
      interface: other
      tokens_total: 1651196
      estimated_usd: 1.44
      duration_minutes: null
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 verify. Verdict APPROVED at de30009 (PR #182), no code change. 512 tests pass, lint green, all 55 variants match DEC-051 (12 CallerError / 18 Unsupported / 25 Corrupt), the four judgment calls hold. Proved the no-wildcard guarantee in a scratch copy: a dummy variant fails E0004 in kind() even after being given a Display arm. tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  totals:
    tokens_total: 7407456
    estimated_usd: 5.64
    session_count: 2
---

# SPEC-040: **`Error::kind()` / `ErrorKind`** (needs a DEC first):

> **DESIGNED 2026-11-10 (DEC-051), `cycle: build`.** The three-kind classification
> rubric and the full per-variant table live in **DEC-051** — read it first; this spec
> is the build contract against it. SPEC-039 shipped, so the last new failure paths
> (`PlaneTooLarge`, `StripBytesExceedFile`) are present to classify.

## Context

Source `feedback/2026-11-02-pyirradiance-requests.md` item 3. A consumer wants to separate "unsupported format, skip it" from "corrupt or hostile file". `Error` (`src/lib.rs:143`, `#[non_exhaustive]`, ~40 variants) mixes unsupported-but-valid input, corrupt input and caller misuse (`PlaneBufferWrongLength`, `*WrongLength`), so a boolean `is_unsupported` would report caller bugs as hostile files. Needs a DEC before the build (next free DEC-035). Sequenced after SPEC-039 because that spec adds the last new failure paths the table must classify.

## Goal

Decide and add a stable `Error::kind() -> ErrorKind { Unsupported, Corrupt, CallerError }` (`#[non_exhaustive]`), with an exhaustive-in-tests classification so a new `Error` variant cannot land unclassified.

## Inputs

- **DEC-051** — the rubric, the full per-variant table, the wildcard-free-match decision, and the four flagged judgment calls. Read first.
- **`src/lib.rs`** — the `Error` enum (`#[non_exhaustive]`, ~55 variants) and its `Display`; add `ErrorKind` + `impl Error { pub fn kind(&self) }` here.
- **`tests/`** — where the exhaustive classification test lands; `tests/support/` for any constructor helpers.

## Outputs

- **Files modified:** `src/lib.rs` (new `ErrorKind` enum + `Error::kind()`); `CHANGELOG.md` (`[Unreleased] → Added`); `docs/provenance-ledger.md` (one row — own prior code, class 3, a classification over this crate's own `Error`).
- **Files created:** `tests/error_kind.rs` — the exhaustive per-variant classification test.
- **New public API:**
  - `#[non_exhaustive] pub enum ErrorKind { Unsupported, Corrupt, CallerError }` (derive `Debug, Clone, Copy, PartialEq, Eq`).
  - `impl Error { pub fn kind(&self) -> ErrorKind }` — a `match self { … }` with **no `_ =>` arm**.
- **No new flags; no new `Error` variant; no `Display` change; no change to how any error is produced.**

## Acceptance Criteria

- [ ] `ErrorKind { Unsupported, Corrupt, CallerError }` is public and `#[non_exhaustive]`; `Error::kind(&self) -> ErrorKind` is public.
- [ ] `kind()` is a wildcard-free `match` over every variant — **adding a dummy variant to `Error` makes `kind()` fail to compile** (the implementer confirms this locally and notes it in the handback; the shipped code has no dummy).
- [ ] Every current variant is classified exactly as DEC-051's table says; the classification test constructs a representative of **each** variant and asserts its kind (a variant missing from the test is a gap the test itself surfaces).
- [ ] The three kinds map a consumer's decision: `Unsupported` ⇒ skip, `Corrupt` ⇒ bad file, `CallerError` ⇒ caller bug; the SPEC-039 additions `PlaneTooLarge` (`CallerError`) and `StripBytesExceedFile` (`Corrupt`) are classified per DEC-051's judgment calls.
- [ ] `just test` (`--all-features`) and `just lint` pass; no panic path added (`kind()` is total and infallible); `library-not-application` holds (no binary surface).
- [ ] No existing test changes behaviour; `Display` text and every error's production site are untouched.

## Failing Tests

- **`tests/error_kind.rs`**
  - `"every_variant_classified"` — constructs one representative of each `Error` variant and asserts `.kind()` equals DEC-051's assignment (grouped `Unsupported` / `Corrupt` / `CallerError` lists).
  - `"the_three_buckets_are_distinct"` — asserts a known `Unsupported` (e.g. `UnsupportedCompression`), a known `Corrupt` (e.g. `Truncated`), and a known `CallerError` (e.g. `PlaneBufferWrongLength`) return three different kinds.
  - `"spec_039_additions_classified"` — asserts `PlaneTooLarge` ⇒ `CallerError` and `StripBytesExceedFile` ⇒ `Corrupt`.
  - *(No red-proof needed: the compiler's exhaustiveness is the guarantee that a new variant cannot land unclassified — stronger than a mutation test. The build reflection must still report that it confirmed the compile-time failure on a throwaway dummy variant.)*

## Non-Goals

- No new `Error` variants and no change to existing variants or `Display` text
- No `is_unsupported` boolean alias (DEC-051 rejected it)
- No per-variant stability promise beyond the three kinds (a variant may be re-classified later)
- No change to how any error is produced
- No 4th kind for 0.2.0 (DEC-051); the release (SPEC-042) and determinism (SPEC-041) are separate

## Notes for the Implementer

- Read DEC-051 first; re-derive the per-variant table against the live `src/lib.rs` enum (it may have drifted since 2026-11-10) and classify any variant the table missed by the rubric, flagging it in the handback.
- `kind()` has **no** `_ =>` arm — that is the load-bearing choice; do not add one even to silence a transiently-incomplete match.
- `just test` already bakes in `--all-features`; do not pass the bare flag.

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
