# SPEC-015 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-015-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-05, main loop. Probe RAN against all three decodable
  frames (111,529,040 pixels) and settled the spec's central question: whether an
  analytic oracle must **reimplement** the transform (weak independence) or can
  assert **properties** of it (strong). It can. Measured: the shipped output is
  within **0.499968 LSB** of the exact real-valued affine map on every pixel,
  **zero** at or above 0.5 — so `AC1`'s tolerance is pre-registered from evidence,
  and stated rule-agnostically so it is satisfied by any correct rounding and
  violated by any wrong map. **45.0–50.1 %** of pixels differ from a truncated
  map, so an oracle written with `floor` fails half of every frame (`DEC-018`'s
  warning, now numbers). `histogram(output) == histogram(normalized crop window)`
  held **exactly** on all three, including the `Orientation 6` frame, **without
  the eight-case table appearing anywhere** — and that property catches both
  faults the existing oracles miss: `SPIKE-001`'s `BlackLevel + 64`
  (36,824,570 pixels wrong, 78.8 % — the fault SSIMULACRA2 scores **95.62,
  passing** on and `--raw-checksum` is bit-identical on) and `SPEC-014/FU-3`'s
  orientation identity (15,425,929 wrong, 33 % — the fault that left 141/141
  green). Nine ACs, six failing tests, red-proof required **tier-A** per
  `SPEC-013/FU-1`. Complexity raised **S → M**: the stage's S predates the probe.
  `HANDOFF-035` ready.
- [x] **build** — 2026-09-05, `HANDOFF-035`, `feat/spec-015-analytic-levels-and-geometry-oracle`
  at `2532dc2`, CI observed green (9/9 jobs, run `34000895054`). All nine ACs met; `src/develop.rs`,
  `src/plane.rs`, `src/ifd.rs` 0 lines changed (AC7) — no defect found. Every measured number on
  the three real files reproduces the design probe's `## Implementation Context` exactly. Two new
  decisions (`DEC-020`, `DEC-021`); no fuzz target (adds none). 150 tests (was 143), 0 failed, 0
  skipped. Three follow-ups, zero ship-blockers — see `HANDOFF-035`'s Findings.
- [ ] **verify** — `HANDOFF-036`, at `7439f49`. Orchestrator reconciled first
  (`DEC-004` rule 1): CI **9/9 on both** SHAs, `src/` empty diff, every number
  reproduced, **both red-proofs re-run with the corpus absent**, and `AC3`
  confirmed by grep rather than assertion — **zero** orientation match arms in
  either new file. **And measured a blind spot to hand on:** applying
  `Orientation 8`'s mapping where the file says 6 — a valid, same-multiset
  permutation — leaves **all three** tier-B oracle tests green on 46,726,912
  real pixels. It is caught only by positional tier-A fixtures of ≤6 pixels.
  That is `DEC-020`'s inherent price rather than a defect, but nothing records
  it; verify judges whether it belongs in that decision's `## Consequences`.
  Six further checks handed on, including one nobody has measured — `AC1`'s
  sensitivity floor at `BlackLevel + 1`, which `DEC-004` measured as
  SSIMULACRA2 100.00, i.e. invisible to the develop oracle.
- [ ] **ship**
