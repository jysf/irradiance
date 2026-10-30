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
  id: HANDOFF-073
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: verifier             # implementer | verifier
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
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 9885669            # REAL combined count — what cost-audit reads
  estimated_usd: 22.79             # tokens_total × your rate, or your harness's number
  duration_minutes: 30
  branch: claude/spec-027-verify-6d5a34
  pr: https://github.com/jysf/irradiance/pull/53   # verify-bookkeeping PR
  completed_at: 2026-10-30          # YYYY-MM-DD
  notes: "✅ APPROVED at commit e95ec04 (PR #52, already MERGED to main before this verify ran — a retrospective cold independent review; no build branch remained to push to, hence a small verify-bookkeeping PR). PR #52 is TEST/CORPUS/FUZZ ONLY (13 files, +696/-110, NO src/ change): git diff c178d0b..e95ec04 touches tests/corpus/manifest.toml, tests/{corpus_manifest,ifd_reader,metadata_oracle,plane_oracle,plane_unpack,support/tiff}.rs, fuzz/seeds/ifd/cfa-*.tiff (3) + the spec/timeline/HANDOFF-072. Independent ground truth recomputed this session: on-disk sha256 Canon 95dc76a2f255e623acaa5149b320389f7da7c995da890c3e0d511c0ff681b2e1 (48,382,640 bytes) + Nikon c2fefd96e0125f5d5b891dff66b9ac673d6f0059db21c3a9dd469fe460f88bb7 (37,562,890 bytes) == manifest; dnglab 0.7.2 analyze --raw-checksum Canon a18235d6abced4df9948b6fbe91aba7c + Nikon b8e8025d370cc2e97f675e63a0e6bbdb == manifest [file.oracle] raw_checksum (not a self-comparison). corpus_files_match_their_pinned_sha256 verified 9/9 live. Oracle fold confirmed manifest-driven: DECODABLE 4->6, plane_md5_matches_the_pinned_raw_checksum hashes both Bayer and reads each expected checksum from entry.oracle.raw_checksum, partition 6+3==9; the bespoke bayer_plane_md5 test + CANON/NIKON_RAW_CHECKSUM consts + require_bayer are GONE (grep: no Bayer checksum literal in plane_oracle.rs, only the manifest + the corpus_manifest.rs manifest-integrity spot-check). Byte-aligned red-proof reddens from the MANIFEST reference: honest b8e8025d (== manifest Nikon) vs mutant fdcb897fbae2bd9d180e05f9217f5011, control green (live --nocapture). plane_unpack SPEC-028/029 migrated to manifest.get()+entry.require(); forward-ref 'SPEC-027 registers it' skip messages gone (grep clean). CFA fuzz: 3 synthetic seeds (cfa-sensor-ii/mm, cfa-pattern-past-eof) in fuzz/seeds/ifd, cargo run --example fuzz-seeds idempotent (empty git diff). Panic-free: a_well_formed_cfa_container_parses_to_a_cfa_sensor (color_filter Some(Cfa{repeat_dim [2,2], pattern [0,1,1,2]})), a_truncated_cfa_container_returns_without_panicking (typed Truncated on header-only prefix), a_hostile_cfa_tag_costs_the_field_not_the_file (DEC-012/DEC-015 costed: color_filter None + CFAPattern in malformed_tags, no panic). THREE deviations ADJUDICATED (all honest): (1) CFA_CHECKED_ELSEWHERE in ifd_reader is an honest exclusion, not a coverage drop — partition EXPECTED(7)+CFA_CHECKED_ELSEWHERE(2)==9 is enforced and the CFA files are genuinely checked by plane_oracle (DECODABLE) + plane_unpack; (2) metadata_oracle CFA_DEFERRED_TO_SPEC_026 is honest — SPEC-026's domain (stated Non-Goal), the default on an unmapped file is a PANIC (loud), and mono coverage is unchanged (exiftool 7/9, dnglab 6/6 live); (3) the hostile cfa_pattern_past_eof is COSTED not a typed Err, which is the CORRECT DEC-012 (strict-on-structure/tolerant-on-shape) + DEC-015 (malformed_tags records a lost value) behaviour — no src/ changed, so this is SPEC-024's established behaviour, and the spec's 'typed Err' wording was the slip (the truncated container IS a typed Truncated). Reader interlock exhaustive: all manifest readers updated (corpus_manifest MANIFEST_ENTRIES=9, plane_oracle dynamic files.len() partition, ifd_reader partition, metadata_oracle 3 loops); no stale count-7 manifest reader survives. Gates: just test full suite (0 failed across all 13 binaries, 1 pre-existing documented ignore in develop.rs; corpus_manifest 9/9, plane_oracle 14/14, plane_unpack 18/18, ifd_reader 15/15, metadata_oracle 30/30, lib 95/0), just lint-ci clippy -D warnings pinned +stable 0.1.98 exit 0, cargo fmt --check exit 0, just validate exit 0 (33 valid / 150 parseable), just ready clean, just decisions-audit 0 structural errors (11 pre-existing advisory). Bounded fuzz over the new CFA seed: ifd 9,199,147 runs/46s 0 crashes, plane 10,718,767 runs/46s 0 crashes (plane fed both seed sets incl. the CFA ifd seeds — both targets take the same TIFF format), 0 artifacts, tree clean. No new DEC; verify emitted none. tokens_total summed from THIS verify session's own Claude Code transcript JSONL (scratchpad-UUID anchored 15b4889e, deduped by message.id across 50 usage records, all claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (9.62M of 9.89M, 97.3%); estimated_usd at per-category Opus list rates. Floor-by-id snapshot taken at hand-back, not a budget delta."
  synced_at: 2026-10-30
