---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-072
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-30
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-027

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 65438120           # REAL combined count — what cost-audit reads
  estimated_usd: 146.41            # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/spec-027-build-b56d8c
  pr: https://github.com/jysf/irradiance/pull/52
  completed_at: 2026-10-30          # YYYY-MM-DD
  notes: "test-only — no src/ change, no new DEC. Registered both Bayer DNGs in tests/corpus/manifest.toml with MEASURED bytes/sha256 (Canon 5G4A9394.DNG: 48382640 bytes, sha256 95dc76a2f255e623acaa5149b320389f7da7c995da890c3e0d511c0ff681b2e1; Nikon _DSC4615.DNG: 37562890 bytes, sha256 c2fefd96e0125f5d5b891dff66b9ac673d6f0059db21c3a9dd469fe460f88bb7), raw_checksum re-confirmed against dnglab 0.7.2 (Canon a18235d6abced4df9948b6fbe91aba7c, Nikon b8e8025d370cc2e97f675e63a0e6bbdb), licence CC0-1.0 (raw.pixls.us; Canon Adobe-written DNG, Nikon dnglab-converted NEF — provenance corroborated by each file's own Software tag). Folded the Bayer oracle onto the manifest (DECODABLE 4->6, partition 6+3==9, deleted the bespoke Bayer oracle test + CANON/NIKON_RAW_CHECKSUM consts + require_bayer; red-proof now manifest-sourced). Migrated the SPEC-028/029 plane_unpack tests to manifest.get()+require(). Added a synthetic CFA fixture family to tests/support/tiff.rs (valid_cfa_sensor + cfa_pattern_past_eof) emitted into fuzz/seeds/ifd via tiff::all(), asserted in tests/ifd_reader.rs. THREE interlocks the design did not spell out, each handled minimally and documented inline: (1) bumped MANIFEST_ENTRIES 7->9; (2) ifd_reader's manifest==EXPECTED partition — the CFA files are mono-oracle-incompatible (12-strip Nikon, photometric 32803), so added CFA_CHECKED_ELSEWHERE; (3) metadata_oracle iterates all manifest files and would panic on an unmapped Bayer file — skipped them as SPEC-026's domain (an explicit Non-Goal). One design-wording deviation: the hostile CFA-tag variant is COSTED (DEC-012), not a typed Err, so it is asserted no-panic + costed rather than as an error. tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated."
  synced_at: 2026-10-30
---

# HANDOFF-072: Corpus + fuzz: register the uncompressed Bayer DNG in the corpus manifest and extend the fuzz corpus; panic-free on CFA input

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-027` to
`claude-opus-4-8` (acting as implementer) for the **build** cycle — register the
two uncompressed Bayer DNGs in `tests/corpus/manifest.toml`, fold SPEC-025's
Bayer plane oracle onto the manifest, migrate the SPEC-028/029 corpus tests to
the manifest, and add a synthetic CFA fuzz seed + a tier-A panic-free-on-CFA
test. STAGE-006 housekeeping; test-only, no `src/` change, the pinned checksums
do not move.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-cfa-sensor-model-and-bit-exact-mosaic-plane.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-027-corpus-fuzz-register-the-uncompressed-bayer-dng-in-the-corpus-manifest-and-extend-the-fuzz-corpus-panic-free-on-cfa-input.md`
  — Acceptance Criteria + Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps,
  `just lint-ci` for CI's clippy, the five-lint panic policy, the corpus env var.
- **The manifest reader (SPEC-002):** `tests/support/corpus.rs` —
  `Manifest::load`/`get`, `CorpusFile::require`/`verify`, the required-field set.
  `CorpusFile` ignores extra TOML keys (`make`/`model`/`bits`/`provenance`/
  `notes`), so only `path`/`tier`/`bytes`/`sha256`/`licence`/`source` +
  `[file.oracle] raw_checksum` are load-bearing.
- **The oracle apparatus reused verbatim:** `tests/plane_oracle.rs` — SPEC-013's
  `decode_and_hash`, `plane_md5`, `stage_probe_crate`, `build_and_run_probe`, and
  SPEC-025's `inject_byte_order_swap_fault`.

### Decisions that apply

- `DEC-003` — the tier-B corpus storage model: the manifest is committed, the
  files never are; licence/provenance has **no second chance**. The two new
  entries fill `licence = "CC0-1.0"` + a raw.pixls.us `source`/`provenance`.
- `DEC-012` — the interpretation-class rule: a malformed CFA tag costs that field
  alone (`color_filter -> None`, recorded in `malformed_tags`) and never fails
  the file. This is why the hostile CFA fuzz seed is *costed*, not a typed `Err`.
