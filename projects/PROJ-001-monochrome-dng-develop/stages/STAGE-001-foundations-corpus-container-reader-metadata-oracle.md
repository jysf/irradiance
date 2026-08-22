---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-001                     # stable, zero-padded, continuous across the repo
  status: shipped                 # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: 2026-08-22

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Proves the container half of the thesis: that a DNG's real structure and metadata can be read from a published spec, with an oracle that says so.
  delivers:
    - "A bounded, panic-free TIFF/IFD reader with SubIFD recursion"
    - "Typed DNG metadata extraction"
    - "A two-tier corpus with a manifest, and tests that skip cleanly when tier B is absent"
    - "A metadata oracle diffing our output against dnglab and exiftool"
  explicitly_does_not:
    - "Any pixel decode — no strip reading, no unpacking"
    - "Any vendor container — DNG only"
    - "Colour of any kind"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-001: Foundations: corpus, container reader, metadata oracle

## What This Stage Is

When this stage ships, `irradiance` can open a real Leica Q2 Monochrom DNG,
walk its IFD tree to the full-resolution **sensor IFD** — a `SubIFD` on every
file held except the Pentax PEF, whose plane sits in `IFD0` and which carries no
`SubIFDs` tag at all — and report every tag the develop
pipeline will later need — dimensions, bit depth, black and white levels, active
area, default crop, orientation, and the two opcode lists — with an oracle that
proves those values right rather than merely plausible. It reads no pixels.

**Estimated effort: 25–40 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

Everything downstream reads tags. Getting the container wrong silently
poisons every later stage, and the metadata oracle is the cheapest of the three
layers to stand up — `dnglab analyze --meta --json` and `exiftool` both already
produce machine-diffable output on files we hold.

The corpus lands here for the same reason: it is the project's top pre-mortem
risk, it cannot be delegated, and nothing after this stage is verifiable without
it. Front-loading it means the risk is retired while the code is still cheap.

## Success Criteria

- Our parsed tags match `dnglab analyze --meta --json` and `exiftool` on every tier-B file we hold
- The metadata oracle goes **red** on a deliberately corrupted tag — proven, not assumed
- The reader never panics on truncated, cyclic, or hostile input; a fuzz target exists and has run
- Tier-A fixtures run in CI; tier-B tests skip with a clear message when the corpus is absent
- Every offset and length read is bounds-checked and returns a typed error

## Scope

### In scope
- Two-tier corpus layout, manifest schema (path, hash, provenance, licence), skip-when-absent harness
- TIFF/IFD reader: byte-order handling, SubIFD recursion, depth and cycle guards
- DNG tag model and typed extraction for the tags in `docs/measured-q2m-dng.md`
- Metadata oracle harness
- The first fuzz target

### Explicitly out of scope
- Strip/tile reading and pixel unpacking (STAGE-002)
- Opcode *execution* — this stage only records that the lists are present (STAGE-003)
- Any write path; `irradiance` is read-only

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-001` to promote these outlines into real specs.

- [x] SPEC-001 (shipped on 2026-08-20) [S] Crate scaffold: Cargo.toml, measured MSRV, panic-free lints, Rust CI
- [x] SPEC-002 (shipped on 2026-08-20) [S] Corpus manifest reader and skip-when-absent harness
- [x] SPEC-003 (shipped on 2026-08-21) [L] TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target
- [x] SPEC-004 (shipped on 2026-08-21) [M] DNG tag model and typed metadata extraction
- [x] SPEC-006 (shipped on 2026-08-20) [S] Close the allow-attribute bypass in the panic-free gate — split from SPEC-002 at SPEC-001 ship; depends only on SPEC-001
- [x] SPEC-005 (shipped on 2026-08-22) [M] Metadata oracle: diff parsed tags against `dnglab analyze --meta --json` and `exiftool`, and prove it goes red — **the stage's own success criterion, and the last spec in this backlog.** Designed 2026-08-21; `HANDOFF-021` ready. Re-sized S→M at design: the probe found the two tools answer different questions, so the spec carries a per-tool scope, three asserted divergences and a two-tier red-proof.

- [x] SPEC-007 (shipped 2026-08-21) — extraction tolerance per DEC-012's amendment
- [x] SPEC-008 (shipped on 2026-08-21) [S] Pin the Structure class with tests that fail when it is softened

**Count:** 8 shipped / 0 active / 0 pending

## Design Notes

### Per-spec context (the detail deliberately kept OUT of the backlog titles)

`just frame-stage` derives each spec's **filename** from its backlog summary, so
summaries stay short. The constraints that shape each one live here.

**Crate scaffold** — nothing else in this stage can exist without it (AGENTS.md
§5). It must land, in one change: `edition = "2021"`; a `rust-version` **measured
from the real dependency set, never guessed** (a design-time probe, §12); the
panic-free clippy set (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
`arithmetic_side_effects`), allowed inside `#[cfg(test)]` and `src/bin/irr.rs`;
`#![forbid(unsafe_code)]`; and the Rust CI jobs — `fmt --check`, `clippy -D
warnings`, `test`, `cargo deny check licenses`. ⚠ Per DEC-003, CI **cannot** run
tier-B tests, so a green badge must not be read as "bit-exact".