---

# HANDOFF-073: VERIFY — Corpus + fuzz: register the uncompressed Bayer DNG in the corpus manifest and extend the fuzz corpus; panic-free on CFA input

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-027` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — review cold
the build that registered the two uncompressed Bayer DNGs in
`tests/corpus/manifest.toml`, folded SPEC-025's bit-exact Bayer plane oracle onto
the manifest, migrated the SPEC-028/029 corpus tests to the manifest, and added a
synthetic CFA fuzz seed + a tier-A panic-free-on-CFA test. STAGE-006 housekeeping;
test/corpus/fuzz-only, the pinned checksums do not move.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-027-...-panic-free-on-cfa-input.md`
  — Acceptance Criteria (7) + Failing Tests are the contract.
- **Build HANDOFF:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-072-build-...-panic-free-on-cfa-input.md`.
- **The manifest:** `tests/corpus/manifest.toml` — the two new Bayer `[[file]]` entries.
- **The tests:** `tests/corpus_manifest.rs`, `tests/plane_oracle.rs`,
  `tests/plane_unpack.rs`, `tests/ifd_reader.rs`, `tests/metadata_oracle.rs`,
  `tests/support/tiff.rs`, and the `fuzz/seeds/ifd/cfa-*.tiff` seeds.

### Decisions that apply

- `DEC-003` — the tier-B corpus storage model: the manifest is committed, the
  files never are; licence/provenance has no second chance.
- `DEC-012` — strict on structure, tolerant on shape: a malformed interpretation
  tag (CFAPattern past EOF) errors the interpret call, never the container walk —
  the file reads, the field is costed. This is why the hostile CFA seed is costed,
  not a typed `Err`.
- `DEC-015` — `malformed_tags` records a lost value (CFAPattern recorded,
  `color_filter -> None`), not an errored read.
- `DEC-029` — samples above `WhiteLevel` are legal; the byte-swap guard lives on
  the plane oracle's red-proof (SPEC-025), which this spec keeps, now
  manifest-sourced.
- `DEC-017` — the red-proof mechanism (mutate a crate copy, prove the OUTPUT
  changed); reused unchanged.

### Constraints that apply

- `no-panics-on-untrusted-input` (blocking) — the CFA parse path proven panic-free
  on a well-formed mosaic, every truncation, and a hostile CFA tag; tier-A + a
  bounded `ifd`/`plane` fuzz run.
- `oracle-must-be-shown-red` (blocking) — the Bayer byte-aligned red-proof kept,
  now manifest-sourced.
- `provenance-recorded-per-algorithm` — no new algorithm (test/corpus only).
- `one-spec-per-pr` (blocking) — PR #52 references SPEC-027 only.

### Prior related work

- `HANDOFF-072` — the build cycle under review (PR #52, merged at `e95ec04`).
- `SPEC-024` (PR #37/#38) — the CFA gate + `Cfa` model; it established the costed
  CFA-tag behaviour this spec's panic-free test documents (no src/ change here).
- `SPEC-025` (PR #48/#50, HANDOFF-070/071) — the bit-exact Bayer plane oracle +
  byte-aligned red-proof, direct-path; this spec folds it onto the manifest.
- `SPEC-028` (PR #41/#43) — multi-strip assembly (the Nikon is 12 strips).
- `SPEC-029` (PR #44/#46) — relaxed `SampleExceedsWhiteLevel` + DEC-029.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold.
Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Shipping (archive / CHANGELOG / brag / reflection / cost totals / verdict
  stamp) — the maintainer's main-session step.
- SPEC-026 (colour-metadata oracle) — the Bayer metadata claim this spec defers.
- Changing the pinned plane-oracle checksum values — they do not move.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`), fill
`## Handback`, set `handoff.status`/`handback.status` → `completed`, return the
verdict, open a verify-bookkeeping PR. Leave `task.verify_verdict` as the
machinery sets it — the maintainer's ship (verify → ship) stamps `approved`.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-027-verify-6d5a34` → verify-bookkeeping PR #53
  (PR #52 already MERGED at `e95ec04`, so no build branch remained to push to —
  this is a retrospective independent review).
