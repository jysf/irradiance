# SPEC-056 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-056-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator claude-opus-4-8, 2026-11-11) — STAGE-014 minted as the post-0.2.0
  maintenance stage. DEC-053 decided with the maintainer: refuse an unreadable OpcodeList at develop
  (new Error::UnreadableOpcodeList, kind Corrupt; read stays tolerant), guard the Sensor-taking
  primitives on the colour path, public check + DevelopPath; escape hatch deferred. Probes: `irr ifd`
  over all 8 tier-B files (no opcode-list tag malformed; only K3III 50713), `cargo doc -D warnings`
  (19 private intra-doc links in 8 files), nested-cargo audit (all 7 staged-build tests hardcode
  `target/release`, not just tests/develop.rs). HANDOFF-125 (build) prepared. Spec moved design to build.
- [ ] **build** — delegated via HANDOFF-125 to a cold claude-opus-5-5 session. Branch
  `claude/spec-056-build-<suffix>` from freshly-merged `main`.
- [ ] **verify** — cold, separate session (HANDOFF-126 when build hands back).
- [ ] **ship** — orchestrator: handback-sync + cost recovery, brag, 0.2.1 bump + CHANGELOG section;
  maintainer tags + publishes.