**Corpus manifest reader** — storage and schema are already settled by DEC-003
and `tests/corpus/manifest.toml` ships seeded with three pinned frames. This spec
builds the thing that *reads* it: resolve `$IRRADIANCE_CORPUS_DIR`, verify
`sha256`, and **skip loudly, naming the missing file**, when a tier-B entry is
absent. Without this the manifest is exactly the unread field AGENTS.md §11
warns about — and that debt is already recorded in the manifest's own header.

**TIFF/IFD reader** — SPIKE-001 measured a working version at ~117 code lines
with zero dependencies, but that version is **discarded and must not be
consulted as an implementation** (see `spikes/done/SPIKE-001-*.md`); re-derive it
test-first. The guards it needs are known: depth limit, cycle detection on
visited offsets, bounds-checked payload ranges. Select the sensor IFD on
`NewSubfileType == 0 && PhotometricInterpretation == 34892 && SamplesPerPixel == 1`
— **never on largest dimensions**, because `SubIFD2` is a full-resolution JPEG
preview only 56 px narrower than the plane.

**DNG tag model** — the tag set is enumerated in `docs/measured-q2m-dng.md`.
⚠ `Orientation` is **per-frame, not a camera constant** (proved across our three
frames); read it from the file every time.

**Metadata oracle** — `dnglab analyze --meta --json` plus `exiftool`. Ships with
its red-proof (constraint `oracle-must-be-shown-red`): a corrupted tag must turn
it red. ⚠ Do **not** extend this layer to cover levels — DEC-004 settles that
levels are verified analytically, and SPIKE-001 measured why a comparison-based
check cannot do it.

**Implement from the Adobe DNG Specification, not from anyone's code.** The
spec is public and carries a patent grant for compliant implementations. Every
tag handler gets a provenance-ledger entry naming the spec section.

`crustyimg/src/metadata/tiff.rs` (718 lines) is a proven *design* to reference —
it is the maintainer's own code — but it cannot be reused: it is crustyimg-internal,
the dependency runs one way, and it is an order-preserving *serializer* for the
metadata lane. This reader is read-only and needs SubIFD reach for image data,
which that one does not have.

**Stopping point A:** real sensor dimensions, bit depth, levels, crop geometry,
orientation and opcode presence, read from the maintainer's own file.

## Dependencies

### Depends on
- External: `dnglab` and `exiftool` (tools, run — never linked)
- External: the Adobe DNG Specification
- SPIKE-001 — the corpus and the verified oracle contract

### Enables
- STAGE-002 — the plane decode reads its geometry and levels from here

## Stage-Level Reflection

*Shipped 2026-08-22.*

### Success criteria, checked rather than asserted

| # | criterion | verdict |
|---|---|---|
| 1 | parsed tags match `dnglab` **and** `exiftool` on every tier-B file | ⚠ **met with one measured exception.** `exiftool` on all **7**; `dnglab` on **6** — `K3III.PEF` is excluded **by name and with a written reason**, because dnglab's black/white/crop values for a vendor container come from rawler's camera database and exist in **no tag in that file**. The criterion assumed the two tools answer the same question. They do not. |
| 2 | the oracle goes **red** on a corrupted tag — proven, not assumed | ✅ two tiers, both directions, each with a negative control, watched failing by **two** parties |
| 3 | never panics on truncated/cyclic/hostile input; a fuzz target exists and has run | ✅ target exists, ran repeatedly (>50 M execs cumulative, zero surviving artifacts). ⚠ "never panics" remains **evidenced, not proven** — fuzzing plus a lint policy is not a proof |
| 4 | tier-A runs in CI; tier-B **skips with a clear message** when absent | ⚠ **met through `just test`, not through the harness.** Measured: `cargo test` reports **87 passed with the corpus and 87 without** — an identical count, because a skip returns early and counts as a pass. `just test` runs `examples/corpus-status.rs` first and names every missing file, which is why this is met at all. Bare `cargo test` tells you nothing |
| 5 | every offset and length read is bounds-checked, typed error | ✅ in the code, and mechanically enforced **locally**. ⚠ **Not enforced in CI for this stage's entire duration** — see below |