- **Completed at:** 2026-10-30
- **All acceptance criteria met?** Yes — every one of SPEC-027's 7 ACs reproduced cold.
- **Verdict:** **✅ APPROVED** at commit `e95ec04`.

#### PR #52 scope confirmed (test/corpus/fuzz only)

`git diff --name-only c178d0b..e95ec04` is 13 files, +696/-110: `tests/corpus/manifest.toml`,
`tests/{corpus_manifest,ifd_reader,metadata_oracle,plane_oracle,plane_unpack,support/tiff}.rs`,
`fuzz/seeds/ifd/cfa-{sensor-ii,sensor-mm,pattern-past-eof}.tiff`, plus the spec,
its timeline, and HANDOFF-072. **No `src/` change** (`git diff --name-only … | grep ^src/`
empty), so no library/behaviour change, exactly as the spec requires.

#### Independent ground truth (the anti-tautology check)

Recomputed this session on the real corpus files:

| file | my sha256 / bytes | my `dnglab 0.7.2 --raw-checksum` | manifest | match |
|---|---|---|---|---|
| `CANON-EOS-5D-MARK-III/5G4A9394.DNG` | `95dc76a2…b2e1` / 48,382,640 | `a18235d6abced4df9948b6fbe91aba7c` | same | ✅ |
| `NIKON-D300S/_DSC4615.DNG` | `c2fefd96…8bb7` / 37,562,890 | `b8e8025d370cc2e97f675e63a0e6bbdb` | same | ✅ |

The oracle asserts `irradiance_plane_md5 == manifest.raw_checksum`; I independently
verified `dnglab_checksum == manifest.raw_checksum` and `on-disk sha256 == manifest`.
dnglab is a separate tool (run, never linked), so `irradiance == dnglab`
transitively — not a self-comparison. `corpus_files_match_their_pinned_sha256`
printed `verified 9/9 corpus file(s)` live.

#### Gates re-run (all green, this session, corpus present)

