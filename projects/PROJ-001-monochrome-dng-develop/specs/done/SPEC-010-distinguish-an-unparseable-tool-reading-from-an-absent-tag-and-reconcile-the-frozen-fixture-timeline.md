# SPEC-010 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-010-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-22, main loop. Probed the defect rather than
  describing it: all four multi-valued tags collapse absent and garbled to a
  byte-identical `ToolReading`, and `BlackLevel [512,999]` reads `Some(512)`.
  Found the information is **discarded, not missing** — `Field.values` is
  already `Option<Vec<u32>>` and the distinction dies in `reading_from_fields`.
  The fix was already built and measured by `SPEC-005/FU-8`, so the spec tells
  build to **reproduce, not re-derive**. Eight failing tests named; the
  red-proof is the same code with one comparison removed. `HANDOFF-024` ready.
- [x] **build** — 2026-09-03, `HANDOFF-024`, run directly in this CLI session
  (not a sub-agent). `ToolValue<T>` tri-state added to
  `tests/support/tools.rs`; `reading_from_fields` classifies each optional
  field Absent/Unreadable(raw)/Value; `diff()` → `diff_with_malformed()` +
  one generic `compare_optional` arm replaces seven per-tag branches. Eight
  named tests added to `tests/metadata_oracle.rs`, all found via `-- --list`
  summed across all six targets. AC6's red-proof calls the real
  `diff_with_malformed(sensor, reading, &[])` (SPEC-005/FU-8's measured
  "not consulted" mutant) rather than mutating source; additionally verified
  live by mutating `diff()` itself, watching `metadata_matches_exiftool_on_
  every_corpus_file` go red on K3III.DNG, then restoring byte-identical.
  `DEC-014` written as `DEC-013`'s true successor (`AC7`); `DEC-013` left
  `status: rejected`, `superseded_by: null` (decisions-audit requires the two
  agree, and "superseded" would understate why it was rejected) with a prose
  pointer instead. Ten gates + `lint-ci` + `oracle-meta` green locally;
  `FU-9` (`is_active()` reads only `superseded_by`) confirmed still open —
  out of this spec's scope, carried as a finding. `feat/spec-010-tri-state-
  tool-reading`.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
