---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. `handback:` is the RETURN path and is not
# optional: `just handback-sync SPEC-NNN` reads it and appends the cost session.
# Contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-095
  cycle: build                     # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-044

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` is the one field the cost gate reads — report the REAL number
# from your interface (Claude Code: `/cost`). If your platform exposes NO token
# count, set tokens_total: null AND say why in `notes`. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-095: Bit-exact corpus oracle — both held Compression-7 DNGs vs dnglab (build)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-044` to the implementer for the **`build`**
cycle: prove both held Compression-7 DNGs decode to a `u16` plane whose MD5 **equals**
`dnglab analyze --raw-checksum`, exactly — the verification half of STAGE-011. This
includes the **one decoder change** the real Pentax K-3 III needs to decode at all
(DEC-039): `ljpeg::parse_headers` must **skip APPn / COM header segments by length**.
The de-interleave mapping (SPEC-046) is already confirmed correct on both real files by
the design §12 probe — you are not re-deriving it, you are locking it with the oracle.

## Context the Receiving Agent Needs

### Primary
- **Spec:** `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md`
  — read all of it. `## Inputs` carries the design §12 probe (both pinned MD5s, the K3III
  and Typ 246 marker maps, the exact `parse_headers` / `parse_dht` line references and the
  `tests/plane_oracle.rs` / `tests/develop_oracle.rs` anchors). `## Outputs` /
  `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
- **Project brief** + the stage `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
- **Toolchain brief:** `guidance/toolchain-brief.md`.

### The §12 probe already ran — transcribe, don't rediscover
- `dnglab 0.7.2 analyze --raw-checksum` (also the pinned `manifest.toml` values):
  - `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` → `c72e7d9579e2adaef9c6247841b8ac1d`
    (5984×4000, 12-bit, big-endian; stream `SOI → SOF3 → DHT → SOS`, **no APPn** — decodes
    and matches **today, unmodified**).
  - `PENTAX-K3III-MONO/K3III.DNG` → `98bae1339c25eb6e04fbac6b5f643189`
    (6304×4224, 14-bit, little-endian; stream `SOI → SOF3 → DHT → APP0(len 60) → SOS` —
    fails today with `LjpegUnexpectedMarker { marker: 0xe0 }`; matches once APPn/COM are
    skipped).
- The APPn/COM-skip was applied as a throwaway probe and **both files matched exactly**.
  So: the mapping needs nothing; the K3III gap is entirely the header-skip.

### Decisions that apply
- `DEC-039` — skip APPn (`0xFFE0..=0xFFEF`) / COM (`0xFFFE`) header segments by length;
  fold into SPEC-044; mapping confirmed on both real files. **Read it first.**
- `DEC-037` — the two-component column-interleave scope (what SPEC-046 shipped).
- `DEC-030` — exact oracle, no tolerance (MD5 equality, not approximate).
- `DEC-003` — corpus files are never committed; located at `$IRRADIANCE_CORPUS_DIR`,
  pinned in `tests/corpus/manifest.toml`. Tests skip **loudly** by name when absent.
- `DEC-035` (clean-room / patent-clean — APPn/COM skip is Annex B marker structure, no new
  algorithm), `DEC-016` (caller-owned buffer), `DEC-018` (u16 modular `wrapping_*`).

### Constraints
- `oracle-must-be-shown-red` — the Compression-7 oracle must be demonstrated able to go red
  on a real decode fault (staged-copy subprocess into `src/ljpeg.rs`; the existing
  `inject_chunk_extraction_fault` targets `src/plane.rs` and will NOT do).
- `no-panics-on-untrusted-input` — a truncated/garbage APPn length is a typed
  `Error::Truncated`, never a panic.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work
- `SPEC-046` (shipped) — the two-component `reconstruct_two_component` / per-component
  predictor state / column de-interleave, and the `encode_lj92` two-component encoder in
  `mod tests` you reuse for the tier-A APPn fixture.