- **`just test`** — full suite, **0 failed across all 13 binaries** (1 pre-existing
  documented `#[ignore]` in `develop.rs`): `src/lib.rs` 95/0, `corpus_manifest`
  **9/9**, `plane_oracle` **14/14**, `plane_unpack` **18/18**, `ifd_reader`
  **15/15**, `metadata_oracle` **30/30**, the rest 0-failed. Corpus genuinely
  present (`--nocapture`): `verified 9/9`, exiftool `7/9`, dnglab `6/6` — the
  expected partition, no Bayer SKIP lines.
- **clippy `-D warnings`** (`just lint-ci`, pinned `~/.cargo/bin/cargo +stable`,
  clippy 0.1.98) → exit 0.
- **`cargo fmt --check`** → exit 0. **`just validate`** → exit 0 (33 valid required
  front-matter; 150 parseable). **`just ready`** → clean.
- **`just decisions-audit`** → **0 structural errors**, 11 pre-existing advisory
  scope warnings (unchanged — SPEC-027 added no DEC and no src change).
- **Fuzz** (`+nightly`, PATH dance) over the new CFA seed — **`ifd` 9,199,147 runs
  / 46s, 0 crashes**; **`plane` 10,718,767 runs / 46s, 0 crashes** (plane fed both
  `fuzz/seeds/plane` and `fuzz/seeds/ifd`, the latter carrying the CFA seeds — both
  targets take the same TIFF byte format). No `fuzz/artifacts/*` entry; tree clean
  (`fuzz/corpus/*` is gitignored).

#### Spec acceptance criteria — all 7 met

- **AC1 — both Bayer DNGs registered, `MANIFEST_ENTRIES == 9`, sha256 matches.**
  Both `[[file]]` entries carry every required field + `[file.oracle] raw_checksum`;
  `pgm_bytes`/`strip_bytes` omitted by design and asserted `None`. `corpus_manifest_parses`
  9/9; `corpus_files_match_their_pinned_sha256` `9/9` live. ✅
- **AC2 — Bayer plane oracle is manifest-driven, no hard-coded Bayer checksum
  constant remains.** `CANON_BAYER`/`NIKON_BAYER` in `DECODABLE` (→6);
  `plane_md5_matches_the_pinned_raw_checksum` reads each expected from
  `entry.oracle.raw_checksum`. The bespoke test, the `*_RAW_CHECKSUM` consts, and
  `require_bayer` are deleted; grep finds no Bayer checksum literal in
  `plane_oracle.rs` (only the manifest pin + the `corpus_manifest.rs`
  manifest-integrity spot-check). ✅
- **AC3 — `compressed_files_are_skipped_by_name` accounts for every entry.**
  `DECODABLE(6) + SKIPPED_COMPRESSED(3) == manifest.files.len()(9)`; corpus present,
  it parses all 9 by real Compression. ✅
- **AC4 — byte-aligned red-proof reddens (manifest-sourced); mono oracle + sub-byte
  red-proof stay green.** Live `--nocapture`: honest `b8e8025d…` (== manifest Nikon,
  via `manifest.get(NIKON_BAYER).require(&root)` + `entry.oracle.raw_checksum`) vs
  mutant `fdcb897fbae2bd9d180e05f9217f5011`; control green. Mono sub-byte red-proof
  `cb653b5b… → 59b032fe…` still reddens. ✅
- **AC5 — a synthetic CFA fixture is in the `ifd` seed set and the `ifd` test lane;
  `cargo run --example fuzz-seeds` regenerates it.** `cfa-sensor-{ii,mm}.tiff` +
  `cfa-pattern-past-eof.tiff` present; regeneration is idempotent (empty git diff);
  asserted in `ifd_rejects_hostile_input` + the three new CFA tests. ✅
- **AC6 — panic-free on CFA input.** `a_well_formed_cfa_container_parses_to_a_cfa_sensor`
  (`color_filter == Some(Cfa { repeat_dim:[2,2], pattern:[0,1,1,2] })`,
  `malformed_tags` empty, both byte orders); `a_truncated_cfa_container_returns_without_panicking`
  (every prefix returns; header-only prefix is a typed `Truncated`);
  `a_hostile_cfa_tag_costs_the_field_not_the_file` (DEC-012/DEC-015 costed:
  `color_filter == None`, `CFAPattern` in `malformed_tags`, no panic). Tier-A. ✅
