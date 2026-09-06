# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06, main loop. Probe RAN and **measured all five
  carried findings rather than inheriting them, which changed two.**
  `SPEC-005/FU-3` reproduced exactly: corpus 7/7 present with `exiftool`/`dnglab`
  absent, the pre-flight prints *"no tier-B test will skip"* while all 30 oracle
  tests skip in **0.04 s**. `SPEC-012/FU-1` measured **by mutation**: deleting 8
  and 12 from `SUPPORTED_BITS` leaves **152/152 green**, so two of four declared
  depths are exercised by nothing. `SPEC-012/FU-2` still open — `white_level`
  exists only in `develop_fixture`, never in `plane_fixture`. `SPEC-005/FU-2`
  **sharpened**: `req` does not merely truncate, its doc comment *justifies* the
  truncation — a documented assumption with no falsifier, which is the same class
  as the other four. Fifth instance added from `SPEC-015`'s own cycles: `just
  validate` greps and never parses, and reported *"valid required front-matter"*
  on two files no parser could read, one of them shipped and archived undetected.
  Seven ACs, five failing tests, **all tier A on purpose** — every finding is
  about a surface that lies when something is absent, and CI is where things are
  absent. The design question the frame left open is **settled** (check the
  tools, with a `DEC-*` recording the rejected option). Complexity raised
  **M → L**. ⚠ The gate-script audit is **excluded and sized** — 8 of 13
  `pipefail` scripts, 28 unguarded greps in `test.sh` alone — so `STAGE-005`'s
  separate bullet can be written. `HANDOFF-038` ready.
- [ ] **build** — `HANDOFF-038`.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
