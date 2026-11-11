---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. This is the BUILD half of SPEC-039 (group E).
# The verify half will be a later handoff. `handoff.cycle` distinguishes them.
#
# The `handback:` block is the RETURN path for cost; do not leave it unfilled.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-117
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.build — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-039

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Report the REAL token
# number from your own interface. If your platform exposes none, set
# tokens_total: null AND say why in notes. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 19609670         # REAL combined count — what cost-audit reads
  estimated_usd: 13.34
  duration_minutes: 75
  branch: claude/spec-039-build-4c9ff9
  pr: 178
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns. (synced_at was reset to null at ship: the build session stamped it but the cost never reached SPEC-039's cost.sessions, so handback-sync re-transcribes it here.)"
  synced_at: 2026-11-10
---

# HANDOFF-117: Build the unified validate-before-allocate preflight (SPEC-039, group E)

## Delegation Summary

The architect hands `SPEC-039` to the implementer for the **build** cycle: add
`pub fn plane::validate(sensor, byte_order, file, max_pixels) -> Result<usize, Error>`
— one allocation-free, panic-free pre-allocation gate covering **both** the uncompressed
and the Compression-7 path, with a caller-selected pixel budget checked first — and
refactor `unpack_into` onto the same shared checks so the two cannot drift. Also ratify
EOI-optional with a test, add the `ifd`/`plane`/`ljpeg` fuzz smoke jobs, extend the
`plane` fuzz target to the validate contract, and reconcile the stale docs. The whole
decision is **DEC-050**. **Read SPEC-039 and DEC-050 in full first** — `## Design` has
the budget-first ordering and the precedence-preserving refactor, which are the
non-obvious parts.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-039-plane-validate-validate-before-allocate-extract.md` — all of it, especially `## Design`, `## Acceptance Criteria`, `## Failing Tests`.
- **Decision:** `./decisions/DEC-050-unified-validate-before-allocate-preflight-and-caller-selected-pixel-limit.md` — the contract and the rejected alternatives (do NOT re-litigate: no 3-arg structural-only `validate`, no library-chosen cap, no decode inside `validate`, no required EOI, and the two group-C follow-ups stay deferred).
- **Code:** `src/plane.rs` (`unpack_into` `:278`, `unpack_uncompressed`, `unpack_jpeg` `:526`, `check_strip_shape` `:475`, the compression dispatch `:290`, `SUPPORTED_BITS`); `src/ifd.rs` (`Sensor` fields, `packed_bits()`, `MAX_TAG_VALUES`); `src/lib.rs` (`Error` `#[non_exhaustive]` `:143` + `Display`); `src/ljpeg.rs` (`parse_headers` returns at `SOS`, the scan completes on sample count, `MARKER_EOI` — for the EOI test/doc); `fuzz/fuzz_targets/plane.rs`; `.github/workflows/ci.yml` (the `fuzz-opcode` job `:253` to mirror); `examples/fuzz-seeds.rs`.
- **Docs to reconcile:** `README.md` (status `:42-45`, PROJ-003 scope `:157`), `docs/conformance-matrix.md` (Leica M Typ 246 + Pentax K-3 III rows), `CHANGELOG.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap (DEC-004 rule 5).

> ⚠ Line numbers were read 2026-11-10 against `main` after A–D landed; re-confirm
> before editing in case of drift.

### Decisions that apply

- `DEC-050` — the contract this builds.
- `DEC-016` — zero-allocation discipline (`validate` and the shared helpers allocate nothing).
- `DEC-047` (SPEC-053) — the decoded-frame shape binding stays a runtime decode check; `validate` does not duplicate or pre-empt it.
- `DEC-045` — the re-plan; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus never committed; the bit-exact + no-new-error corpus check is a local run, recorded in the handback.

### Constraints that apply (see `./guidance/constraints.yaml`)

- `no-panics-on-untrusted-input` — five deny lints; the budget product is `u64` with checked arithmetic; no indexing/slicing on hostile strip tables.
- `oracle-must-be-shown-red` — the three red-proofs (coverage, aliasing, budget removed) are load-bearing, shown red before green.
- `library-not-application` — `max_pixels` is the caller's number; the library adds no cap of its own.

### Prior related work

- `SPEC-052` (A) — the IFD-parse resource guard, upstream of `Sensor`; `validate` composes with it, does not duplicate it.
- `SPEC-053` (B) — `check_strip_shape`, the decoded-frame shape binding `validate` relies on at decode time.
- `SPEC-054` (C) — the required-processing check; its two verify follow-ups are **deferred**, not part of this build.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `pub fn plane::validate(sensor, byte_order, file, max_pixels) -> Result<usize, Error>` — allocation-free, panic-free, both paths, budget checked first (DEC-050 §2–§5).
- `Error::StripBytesExceedFile { declared: u64, file_len: usize }` and `Error::PlaneTooLarge { pixels: u64, limit: usize }` + their `Display`.
- `unpack_into` refactored onto the shared helpers with error precedence preserved (`PlaneBufferWrongLength` still wins).
- EOI-optional test + a one-line doc on the decode entry (no code change to the decoder).
- `tests/plane_validate.rs` with every `## Failing Tests` case; the three red-proofs shown red first.
- `fuzz/fuzz_targets/plane.rs` extended; `ifd`/`plane`/`ljpeg` 60 s CI smoke jobs added (mirror `fuzz-opcode`).
- README + CHANGELOG + conformance-matrix reconciled; a tested-DNG-modes compatibility table; a `docs/provenance-ledger.md` row.
- Tier-B corpus bit-exact and trips neither new error (local run, recorded in the handback).
- A PR against `main` from `claude/spec-039-build-<suffix>` referencing HANDOFF-117, SPEC-039, STAGE-010, PROJ-002, DEC-045 / DEC-050.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass (`just test`, `just lint`, the fuzz smoke)? Build reflection answered honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- §12 behavioural surface (AGENTS.md §12): confirm the **budget check fires first on BOTH paths** (not just uncompressed), `validate` and `unpack_into` genuinely share the checks (mutate a layout, see both change), the uncompressed bound holds under aliased strips, a Compression-7 plane validates structurally **and** the shape binding still fires at decode, and EOI-optional holds — exercised, not just shape-checked.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- `Error::kind()` (SPEC-040) and the 0.2.0 release (SPEC-042).
- Group C's two deferred follow-ups (malformed `OpcodeList` tag read as absent; stage functions skipping the required-processing check) — DEC-050 §Scope. A cheap safe fix → a new spec, not this handoff.
- Validate helpers for `demosaic_into`/`to_linear_f32_into`; any library cap on `n`.
- The parser guard (SPEC-052), the shape binding (SPEC-053), the opcode validation (SPEC-054).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (with a real `tokens_total`) and the
`## Handback

*Filled in by the receiving agent. The orchestrator transcribes, it does not reconstruct.*

### Execution notes

- **Branch / PR:** `claude/spec-039-build-4c9ff9` → https://github.com/jysf/irradiance/pull/178
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes. `plane::validate` (both paths, budget first,
  allocation-free, reads no strip bytes); `Error::StripBytesExceedFile` +
  `Error::PlaneTooLarge` + `Display`; `unpack_into` refactored onto the shared
  `plane_kind` / `pixel_product` / `StripWalk` / `check_strips` helpers with
  `PlaneBufferWrongLength` still winning; EOI-optional doc on `ljpeg::decode_into`
  + `eoi_optional` test (no decoder change); `tests/plane_validate.rs` (11 tests +
  3 red-proofs); `fuzz/fuzz_targets/plane.rs` on the
  `validate → allocate(capped) → unpack_into` contract + 3 new seeds
  (`forged-dims-uncompressed`, `forged-dims-compression7`, `aliased-strips`);
  `fuzz-ifd` / `fuzz-plane` / `fuzz-ljpeg` 60 s CI jobs; README (status block,
  usage now calls `validate`, PROJ-003 scope, new tested-DNG-modes table),
  CHANGELOG `[Unreleased]`, conformance matrix (Typ 246 + K-3 III rows) and a
  provenance-ledger row.
- **Red-proofs (control first, DEC-009), all shown red:**
  coverage removed → control `err PackedSizeMismatch { expected_bits: 256, strip_bits: 128 }`, mutant `ok 16`;
  aliasing removed → control `err StripBytesExceedFile { declared: 25600, file_len: 8320 }`, mutant `ok 12800` (which breaks `n·bits ≤ 8·len`);
  budget removed → control `err PlaneTooLarge { pixels: 8, limit: 4 }`, mutant `ok 8`.
- **Gates:** `just test` (with `IRRADIANCE_CORPUS_DIR` set) green: 22 test binaries, 507 passed, 0 failed, 0 corpus skips; `just lint` green
  (clippy `-D warnings` + `fmt --check`); `just lint-no-allow` green; fuzz-graph
  clippy green; `just fuzz-plane 120` → 12,168,303 runs / 121 s, no crash;
  `ljpeg` and `ifd` 30 s local smoke clean (ljpeg 1,976,793 runs, ifd 2,697,950 runs).
- **Tier-B corpus:** bit-exact, and trips neither new error. Local run of
  `corpus_files_validate_and_trip_neither_new_error` with `IRRADIANCE_CORPUS_DIR`
  set, 9/9 files present: the 8 DNGs (Leica Q2 Monochrom ×3, Leica M Monochrom,
  Leica M Typ 246 [Compression-7, Nf=2, MM], Pentax K-3 III DNG [Compression-7],
  Canon 5D Mark III, Nikon D300S 12-strip) each `validate(.., usize::MAX)` → `Ok(w·h)`
  and `unpack_into` → plane MD5 == the manifest's pinned `dnglab` checksum; the
  K-3 III PEF (`Compression 65535`) is refused by both with the same
  `UnsupportedCompression`. No file returns `StripBytesExceedFile` or
  `PlaneTooLarge`. The existing tier-B suites (plane/develop/colour oracles,
  `ljpeg_shape_binding`) also pass on the refactored `unpack_into`.

### Cost self-report

- **Tokens (total):** 19609670
- **Estimated USD:** 13.34
- **Duration (minutes):** ~75
- **Source of the number:** session transcript `usage` records (summed, deduped by message id)

### Drift and new artifacts

- **New decisions emitted:** none (DEC-050 built as written).
- **Deviations from spec:**
  - DEC-050 §3 lists "supported precision" among the Compression-7 pre-decode
    checks. `unpack_jpeg` runs no precision check before decoding (the precision is
    the SOF-3 frame's own `P`, inside the strip bytes `validate` must not read), so
    `validate` adds none either — adding one to `validate` alone would break the
    "same variant" contract, and adding it to `unpack_into` would change behaviour
    for shipped files. Precision stays a decode-time check.
  - Compression-7 error precedence among *strips*: per Design §5 ("the structural
    pass then decode"), every strip's layout/bounds/coverage is now checked before
    the first strip is decoded. Before, strip *k*'s decode error could win over strip
    *k+1*'s truncation, and an undershoot was reported after decoding. Only a file
    with both a bad strip table and a bad SOF-3 stream sees a different variant.
    Recorded in the CHANGELOG `Changed` entry.
  - `StripBytesExceedFile` is checked after the existing per-strip checks and
    coverage (so every pre-existing error keeps its precedence); it is shared by
    `unpack_into` — the "newly-rejected forged overlap" the spec's Outputs allow.
- **Follow-up work identified:** none new. (The deferred group-C follow-ups stay
  deferred per DEC-050 §Scope.) Minor: `cargo fmt --manifest-path fuzz/Cargo.toml`
  wants to reformat `fuzz/fuzz_targets/opcode.rs` — the fuzz graph has no fmt gate;
  left untouched.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — Only DEC-050
   §3's "supported precision" for Compression-7, which has no pre-decode form
   without reading strip bytes; resolved as above. `Sensor` being sealed (SPEC-054)
   meant tests had to build real TIFFs — a small general strip-table builder in the
   test file covered every case.
2. **Was there a constraint or decision that should have been listed but wasn't?** —
   No.
3. **If you did this task again, what would you do differently?** — Start the full
   corpus `just test` in the background earlier; it is the long pole (~25 min
   locally with the corpus present).
