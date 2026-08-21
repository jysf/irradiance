---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-002
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
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
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-008, build cycle
  created_at: 2026-08-20

references:
  # Namespace matters (AGENTS.md §10): DEC-003 and DEC-010 are repo decisions
  # in /decisions/; DEC-004 rule 4 is the TEMPLATE's decision in /docs/decisions/.
  decisions: [DEC-003, DEC-010, docs/DEC-004]
  constraints:
    - no-new-top-level-deps-without-decision
    - no-copyleft-dependencies
    - provenance-recorded-per-algorithm
    - library-not-application
    - oracle-must-be-shown-red
    - test-before-implementation
  related_specs: [SPEC-001, SPEC-003, SPEC-004, SPEC-005]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-001]                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling every corpus-dependent test"

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
      agent: claude-opus-5
      interface: other
      tokens_total: 9498150
      estimated_usd: null
      duration_minutes: 30
      recorded_at: 2026-08-20
      notes: "Build cycle for SPEC-002 (HANDOFF-008). Reader + visible skip shipped; all seven gates green and pasted in the handback, including the criterion `just test 2>&1 | grep SKIP` demonstrated in three corpus states (0/7, 6/7, 7/7 present) plus an adversarial mutation that takes it to 0 lines. One dev-dependency added (toml, DEC-010); [dependencies] still empty. SHA-256 written from FIPS 180-4 rather than taken as a second dep, with a provenance-ledger row. tokens_total is REAL but not from /cost, which is a client-side slash command the assistant cannot execute; I summed the usage objects in this session's own transcript (~/.claude/projects/-Users-...-irradiance/dbdeb6a8-....jsonl). Composition: input 150 + output 65,303 + cache-write 159,959 + cache-read 9,272,738 (97.6% cache-read). FLOOR - written before the session ends. IMPORTANT, and it breaks comparability with every SPEC-001 figure: this number is DEDUPED BY message.id. A transcript writes one jsonl line per content block and repeats the same usage object on each, so the raw sum every SPEC-001 session used double-counts multi-block messages - measured 1.7x on SPEC-001's own verify-4 transcript (116 raw objects, 67 distinct ids, raw 14,177,812 vs deduped 8,053,949, recorded 10,962,512). Signal token-counts-not-comparable updated with the measurement; SPEC-001's cost.totals of 51,979,929 should be re-summed with dedup rather than left standing."
  totals:
    tokens_total: 9498150
    estimated_usd: 0
    session_count: 1
---

# SPEC-002: Corpus manifest reader and skip-when-absent harness

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-002]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

DEC-003 settled corpus storage and `tests/corpus/manifest.toml` already ships
seeded with three pinned Q2 Monochrom frames — path, size, sha256, licence,
source, and the pinned `dnglab --raw-checksum`. **Nothing reads it.** Its own
header records that as a scheduled debt owned by this spec.

## Goal

The reader. Resolve `$IRRADIANCE_CORPUS_DIR` (defaulting to the gitignored
`tests/corpus/tier-b/`), verify each entry's `sha256`, and **skip loudly —
naming the missing file** — when a tier-B entry is absent.

A silent skip reports green for work it never did, which is the same defect
class as an oracle that cannot go red. Nothing else in this stage may hardcode
a corpus path.

## Inputs

- **Files to read:** `tests/corpus/manifest.toml` — the thing being read, 7
  `[[file]]` entries; `decisions/DEC-003-corpus-storage-and-manifest.md` — why
  tier B is external and why the skip must be loud;
  `guidance/toolchain-brief.md` — the `+toolchain` trap and the installed set.
- **External APIs:** none. `toml` 0.8 (`docs.rs/toml/0.8.23`) as a
  dev-dependency; no auth, no network, nothing at runtime.
- **Related code paths:** `src/lib.rs` (untouched — the reader cannot live
  there; see DEC-010), `app.just`, `.github/workflows/ci.yml`.

## Outputs

*Filled in at build with what was actually produced.*

- **Files created:**
  - `tests/support/corpus.rs` — the reader + SHA-256. Shared by the test binary
    and the example via `#[path]`; not part of the library.
  - `examples/corpus-status.rs` — the **visible** surface. One line per entry.
  - `tests/corpus_manifest.rs` — the spec's failing tests, plus the red-proof.
  - `decisions/DEC-010-toml-as-a-dev-only-dependency.md`.
- **Files modified:**
  - `Cargo.toml` — `[dev-dependencies] toml` with the measured feature set.
    `[dependencies]` stays empty.
  - `app.just` — `test:` runs `corpus-status` **before** `cargo test`.
  - `.github/workflows/ci.yml` — same ordering in the `rust / test` job, so the
    skip is in CI logs too.
  - `tests/corpus/manifest.toml` — header debt closed; records what the reader
    now requires of an entry.
  - `docs/provenance-ledger.md` — the SHA-256 row (class 1, FIPS 180-4).
