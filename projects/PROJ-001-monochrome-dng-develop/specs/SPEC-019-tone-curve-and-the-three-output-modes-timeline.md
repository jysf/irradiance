# SPEC-019 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-019-<cycle>.md`.

## Instructions

- [x] **design** — 2026-10-26, main loop (not separately metered). Scope
  grounded in a real-file probe (`exiftool` on `L1026192.DNG`): the Q2M carries
  no profile tone curve, so the curve is the sRGB OETF. Chose an additive
  encode layer over `develop_into` (`DEC-018` not reopened). Spec + 10 ACs +
  Failing Tests written; `HANDOFF-052` (build) minted; `prompts/SPEC-019-build.md`
  written. New `DEC-025` (tone/output model) pre-specified for build to emit.
- [ ] **build** — `HANDOFF-052`, to `claude-opus-5` (implementer). Not started.
  Make the Failing Tests pass; add `src/tone.rs` + committed generated table +
  `examples/gen-tone-table.rs` + `tests/tone_oracle.rs`; `irr develop --format`
  reader; provenance row; emit `DEC-025`. `just lint-ci` before every push.
- [ ] **verify** — a separate agent reviews cold: analytic oracle red-proof
  actually reds, no runtime `powf`, byte-identity anchors, `develop_into`
  untouched, gates green with CI observed on the shipping SHA.
- [ ] **ship** — merge, CHANGELOG, `cost.totals`, `just cost-audit`, Reflection,
  follow-up disposition; this closes STAGE-003's spec backlog → stage reflection.
