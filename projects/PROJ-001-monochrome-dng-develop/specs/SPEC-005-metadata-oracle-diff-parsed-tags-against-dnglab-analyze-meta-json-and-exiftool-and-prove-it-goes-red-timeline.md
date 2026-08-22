# SPEC-005 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-005-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-21, main loop. Probed `exiftool 13.55` and
  `dnglab 0.7.2` against all seven corpus files before writing the spec
  (§15 design rule 4). The probe changed the design: the two tools answer
  **different questions** — exiftool reads what the file says, dnglab reports
  what a decoder concluded through rawler's camera database — so exiftool is
  the tag-level oracle on all seven files and dnglab is an
  interpretation-level cross-check on the six DNGs, with three known
  divergences **asserted** rather than skipped. `exiftool -T -n -s3` removed
  the need for any JSON dependency. Nine failing tests named; two-tier
  red-proof (the tier-A comparator half runs in CI, which is the only half
  CI can see). `HANDOFF-021` ready for build.
- [ ] **build** — `HANDOFF-021`. Dispatch to a separate CLI session.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
