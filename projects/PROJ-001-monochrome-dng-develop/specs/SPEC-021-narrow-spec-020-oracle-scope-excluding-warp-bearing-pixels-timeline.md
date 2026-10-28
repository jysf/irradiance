# SPEC-021 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-021-<cycle>.md`.

## Cycles

- [x] **design** — 2026-10-27, main loop (orchestrator). Framed at SPEC-018 ship;
  designed here. A read-only Explore subagent gathered the inputs and corrected
  a key assumption: SPEC-020's SSIMULACRA2 oracle is `tests/perceptual_oracle.rs`,
  the two `#[ignore]`'d tests are SPEC-018's AC8/AC9 in `tests/warp.rs`, and the
  analytic replacement (AC4/AC10) already shipped — so this is docs + test-reframe,
  not a new oracle. AC1–AC6 written; failing tests specified.
- [~] **build** — `HANDOFF-058`. New DEC-026 amending DEC-005 (DEC-022 shape);
  `docs/oracle-contract.md` warp+Orientation carve-out; retire the two
  `#[ignore]`s (AC8 → passing characterization `-60.169 < +83.145` on
  Orientation-1 frames; L1026016 → explicit skip, not the latent `assert_eq!`
  panic); fix stale cross-refs (comment-only). 0 `src/` logic lines; red-proof
  already shipped as AC10 (cite, don't duplicate). Delegated to a cold session.
- [ ] **verify** — cold, independent: confirm DEC-026 + oracle-contract carve-out,
  that the reframed tests assert the real inversion and the Orientation skip is
  explicit (not a latent panic), that `src/` has 0 non-comment lines, and that
  AC4/AC10 are untouched. Separate session.
- [ ] **ship** — CHANGELOG, cost totals + `just cost-audit`, reflection, archive.