### The finding this stage should be remembered for

**The gate that mechanically enforces `no-panics-on-untrusted-input` was dark in
CI from the day it was created until the day this stage closed.** `PATCH-001`
measured it: 17 consecutive red runs on `main` from `ship(spec-001)` onward, the
red-proof job failing in **all 17**. Every verify cycle in that window reported
"ten gates green" — and every one was telling the truth about the **local** gates.
Nobody read CI.

Criterion 5 was therefore true of the code and false of the process, for eight
specs. That is the single most valuable thing this stage produced, and it was
found by accident while pushing an unrelated PR.

### Answers

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** The library
  opens a real Leica Q2 Monochrom DNG, walks to the sensor IFD, and reports every
  tag the develop pipeline needs — now diffed **live** against two external tools
  rather than a hand-typed table. It reads no pixels, as scoped. Four camera
  bodies, seven files, `src/` byte-identical across SPEC-005's four cycles.
- **How many specs did it actually take?** **8 specs + 1 patch**, against a
  backlog of **6**. SPEC-006 was split out of SPEC-002 at SPEC-001's ship;
  SPEC-007 and SPEC-008 were *both* generated by follow-ups from SPEC-004 and
  SPEC-007 respectively — the stage grew a third of its length from its own
  review findings, which is a feature of the review loop and a cost nobody
  estimated.
- **How many outlines survived unchanged?** **4 of 6.** SPEC-002 lost its
  allow-attribute half to SPEC-006; SPEC-005 was re-sized `S`→`M` at design when
  its probe found the two oracle tools answer different questions.
- **What changed between starting and shipping?** The oracle stopped being a
  thing we *wrote down* and became a thing we *run* — and in the process the
  stage learned that its own gates needed the same treatment.
- **Complexity calibration — the number worth keeping:** `SPEC-001` was estimated
  **S** and landed **XL**, a four-notch miss on the very first spec, and every
  later estimate was within one notch (`S→M`, `M→L`, five exact). The scaffold
  spec is the one nobody can size, because it is where every unmeasured
  assumption about the toolchain gets paid at once.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - ✅ **Three landed in §16 at this close**, all at or past bar:
    `measurement-over-generalised` (N=5, as a *writing rule* — instance 5 was
    committed by an agent who had just enforced it on someone else),
    `attribute-text-inside-doc-comments` (N=5), and
    `a-gate-that-fails-mutely-is-a-gate-that-never-ran` (N=4, with a measured
    false green among its instances).
  - ✅ **`AGENTS.md` §15 gained a follow-up disposition policy** mid-stage — 34
    findings had accumulated across handbacks with no index, no status and no
    owner. Every finding since has had exactly one of four dispositions.
  - ⚠ **`unrun-docs-carry-errors` deliberately NOT codified**, still N=2. The bar
    did its job; no third instance appeared. Drop it at STAGE-002's close if it
    is still at N=2.
  - ⚠ **Constraint `no-panics-on-untrusted-input`'s enforcement text should say
    that a gate's CI job must be *observed green*, not merely present.** The text
    already says "do not delete either job without replacing its guarantee" — it
    did not anticipate a job that exists, runs, and has never once passed.

### Follow-up work, and where it goes

Nothing carries into a new stage. Three specs are framed in **STAGE-002**, each
the destination for dispositioned follow-ups: **SPEC-009** (pin the Structure
class's *membership* — measured: ten of eleven can be deleted with 66/66 still
green), **SPEC-010** (distinguish an unparseable tool reading from an absent tag;
its fix is already built and measured by SPEC-005/FU-8), **SPEC-011** (lint the
fuzz crate — the second cargo graph sits outside every `-D warnings` gate).

⚠ **One sweep is owed and is not a spec yet:** the mute-gate shape was measured
in `_lib.sh`, `decisions-audit.sh`, `ready.sh`, `release-notes.sh`,
`report_daily.sh` and `test.sh`. The rule is codified and `lint-red-proof.sh` is
clean; the other six are unaudited. That belongs in STAGE-002, not in a note.
