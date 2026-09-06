# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06, main loop (session `local_7c8b...`). Probe
  RAN and **measured all five carried findings rather than inheriting
  them, which changed two.** `SPEC-005/FU-3` reproduced exactly: corpus
  7/7 present with `exiftool`/`dnglab` absent, the pre-flight prints
  *"no tier-B test will skip"* while all 30 oracle tests skip in
  **0.04 s**. `SPEC-012/FU-1` measured **by mutation**: deleting 8 and
  12 from `SUPPORTED_BITS` leaves **152/152 green**, so two of four
  declared depths are exercised by nothing. `SPEC-012/FU-2` still open —
  `white_level` exists only in `develop_fixture`, never in
  `plane_fixture`. `SPEC-005/FU-2` **sharpened**: `req` does not merely
  truncate, its doc comment *justifies* the truncation — a documented
  assumption with no falsifier. Fifth instance added from `SPEC-015`'s
  own cycles: `just validate` greps and never parses, and reported
  *"valid required front-matter"* on two files no parser could read.
  Seven ACs, five failing tests, all tier A on purpose. **`HANDOFF-038`
  written for this L scope; NEVER DISPATCHED — see the `RE-SCOPE
  SUPERSEDED` note at the top of that file.**
- [x] **design (re-scope)** — 2026-09-06, main loop, main-loop-not-
  separately-metered. Design at `3238dcb` was **L, five ACs**; that
  shape is the one that produced three patches on one gate in the same
  week (PATCH-002 → PATCH-003 → PATCH-004). This pass reduces to **S,
  one AC** — AC5 (`just validate` parses YAML), the half that closes a
  CLASS rather than a specific surface. The four dropped ACs are on
  STAGE-005's backlog as separate `(not yet written) [XS]` items, each
  pointing back to `3238dcb`'s `## Implementation Context` for its
  measurement rather than re-deriving them. `HANDOFF-038` is stale on
  its L scope; a NEW handoff for the reduced scope is minted at build
  time (not this cycle). Follow-up work: verify at merge time that the
  four `(not yet written) [XS]` items either land as specs or get an
  explicit `closed:` disposition with a test that will fail — do not
  let them sit indefinitely.
- [ ] **build** — new handoff pending. Mint at build time (do NOT reuse
  `HANDOFF-038` — its `## Delegation Summary` describes five ACs and a
  worker following it would build the L scope). The reduced-scope
  handoff reads: rewrite `scripts/validate.sh` to parse front matter
  through a real YAML parser (`ruby -ryaml` per the DEC), add a
  red-proof gate step (DEC-009 mechanism), one DEC recording the
  parser choice with the rejected alternatives, no `src/` changes.
- [ ] **verify** — a separate agent runs the red-proof against a
  hand-built fixture with an unterminated multi-line scalar (the
  SPEC-015/FU-4 shape) and confirms the OLD script passes on it while
  the NEW script fails on it, naming the file and the parser's own
  message. Verifies the parser is actually present on the CI runner
  image (per the DEC's Validation section) rather than trusted from
  the DEC alone.
- [ ] **ship** — CI observed green on the shipping SHA (AC3),
  Follow-ups table populated per AGENTS.md §15. At ship, either open
  frame stubs for the four deferred XS items in STAGE-005's backlog
  or record why not — `carried into the next build brief is not a
  disposition` (§15).
