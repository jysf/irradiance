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
- [x] **build** (cold claude-opus-5-5, 2026-11-11) — HANDOFF-125, PR #193 (code `1973abf`, merged as
  `dd70dea`); CI 44/44 green on the code commit. 20,494,603 tok / $14.01 (re-summed at ship).
- [x] **verify** (cold claude-opus-5-5, 2026-11-11) — HANDOFF-126: ✅ APPROVED at `1973abf`; handback PR #194.
  10,841,767 tok / $7.64 (re-summed at ship).
- [x] **ship** (orchestrator, 2026-11-11) — total 31,336,370 tok / $21.65 (build + verify); 0.2.1 version
  bump + dated CHANGELOG section in the ship PR; maintainer tags + publishes.