- `SPEC-043` (shipped) — the Compression-7 route in `plane::unpack_into`; do not change it.
- `tests/plane_oracle.rs` — the `DECODABLE` / `SKIPPED_COMPRESSED` lists, `plane_md5`,
  `decode_and_hash`, `plane_md5_matches_the_pinned_raw_checksum`,
  `compressed_files_are_skipped_by_name`, and the staged-copy red-proof machinery.

### The build contract
- Implement SPEC-044's `## Outputs`; make every `## Failing Test` pass.
- The decoder change is **one bounded match arm** in `parse_headers` (mirror `parse_dht`'s
  length-read idiom), placed before the `other => LjpegUnexpectedMarker` catch-all. The
  `Nf=1` path and the Typ 246 decode stay **byte-for-byte unchanged** (characterization
  test; Typ 246's checksum identical with and without the arm).
- Do **not** drop the Compression-7 files into `DECODABLE` — add `DECODABLE_COMPRESSED` and
  extend the partition + oracle loops (see spec `## Notes`), or
  `compressed_files_are_skipped_by_name` breaks.
- `ljpeg` stays `pub(crate)`; no new `Error` variant, no new dependency,
  `#![forbid(unsafe_code)]` + the five-lint deny policy hold.
- Open a PR against `main` from `claude/spec-044-build-<suffix>` referencing `PROJ-003`,
  `STAGE-011`, `SPEC-044`, `HANDOFF-095`, `DEC-039`, `DEC-037`.

### Running the gates (the tier-B lesson)
- The new oracle/red-proof/develop tests are **tier-B, corpus-gated**. The corpus is on
  this machine at `~/PSeven/experiments/crustimg_redo_plus/images` — set
  `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and run the
  **targeted** suites locally so you actually exercise the new tests:
  `cargo test --lib ljpeg::` (tier-A) and `cargo test --test plane_oracle`
  / `cargo test --test develop_oracle` (tier-B). CI cannot see tier-B (no corpus in CI), so
  the real-file oracle only runs locally — that is by design.
- Do **not** launch a blind full `just test` and wait: its unrelated tier-B oracles buffer
  for minutes (the SPEC-038/043/046 lesson). Run the targeted suites, confirm
  `fmt-check` / `lint-ci` / `validate` green, and lean on the PR's CI for the broad
  tier-A/lint sweep.

## Out of Scope
- The provenance-ledger entry, the macOS/Linux determinism assertion, and the broader
  container→decoder hostile-strip hardening — **SPEC-045**.
- The review's later-DHT-replaces (§2.1 sub-point), restart-MCU reset (§2.3), `Pt<P` /
  `Tc/Th` validation (§2.4) — **SPEC-045** (no held file exercises them; K3III's APPn is the
  only review item a held file triggers).
- Bayer/CFA colour multi-component, OpcodeList2 GainMap — out (DEC-037).
- Any change to the reconstruction algorithm or the routing (SPEC-046 / SPEC-043, shipped).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`, or an explicit FLOOR
with a note if your interface has none, as HANDOFF-089..094 did), fill `## Handback`
(reflection included), set both statuses `completed`, run `just advance-cycle SPEC-044 verify`,
open the PR, and mark build `[x]` in the timeline. If blocked: fill `## Handback`, set
`handoff.status: rejected` / `handback.status: blocked`, still report tokens, set
`task.blocked: true`, and add a question to `guidance/questions.yaml`.

Keep commit-message bodies PLAIN PROSE — this repo's push ruleset rejects code tokens
(arrows, `#[...]`, `--flags`, dense backticks) in commit messages; put rich detail in the
PR body and files.

---

## Handback

*Filled in by the receiving agent.*

### Execution notes
- **Branch / PR:** <branch> / PR #<n>
- **Completed at:** <YYYY-MM-DD>
- **All acceptance criteria met?** <yes/no — walk each SPEC-044 acceptance criterion>

### Cost self-report
- **Tokens (total):** <real /cost number, or an explicit FLOOR with its source>
- **Estimated USD:** <or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** <interface>

### Drift and new artifacts
- **New decisions emitted:** <none expected — DEC-039 covers the APPn/COM skip>
- **Deviations from spec:** <none, or what and why>
- **Follow-up work identified:** <e.g. anything for SPEC-045>

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
