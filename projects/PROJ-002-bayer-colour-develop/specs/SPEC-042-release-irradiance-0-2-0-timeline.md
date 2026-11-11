# SPEC-042 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-042-<cycle>.md`.

## Instructions

- [x] **frame** — ID reserved; release-spec template; held for the P1 wave.
- [x] **design** (orchestrator, 2026-11-10) — all blockers shipped (A–E + 040 + 041 on
  main). DEC-052 minted: seal `Cfa`/`WarpRect`/`Opcode` `#[non_exhaustive]` (completing the
  sweep `Sensor` began), cut **semver 0.2.0**, defer the two group-C opcode follow-ups to
  SPEC-056/0.2.1 (fold only the dev-dependency test fix into F), and ship colour without an
  accuracy claim. Filled the release scope, the Phase-1 build-work list, and the two-phase
  cut (Phase 2 = tag + publish, human-gated). HANDOFF-123 (build) prepared. Spec moved
  `design → build`.
- [x] **build** (claude-opus-5-5 cold, PR #186 merged via #187) — delegated via HANDOFF-123 (Phase 1, reversible prep). Branch
  `claude/spec-042-build-<suffix>` from freshly-merged `main`. Seal + fixture migration +
  self dev-dependency + version 0.2.0 + CHANGELOG promotion + release notes + corpus
  evidence run + filled pre-flight; PR says "prep-complete, cut-deferred"; fill the
  handback. No tag, no publish.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-11) — ⚠ PUNCH LIST at d273296: two doc-only fixes (DEVELOP_VERSION doc said "0.1.x"; MB→MiB in notes) → superset PR #187. Clean otherwise: Cfa/WarpRect/Opcode sealed with no catch-all arm, 519 tests pass both with --all-features and bare, publish dry-run clean (v0.2.0, 19 files), no tag/publish, SPEC-056 items not implemented. Confirmed: the three types sealed + in-crate matches compile, a bare
  `cargo test` builds, version/semver + CHANGELOG + notes correct (no colour claim),
  `cargo publish --dry-run` clean, corpus evidence attached, no tag/publish happened.
- [x] **ship (Phase 1)** (2026-11-11) — Phase-1 prep merged via PR #187 (superset of #186). Cost 21,944,993 tok / $14.52 (build 14.9M/$9.80 + verify 7,044,993/$4.72). brag logged; STAGE-010 → 8 shipped / 0 active and STAGE-SHIPPED. **Phase 2 (human-gated, NOT done): tag v0.2.0 + cargo publish.** Two non-blocking follow-ups for SPEC-056: cargo-doc private-item doc links; tests/develop.rs under CARGO_TARGET_DIR. Prior template text:
  then **STAGE-010 stage-ship** (Prompt 1d). **Phase 2 (human-gated):** cut `v0.2.0` + `cargo publish`.
