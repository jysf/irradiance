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
- [x] **verify** — 2026-09-05, `HANDOFF-036`, `claude-opus-5[1m]`.
  ✅ **APPROVED at `a3f0063`** (CI 9/9, run `34003871323`; `src/`, `tests/`, `Cargo.*`,
  `fuzz/`, `.github/`, `scripts/` byte-identical across `2532dc2`/`7439f49`/`c57f88d`/`a3f0063`,
  so every measurement applies to all four). **8 follow-ups `FU-4`..`FU-11`, 0 ship-blockers.**
  Twelve gates + `lint-ci` run by me, all green — 150 passed / 0 failed summed across ten
  targets, **zero** SKIP lines; `lint-ci` force-relinted under clippy **0.1.98** asserted.
  Both red-proofs watched with `IRRADIANCE_CORPUS_DIR` **unset** (levels 0.499968 →
  **264.658371**, 15,841/17,408 px; orientation `[10,0,11,1,12,2]` → `[0,1,10,11,0,0]`, 6/6),
  and `AC6` confirmed against the **CI log** rather than the shape: both are named `ok` in run
  `34003871323`. `AC3` confirmed by grep with asserted counts — **0** match arms in either new
  file vs **8** in `src/develop.rs`; the oracle never reads `sensor.orientation`.
  **The blind spot is confirmed and it is worse than measured.** `Orientation 8`'s mapping
  where the file says 6: **46,712,160 / 46,726,912 px (100.0%)** positionally wrong, multisets
  byte-identical, all three tier-B tests green — caught only by three tier-A fixtures of ≤6 px,
  and the third of those catches it on its *positional* guard, not on its oracle. Then the same
  swap **gated on `crop_width > 100`**: same 100.0% corruption, **150/150 tests pass, nothing
  catches it**. So the mitigation that makes `DEC-020`'s limit acceptable covers only faults
  that manifest at ≤6 pixels. `DEC-020`'s own `## Validation` names this falsifier, it has now
  fired, and the remedy it points at (Option B) is *provably equivalent* to the shipped merge
  and shares the blind spot — the limit is **inherent**, and `## Consequences` **and**
  `## Validation` both need it (`FU-6`, `FU-7`).
  **`AC1`'s floor, measured for the first time: it catches `BlackLevel ± 1` at 9.1x the bound**
  (4.618424 / 4.618424 / 4.546309) on every frame — the fault `DEC-004` measured as
  SSIMULACRA2 **100.00**. And the map's own quantum is ≈4.13, so nothing can land between
  0.499968 and 4.13: the pre-registered `< 0.5` sits in a real gap. Also measured: the
  mutate-rebuild apparatus fails loudly all three ways it can be fooled; `FU-2`'s blind spot is
  closed and the rejected Option C provably had it; the negative control's *both* assertions are
  load-bearing; `AC2`'s true margin is **20.09% clipped share**, not 5 points (`L1000622` is at
  10.05%); `AC8` is 36.20 s serial = 0.3246 s/Mpx, which buys **exactly one** more Q2M-sized
  file (`FU-8`, `FU-9`). CI never runs tier B at all, and a size-gated *dimension* fault is
  caught only there (`FU-10`). All seven mutations ran in an isolated copy of the crate, so the
  working tree's `src/` was never edited — `AC7` in its strongest form. ⚠ `FU-4`: the spec's
  front matter has not been valid YAML since `c57f88d` (`handback-sync` truncated a multi-line
  scalar); every gate is blind to it and the next sync compounds it — one line, fix before
  syncing the verify cost.
- [ ] **ship**