- `DEC-029` — samples above `WhiteLevel` are legal; the byte-swap guard was moved
  onto the plane oracle's red-proof (SPEC-025), which this spec keeps, now
  manifest-sourced.
- `DEC-017` — the red-proof mechanism (mutate a crate copy, prove the OUTPUT
  changed); reused unchanged.

### Constraints that apply

Full text in `./guidance/constraints.yaml`.

- `no-panics-on-untrusted-input` (blocking) — the new CFA parse path is proven
  panic-free on a well-formed mosaic, every truncation of it, and a hostile CFA
  tag, tier-A + a bounded `ifd`/`plane` fuzz run.
- `oracle-must-be-shown-red` (blocking) — the Bayer byte-aligned red-proof is
  kept (now reads its reference + checksum from the manifest).
- `provenance-recorded-per-algorithm` — no new algorithm (test/corpus only); no
  ledger row.
- `one-spec-per-pr` (blocking) — the PR references SPEC-027 only.

### Prior related work

- `SPEC-024` (PR #37/#38) — the CFA gate admitting the Bayer plane + `Cfa` model.
- `SPEC-028` (PR #41/#43) — multi-strip assembly (the Nikon is 12 strips).
- `SPEC-029` (PR #44/#46) — relaxed `SampleExceedsWhiteLevel`.
- `SPEC-025` (PR #48/#50, HANDOFF-070) — the bit-exact Bayer plane oracle +
  byte-aligned red-proof, direct-path. This spec folds those onto the manifest.

## Expected Deliverables

- `tests/corpus/manifest.toml`: two new `[[file]]` entries (Canon + Nikon).
- `tests/corpus_manifest.rs`: `MANIFEST_ENTRIES` 7 → 9 + a Bayer spot-check.
- `tests/plane_oracle.rs`: `DECODABLE` 4 → 6, partition `6 + 3 == 9`, bespoke
  Bayer oracle test + hard-coded checksum constants + `require_bayer` deleted,
  red-proof manifest-sourced.
- `tests/plane_unpack.rs`: SPEC-028/029 corpus tests resolved via the manifest.
- `tests/support/tiff.rs`: `cfa_sensor_entries`/`valid_cfa_sensor` +
  `cfa_pattern_past_eof`, emitted into `fuzz/seeds/ifd` by `examples/fuzz-seeds.rs`.
- `tests/ifd_reader.rs`: the two panic-free CFA tests + the costed-tag test.
- All gates green; a bounded `ifd` + `plane` fuzz over the new CFA seed, 0 crashes.
- A PR against `main` referencing PROJ-002 / STAGE-006 / SPEC-027 / HANDOFF-072 /
  DEC-003 / DEC-012 / DEC-029 / DEC-017.

## Out of Scope

- Any demosaic or colour pipeline — STAGE-007.
- The Bayer **colour-metadata** oracle — SPEC-026 (which is why `metadata_oracle.rs`
  defers the two Bayer files here rather than asserting on them).
- Changing the pinned plane-oracle checksum values — they do not move.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`), fill the
`## Handback` section, set `handoff.status`/`handback.status` → `completed`,
open a PR.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes it.*

### Execution notes

- **Branch / PR:** `claude/spec-027-build-b56d8c` → https://github.com/jysf/irradiance/pull/52
- **Completed at:** 2026-10-30
- **All acceptance criteria met?** Yes — every AC in SPEC-027.
  - **AC1** both Bayer DNGs registered with every required field +
    `[file.oracle] raw_checksum`; `corpus_manifest_parses` passes with
    `MANIFEST_ENTRIES == 9`; corpus present, each on-disk `sha256` matches
    (`corpus_files_match_their_pinned_sha256`). Measured: Canon 48,382,640 bytes
    / `95dc76a2…b2e1`, Nikon 37,562,890 bytes / `c2fefd96…8bb7`.
  - **AC2** the Bayer plane oracle is manifest-driven: `CANON_BAYER` + `NIKON_BAYER`
    are in `DECODABLE` (→ 6), `plane_md5_matches_the_pinned_raw_checksum` hashes
    both and matches each manifest `raw_checksum`; no hard-coded Bayer
    path/checksum constant remains (only the two path constants `CANON_BAYER`/
    `NIKON_BAYER`, both resolved through the manifest).
  - **AC3** `compressed_files_are_skipped_by_name` partitions `DECODABLE 6 +
    SKIPPED_COMPRESSED 3 == 9` and, corpus present, classifies all 9 by real
    Compression (`checked == 9`).
  - **AC4** the byte-aligned Bayer red-proof still reddens with a green control,
    sourcing its reference (Nikon) + expected checksum from the manifest; the mono
    oracle + sub-byte red-proof stay green.
  - **AC5** a synthetic CFA fixture is in the `ifd` fuzz seed set
    (`fuzz/seeds/ifd/cfa-sensor-{ii,mm}.tiff`, `cfa-pattern-past-eof.tiff`) and the
    `ifd` test lane; `cargo run --example fuzz-seeds` regenerates it.
  - **AC6** parser panic-free on CFA input: `valid_cfa_sensor` parses to a
    `sensor()` carrying `Some(Cfa …)`; every truncation of it returns without
    panicking (and a header-only prefix is a typed `Truncated`); the hostile
    `cfa_pattern_past_eof` is costed (DEC-012) — no panic, `color_filter == None`,
    `CFAPattern` recorded malformed. Tier-A, no corpus/tools.
  - **AC7** all gates green; bounded `ifd` + `plane` fuzz over the new CFA seed →
    0 crashes; skip-when-absent stays loud.

### Cost self-report

- **Tokens (total):** 65,438,120 (input 456 · output 408,176 · cache-creation
  1,057,890 · cache-read 63,971,598 — a long session with large files resident in
  context, so cache-read dominates)
- **Estimated USD:** 146.41 (Opus list rates: output $75, cache-write $18.75,
  cache-read $1.50, input $15 per Mtok — an order-of-magnitude, cache-read-dominated
  figure)
- **Duration (minutes):** ~40
- **Source of the number:** harness report — summed from this session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read`
  across all usage records; this desktop Code interface exposes no `/cost`
  command), the all-inclusive convention SPEC-024/025/028/029 build used.
  `estimated_usd` at Opus list rates — an order-of-magnitude, cache-read-dominated
  figure.

### Drift and new artifacts

- **New decisions emitted:** none — test/corpus only, no algorithm or policy change.
- **Deviations from spec:**
  1. **Three interlocks the design did not spell out, each a mechanical
     consequence of registering the files — handled minimally and documented
     inline.**
     - `MANIFEST_ENTRIES` 7 → 9 (the spec named this).
     - `tests/ifd_reader.rs` asserts `manifest.files.len() == EXPECTED.len()` and
       `photometric == 34892`; its layer-0 test assumes a single strip. The CFA
       planes are mono-oracle-incompatible (the Nikon is 12 strips; photometric
       32803), so they are **not** added to `EXPECTED`. Added a
       `CFA_CHECKED_ELSEWHERE` partition (mirroring `plane_oracle.rs`'s
       `DECODABLE`/`SKIPPED_COMPRESSED` split) so every manifest entry is still
       accounted for — the Bayer files are checked by `plane_oracle.rs` +
       `plane_unpack.rs`.
     - `tests/metadata_oracle.rs` iterates every manifest file and `exiftool_group`
       **panics** on an unmapped file. The Bayer **metadata** claim is SPEC-026's
       (an explicit Non-Goal here), so a `CFA_DEFERRED_TO_SPEC_026` list skips
       them in all three loops (confirmed: exiftool 7/9, dnglab 6/6).
  2. **The hostile CFA-tag variant is COSTED, not a typed `Err`.** The spec's
     panic-free test wording says "truncated/hostile CFA variants return a typed
     `Err`", but DEC-012 (which the spec's own Notes cite) makes a malformed CFA
     tag cost the field (`color_filter -> None`, recorded) and the file still
     reads. Resolved in favour of the mandated DEC-012 behaviour + the real
     no-panic point: the **truncated** container is asserted as a typed `Truncated`
     error, and the hostile CFA tag (`CFAPattern` past EOF) is asserted no-panic +
     costed. Both run tier-A with no corpus.
- **Follow-up work identified:** none ship-blocking. SPEC-026 (colour metadata)
  is the already-framed next spec; when it extends `metadata_oracle.rs` to cover
  colour, the `CFA_DEFERRED_TO_SPEC_026` skip is the line it removes.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The spec scoped `MANIFEST_ENTRIES` but not the two *other* per-file manifest
   iterators (`ifd_reader.rs`, `metadata_oracle.rs`) that break the moment a 9th
   entry lands — a reader-count interlock, the exact "ship the reader with the
   field" discipline AGENTS.md §11 warns about, only in reverse (an existing
   reader that assumes a fixed set). The repo's own idiom (a named partition) made
   the fix mechanical once found.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-012. The spec's Notes cite it for "a malformed CFA tag costs the field",
   but the panic-free test's "typed Err" wording contradicts it. Naming DEC-012 in
   the Failing-Tests section would have pre-empted the one judgement call here.

3. **If you did this task again, what would you do differently?**
   — Grep every `for … in &manifest.files` / `EXPECTED_FILES` before writing the
   manifest entries, not after a test fails — the three interlocks were all
   foreseeable from "a new manifest entry has N readers", and finding them up front
   turns three debugging loops into one scan.
