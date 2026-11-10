# SPEC-054 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-054-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — full design against the current tree;
  DEC-049 minted. Corpus opcode lists probed (`exiftool -b` → per-opcode `id`/`flags`):
  no corpus file carries `OpcodeList2`; the Bayer/colour files carry no opcode lists;
  the Q2-Mono's mandatory list-1/list-3 opcodes are both applied — so validation is
  pure hardening and the corpus stays bit-exact. Confirmed `parse_opcode_list` drops
  `flags` for recognized opcodes (so a flags-aware raw scan is required, not the typed
  parse) and that `tests/` builds `Sensor` by struct literal in 7 sites (the
  `#[non_exhaustive]` blast radius). `Sensor` sealed here; the other public structs'
  sweep scheduled to SPEC-042. HANDOFF-112 (build) prepared. Spec moved `frame → build`.
- [x] **build** — delegated via HANDOFF-112. Branch `claude/spec-054-build-<suffix>`
  from freshly-merged `main`. Keep `OpcodeList2` bytes; add the flags-aware
  required-processing scan + `Error::UnsupportedRequiredOpcode`, called at every public
  develop entry (mono + colour + f32 + threaded); seal `Sensor` + migrate the 7 `tests/`
  fixtures onto a dev-feature constructor; red-proofs shown red first; fuzz; corpus
  bit-exact (local run); gates; PR; fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ⚠ PUNCH LIST, code approved as-is at cd05206; removing the check from each of the 5 real develop entries broke 6–7 tests each (per-entry coverage confirmed; the mono f32/threaded entries in the brief do not exist), corpus bit-exact across 8 files, DEVELOP_VERSION unchanged. One doc-only HANDOFF-112 restoration, shipped via the rebased #175. — cold review: the error fires from *each* public entry (§12, not one),
  the scan validates on raw bytes (not the flags-dropping typed parse), the corpus
  develops bit-exact, `DEVELOP_VERSION` unchanged, the dev-feature fixtures do not leak
  into the default surface.
- [x] **ship** (2026-11-10) — group C merged via PR #175 (rebased verify superset of #170/#173). Cost 26,188,198 tok / $17.96 (build 18.67M + verify 7.52M, handback-sync). Brag logged; STAGE-010 backlog updated. Follow-ups raised for group E/F: malformed opcode-list tag read as absent; public stage fns skip the check; plain `cargo test` needs `--all-features`.