- **AC7 — all gates green; bounded `ifd` + `plane` fuzz over the new CFA seed →
  0 crashes; skip-when-absent stays loud.** See Gates. ✅

#### The three flagged deviations — adjudicated (all honest)

1. **`CFA_CHECKED_ELSEWHERE` in `ifd_reader`'s `manifest_pairs` — honest exclusion,
   not a silent coverage drop.** `ifd_reader`'s `EXPECTED` oracle is the monochrome
   `LinearRaw` container oracle (`photometric == 34892`, single-strip layer-0),
   which genuinely cannot describe a CFA mosaic (photometric 32803; the Nikon is
   12-strip). The two Bayer files are named in `CFA_CHECKED_ELSEWHERE`, and the
   partition `EXPECTED.len()(7) + CFA_CHECKED_ELSEWHERE.len()(2) == manifest.files.len()(9)`
   is asserted, with per-entry checks that each is in the manifest and not also in
   `EXPECTED`. Their container reading + bit-exact plane coverage is in
   `plane_oracle.rs` (both in `DECODABLE`, both parsed in
   `compressed_files_are_skipped_by_name`) and `plane_unpack.rs`. A future
   unregistered entry fails the partition loudly. Correct.
2. **`metadata_oracle` skips the Bayer files (`CFA_DEFERRED_TO_SPEC_026`) — honest,
   not a hole.** The Bayer *metadata* claim is SPEC-026's (a stated Non-Goal here).
   The skip is an explicit named list; the default on an unmapped file is a
   `panic` (the loud failure the skip is the sanctioned exception to); and mono
   coverage is unchanged (`exiftool 7/9`, `dnglab 6/6` live — exactly the pre-SPEC-027
   mono set). This is the monochrome scalar oracle (SPEC-005) with no colour-aware
   diff; deferring is correct until SPEC-026 extends it.
3. **The hostile `cfa_pattern_past_eof` is COSTED, not a typed `Err` — correct
   DEC-012 behaviour.** DEC-012's table: a wrong field type / out-of-bounds payload
   errors the *interpret* call, not the *container walk*; DEC-015: `malformed_tags`
   records a lost value. So `Container::parse` succeeds, `sensor()` succeeds,
   `color_filter == None`, `CFAPattern` is in `malformed_tags`, no panic — the
   behaviour SPEC-024 established and this test (on **unchanged** src) documents.
   The spec's Failing-Tests "typed Err" wording was the slip the build flagged; the
   genuinely structural case (truncation) **is** asserted as a typed `Truncated`.
   The test asserts no-panic + costed. Correct.

#### Code review (cold)

- **Reader interlock exhaustive.** `grep` for every `manifest.files` iterator and
  count assertion confirms all readers were updated: `corpus_manifest`
  (`MANIFEST_ENTRIES 7→9` + a Bayer spot-check pinning licence/bytes/raw_checksum/
  omitted `pgm_bytes`/`strip_bytes`), `plane_oracle` (dynamic `files.len()`
  partition), `ifd_reader` (`EXPECTED_FILES` stays 7 = the mono-table length; the
  CFA files accounted in `CFA_CHECKED_ELSEWHERE`), `metadata_oracle` (3 loops). No
  stale count-7 reader of the manifest survives, and no `SPEC-027 registers it`
  forward-reference remains anywhere in `tests/`.
- **CFA fixtures** (`tests/support/tiff.rs`): `cfa_sensor_entries` mirrors
  `sensor_entries` with photometric 32803 + the four CFA tags inline
  (`CFARepeatPatternDim [2,2]`, `CFAPattern [0,1,1,2]` RGGB, `CFAPlaneColor
  [0,1,2,0]`, `CFALayout 1`); `cfa_pattern_past_eof` relocates `CFAPattern` to
  offset `0xFFFF_FF00` with count 64 (out-of-bounds payload) — the exact DEC-012
  interpretation-class fault. Emitted into `fuzz/seeds/ifd` via `tiff::all()`, so
  `examples/fuzz-seeds.rs` needed no edit.