- **New functions / types:** `Manifest::{load, parse, get}`,
  `CorpusRoot::{resolve, at, origin}`,
  `CorpusFile::{resolve, status, require, verify}`, `Status::{Present, Missing}`,
  `Oracle`, and `sha256::{Sha256, hash, hash_file, to_hex}`.
- **New flags / options:** `$IRRADIANCE_CORPUS_DIR` — any directory path.
  **Default when unset or empty: `<crate root>/tests/corpus/tier-b`**, resolved
  from `CARGO_MANIFEST_DIR` so the working directory cannot change the answer.
- **Database changes:** none.

## Acceptance Criteria

1. A reader parses `tests/corpus/manifest.toml` and exposes each entry's path,
   `sha256`, and `oracle.raw_checksum`. **No test in this repo hardcodes a corpus
   path from here on.**
2. `$IRRADIANCE_CORPUS_DIR` resolves the root, defaulting to the gitignored
   `tests/corpus/tier-b/`.
3. Each present file's `sha256` is **verified**, and a mismatch fails loudly and
   names the file (a re-download or truncation must not pass silently).
4. **A missing tier-B file skips — and the skip is VISIBLE under plain
   `cargo test`.** ⚠ Measured at design: `eprintln!`/`println!` inside a *passing*
   test is captured and shows **nothing** without `--nocapture`. A skip nobody can
   see is the silent-green defect this spec exists to prevent, so the visible
   surface must live outside the test harness — see Implementation Context.
5. The `toml` dev-dependency is accompanied by a `DEC-*` recording it (constraint
   `no-new-top-level-deps-without-decision`, as narrowed by DEC-004 rule 4).
6. `cargo deny check licenses` still passes; MSRV 1.90 still holds; all existing
   gates green.

## Failing Tests

```bash
# 1. the reader exists and reads all 7 manifest entries
cargo test --all-features corpus_manifest_parses

# 2. sha256 mismatch is caught (plant a corrupted copy, expect failure)
cargo test --all-features corpus_hash_mismatch_fails

# 3. THE ONE THAT MATTERS — a missing file's skip is visible with NO extra flags
just test 2>&1 | grep "SKIP"        # must print, naming the absent file
```

Test 3 is the acceptance criterion in executable form. It currently fails because
nothing prints, and it will keep failing if the skip is implemented as `eprintln!`
inside a passing test.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- **Any decoding.** No TIFF walk, no tag model, no unpack — SPEC-003 onward.
- **The `#[allow]` bypass in the panic-free gate.** That is SPEC-006; it shares
  no files with this spec.
- **Re-opening DEC-003's storage or schema decisions.** The manifest's shape is
  settled; this spec reads it.
- **A runtime (non-dev) dependency of any kind.** See DEC-010.
- **Reading `[[wanted]]` / `[[available]]`.** They are a human-facing backlog.
  The reader parses `[[file]]` only, and the manifest header now says so.

## Notes for the Implementer

### Two things were measured at design. Do not re-derive them.

**1. `toml` as a dev-dependency — costs and limits, measured on this crate.**

| config | crates in graph | parses? |
|---|---|---|
| `toml = "0.8"` | **12** | yes |
| `default-features = false, features = ["parse"]` | **11** | yes |
| `default-features = false` | 6 | **NO** — `Value: FromStr` unsatisfied |

That last row is a trap: `cargo check` passes because nothing calls the API. It is
a shape-check, not a behaviour-check (AGENTS.md §12). **Use
`features = ["parse"]`** — one crate cheaper than default, and it actually parses.

Also measured with the dep present: `cargo +1.90.0 check --all-targets` → **0**,
and `cargo deny check licenses` → **licenses ok**. It is **dev-only**, so the
library's zero-dependency public claim is untouched — say so explicitly in the DEC
(criterion 5), because "irradiance has no dependencies" must remain true as
written.

**2. `eprintln!` in a passing test is INVISIBLE.** Measured:

```
cargo test              -> 0 SKIP lines
cargo test -- --nocapture -> 2 SKIP lines
```

So criterion 4 **cannot** be met inside the test harness alone. The recommended
shape: a small corpus-status step that `just test` runs **before** the suite,
printing one line per manifest entry (present / MISSING with its path), so the
information is visible with no flags and in CI logs. The in-harness skip then just
returns early — the loudness lives where it can actually be seen.

Do not "solve" this by making `just test` pass `--nocapture` globally; that buries
the signal in full test output rather than surfacing it.

### Scope discipline

This spec builds the **reader** and the visible-skip surface. Storage and schema
are settled (`DEC-003`); the manifest ships seeded with 7 entries. The
`#[allow]`-bypass work is **SPEC-006**, a separate spec sharing no files.

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
