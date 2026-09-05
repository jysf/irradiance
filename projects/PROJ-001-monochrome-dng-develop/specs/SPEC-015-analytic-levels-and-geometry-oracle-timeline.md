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
- [ ] **build** — `HANDOFF-035`.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