- **Oracle fold** (`tests/plane_oracle.rs`): the red-proof apparatus
  (`inject_byte_order_swap_fault`, `stage_probe_crate`, `build_and_run_probe`) is
  SPEC-025's, unchanged; only its reference lookup moved to
  `manifest.get(NIKON_BAYER).require(&root)` + `entry.oracle.raw_checksum`. The
  honest digest is recomputed in-process (not read from the manifest) so the
  red-proof does not depend on the md5 oracle having run first.

### Cost self-report

- **Tokens (total):** 9,885,669 (input 100 · output 60,436 · cache-creation
  204,131 · cache-read 9,621,002 — a long session with the large AGENTS.md / spec /
  test files resident in context, so cache-read dominates at 97.3%)
- **Estimated USD:** 22.79 (Opus list rates: in $15, out $75, cache-write $18.75,
  cache-read $1.50 per Mtok — an order-of-magnitude, cache-read-dominated figure)
- **Duration (minutes):** ~30
- **Source of the number:** harness report — summed from THIS verify session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read`
  across all usage records, deduped by `message.id` to 50 records, scratchpad-UUID
  anchored `15b4889e-…`; all records `claude-opus-4-8`, confirmed). This desktop
  Code interface exposes no `/cost` command — the all-inclusive convention
  SPEC-025's verify used. Floor-by-id snapshot taken at hand-back, not a budget delta.

### Drift and new artifacts

- **New decisions emitted:** none by verify — test/corpus/fuzz only, no src/
  change, no algorithm or policy change.
- **Deviations from spec — adjudicated:**
  1. **`to_agent` tier_map hint said `claude-opus-5`; the verify actually ran on
     `claude-opus-4-8`.** Corrected in the front-matter (same correction
     HANDOFF-069/071 made). Bookkeeping only.
  2. The three build-flagged deviations (CFA_CHECKED_ELSEWHERE, CFA_DEFERRED_TO_SPEC_026,
     costed-not-errored CFA tag) are all **correct** — see "The three flagged
     deviations" above. None is a defect; no punch-list.
- **Follow-up work identified:**
  - **Non-blocking, cosmetic (NOT ship-blocking):** `metadata_oracle`'s
    `CFA_DEFERRED_TO_SPEC_026` does not assert its entries are in the manifest,
    unlike `ifd_reader`'s `CFA_CHECKED_ELSEWHERE` (which does). A typo there would
    still fail loudly (the file hits `exiftool_group`'s panic), so this is a
    style-consistency nit, not a hole. SPEC-026 removes this list entirely when it
    extends the metadata oracle to colour, so no action is warranted now.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The build flagged all three deviations up front and the
   spec's own Notes cited DEC-012 for the costed-CFA-tag behaviour, so the one
   wording tension ("typed Err" vs costed) was pre-resolved. The decisive extra
   step was proving the deviations against the DECs themselves (DEC-012's table,
   DEC-015) rather than taking the handback's word, and confirming via `--nocapture`
   that the Bayer corpus genuinely RAN (not silently skipped).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-015. The handoff cited DEC-012 for the costed CFA tag; DEC-015
   ("malformed_tags records a lost value, not an errored read") is the other half
   of why the hostile seed is costed-and-recorded rather than errored, and makes
   deviation 3 unambiguous. The two namespaces (`/decisions/DEC-012` =
   interpretation rule, `/docs/decisions/DEC-012` = spike lane) were worth
   disambiguating explicitly.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The two decisive checks were (a) recomputing both sha256,
   both `dnglab --raw-checksum`, and watching the live `9/9`, `7/9`, `6/6` counts so
   "corpus present and bit-exact" is independent of the pinned values, and (b)
   grepping every `manifest.files` reader to prove the three-interlock claim was
   exhaustive (no fourth stale reader) rather than trusting the handback's count.
