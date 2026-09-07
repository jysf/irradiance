---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-043
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED — system prompt reports `message.model:
                                    # claude-sonnet-5`. Standing record now 0 for 13 on
                                    # the build hint (tier_map.build predicted opus-5).
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-020

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 36917935           # REAL combined count — what cost-audit reads
  estimated_usd: 15.23             # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: feat/spec-020-develop-oracle-vs-dnglab-srgb
  pr: null                         # not opened — Return Criterion 10, orchestrator opens it
  completed_at: 2026-09-06         # YYYY-MM-DD
  notes: "Deduped by message.id from own transcript (own scratchpad UUID, not text match); 250 usage objects, 136 distinct ids, raw combined 30,764,946 (input 272 / cache-write(1h) 315,710 / cache-read 30,335,786 / output 113,178), priced per-component at published claude-sonnet-5 rates ($3/$15/$6/$0.30 per Mtok, the SPEC-005/HANDOFF-022 precedent) = $12.69, +20% uplift for the remaining handback-writing turns = $15.23 and 36,917,935 tokens."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-043: Build SPEC-020 — the develop oracle vs dnglab srgb

## Delegation Summary

Build `SPEC-020`. It ships **STAGE-003's oracle before its subject** — a
SSIMULACRA2-scored comparator against `dnglab analyze --srgb`, with a
tier-A red-proof that runs in CI without the corpus. `SPEC-018`
(WarpRectilinear, the item that most decides `PROJ-001`'s thesis)
`depends_on: [SPEC-020]` explicitly, so its verify has a check it
cannot rewrite.

Branch is already created for you: `feat/spec-020-develop-oracle-vs-dnglab-srgb`
from `main` at `b0be356` (PR #12 merged, CI green). Rebase to the tip of `main`
before you start if it moved.

`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist. The tier-A red-proof MUST run with this
UNSET (`AC7`); the optional tier-B smoke against real dnglab output needs
it.

## ⚠ The one idea this whole spec turns on

**The oracle lands before the develop pipeline.** That is not a schedule
choice, it is a design invariant — the tier-A red-proof cannot say "our
develop output scores ≥ 85 against dnglab" because there is no develop
output yet. The proof is that **the METRIC responds to the fault classes
`DEC-005` calibrated**, on inputs the harness itself produces:

- identical inputs score ≥ 99.9 (`AC3`)
- 1-pixel shift scores < 85 (`AC4`; `DEC-005` measured 62.96 at ¼ res)
- missing radial warp scores < 85 (`AC5`; `DEC-005` measured −68.05)
- gamma 1.05 scores ≥ 85 (`AC6`; `DEC-005` measured 88.51 — the falsifier's
  mirror; a threshold so tight legitimate tone divergence fails would
  train us to ignore the oracle)

The same wiring a real developed render will later go through. Read
`## The design decision this spec rests on` in the spec before writing
anything.

## What is already measured — reproduce, do not re-derive

- **The `--srgb` shape holds on Q2M files**: 19-byte `P6 8368 5584 65535\n`
  header + `w*h*2` bytes of payload = 93,453,843 bytes, verified twice
  (`docs/oracle-contract.md` § "Layer 3 is not what this document
  originally said", 2026-08-18). `{ printf 'P5 8368 5584 65535\n';
  dnglab analyze --srgb F.DNG | tail -c +20; } > ref.pgm` is the shell
  reference; `AC1` productionises it into a typed reader with the length
  assertion.
- **The `≥ 85` threshold is `DEC-005`**, pre-registered with its falsifier
  (`a missing warp must land far below 85`). The calibration table in
  DEC-005's `## Context` is the anchor for `AC4`/`AC5`/`AC6`; do not
  re-derive numbers from other sources.
- **`ssimulacra2 = "0.5.1"` is crustyimg's pinned version**, resolvable
  from crates.io. Sibling repo integration:
  `~/PSeven/experiments/crustimg_redo_plus/crustyimg/src/quality/mod.rs:109`
  — `compute_frame_ssimulacra2(reference_rgb, candidate_rgb) -> Result<f64, _>`
  taking `Rgb::new(data: Vec<[f32; 3]>, w: usize, h: usize,
  TransferCharacteristic::SRGB, ColorPrimaries::BT709)`. Read that
  file to see the call shape — but ⚠ **do not copy its `::image` dep**;
  see the next section.
- **`SPIKE-001`'s warp coefficients** for `AC5`'s fault:
  `f(r) = 0.999251 + −0.06137651·r² + −0.0939155·r⁴ + 0.0558820·r⁶`,
  optical centre `(0.5, 0.5)`, r normalised at the corner (SPIKE-001
  flagged this as unconfirmed against DNG 1.7.0.0 — verify against the
  spec if the exact numeric locations of your synthetic peak matter).

## Two things that make this different from a normal build

**1. `no image crate`, and it stays out even test-side** (`library-not-
application`; SPEC-020 `## Non-Goals`). Crustyimg's `to_ss_rgb` decodes
via `::image::DynamicImage.to_rgb8()`. Do NOT admit `::image` in
`[dev-dependencies]` — the PNM reader is hand-written (`SPEC-013`'s
`parse_raw_pixel_pgm` is the precedent), and the 16→8 sRGB conversion is
one loop:

```rust
// per pixel: sample as f32 / 65535.0 → replicate 3× → Rgb::new(...)
```

**2. The dev-dep sanction lands with this build**, per AGENTS.md §13
rule 4 (a trivial DEV-only dep + its DEC in one pass). BEFORE writing
the DEC:
- Add `ssimulacra2 = "0.5.1"` to `[dev-dependencies]` in the root
  `Cargo.toml`.
- Run `cargo deny check licenses` (both graphs — `just deny` and
  `just deny-fuzz`) and confirm the license expression comes back
  as BSD/MIT/Apache. **If ANY string in the license expression is
  gpl/lgpl/agpl/unlicense, STOP AND ASK.** This is a
  `no-copyleft-dependencies` blocker, not a routine finding.
- Only then write the DEC (`decisions/DEC-NNN-ssimulacra2-devdep-
  sanction.md`), including alternatives considered (hand-roll, shell to
  `ssimulacra2` CLI — both rejected in the spec's AC9 with the
  reasons).

## The fixture choice, pre-registered

`SPEC-020`'s `## Implementation Context` says: bilinear-quality synthetic
input, ~1024×768, must satisfy ALL FOUR invariants (`AC3`/`AC4`/`AC5`/`AC6`).
If your chosen fixture cannot satisfy all four, **stop and report** — that
is the pre-registered rule; the threshold does not move, the fixture
does. Two candidate shapes to start:

1. A downscaled natural photograph with mid-frequency detail (foliage,
   brick, hair). Closer to real develop output SPEC-018/019 will
   produce; needs a licence-clean source.
2. A synthetic gradient + textured overlay (Perlin noise on a Gaussian
   smooth field). Reproducible without attribution.

Prefer 1 if you can find a licence-clean image (CC0 preferred); use 2
otherwise. Record the choice, the source (or generation seed), and the
measured scores on each of `AC3`/`AC4`/`AC5`/`AC6` in the handback —
those numbers become the anchor a future reviewer reproduces.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across
   all targets. `just lint-ci`, **not** `just lint` — local clippy is
   0.1.97 and CI floats at 0.1.98; assert the version you actually
   linted under. **Push and read CI** — `constraints.yaml` requires
   the gate *observed* green on your SHA.
2. **`src/` is 0 lines changed vs `main`.** Show it (`git diff --stat
   main...HEAD -- src/`). This is test-side + docs + Cargo.toml only,
   per `SPEC-020/AC10`.
3. **Watch all four tier-A fault tests fail (or pass) yourself, with
   `IRRADIANCE_CORPUS_DIR` unset**, and paste the measured scores.
   Every mutation on the fixture inputs: file changed **and** compiled
   **and** *output changed*. That third clause has caught four false
   red-proofs in three specs, and one in `PATCH-002` two days ago where
   the obvious injection exercised the wrong path.
4. ⚠ **Stage your work explicitly with `git add <paths>`, never `-A`.**
   The shared checkout has taken concurrent writes this session and once
   before (`SPEC-014/FU-8`, memory `shared-worktree-has-concurrent-
   writers`). A broad `add` could sweep another writer's in-flight
   changes into your commit.
5. **Confirm `ssimulacra2` licence BEFORE writing the DEC** (see § "Two
   things that make this different"). The DEC records the actual
   licence string from `cargo deny`, not an assumption.
6. **No fuzz target.** SPEC-020 adds no library-side input surface (the
   PNM reader parses **test-side** synthetic + dnglab output). Say so
   explicitly rather than adding one; §12 bar 2 does not fire (SPEC-020
   `## Non-Goals`).
7. **Provenance row** for SSIMULACRA2 (`docs/provenance-ledger.md`).
   Class **2 — third-party permissive crate** (not class 1 — did not
   implement from the paper; not class 3 — not reimplementing from
   another impl). Module: `tests/support/ssimulacra2.rs`.
8. **Handback with a real `tokens_total` deduped by `message.id`** from
   your own transcript, priced **per-component** at the rates for the
   model `message.model` reports, **rounded up ~20 %** to cover the
   turns that write the handback — measured at 9.9 %, 15.4 %, 19.2 %
   low across three prior sessions, and 20 % uplift landed the last
   one 3.1 % low.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block
   only, so `handback-sync` runs once cleanly. Hand-writing has caused
   four duplicate-entry cleanups.
   ⚠ **The project transcript directory also holds the ORCHESTRATOR's
   live session**, on a different model, text-matching this delegation
   because it wrote this handoff. It is **not** a prior attempt.
   Identify your own transcript by the uuid in **your own scratchpad
   path**, not by content match (signal `orchestrator-transcript-looks-
   like-a-prior-attempt`, `SPEC-014/FU-8`).
   ⚠ **`notes:` must be ONE PHYSICAL LINE.** `handback-sync` truncates
   multi-line YAML scalars, leaving front matter unparseable while every
   gate reports green (`SPEC-015/FU-4` + `FU-12`; signal
   `handback-sync-truncates-multi-line-scalars`, bar 2, open).
9. **Correct `handoff.to_agent`** to what your system prompt reports.
   Standing record: the build hint is **0 for 12**.
10. **Do not run `handback-sync`; do not open the PR.** The orchestrator
    reconciles the handback against git+disk first (DEC-004 rule 1) and
    opens the PR after.
11. **Findings `SB-N`/`FU-N`** with proposed §15 dispositions, numbering
    from `FU-1` (this spec's own sequence). A `spec:` disposition must
    **name an AC that would fail** without it. A `closed:` disposition
    must name a test that will fail if the closure is wrong, not a
    person to remember (§15 "a close whose trigger is someone
    remembering is not a good close").
12. Answer §15's reflection questions in the handback.

## Out of Scope

- **Any change to `src/`.** See Return Criterion 2 and `SPEC-020/AC10`.
  If the harness surfaces a call-shape defect in `develop_into`, that
  is a **finding for a follow-up spec**, not a reason to edit here.
- **`::image` crate**, in any position (`library-not-application`).
  Return Criterion "Two things that make this different" § 1.
- **A CLI shell-out to `ssimulacra2`** (the libjxl reference binary) —
  rejected in `SPEC-020/AC9`. The runtime dep is worse than a dev-dep.
- **Scoring against real developed output**. There is no develop
  pipeline yet; that is `SPEC-018` / `SPEC-019`'s AC.
- **Full-resolution recalibration of the ≥ 85 threshold**. `DEC-005`
  marks this as a revisit condition tied to when a real render exists
  — `SPEC-018` or `SPEC-019` do it, not this spec.
- **Levels checks.** `DEC-004` + `SPEC-015` cover them; adding
  perceptual coverage for levels is a category error (`DEC-005`).
- **Fuzz target.** Return Criterion 6; §12 bar 2 does not fire.
- **The `--srgb` output for non-monochrome cameras** (`SPEC-020/Non-Goals`).
- **Opening the PR, running `handback-sync`, touching STAGE-003's
  close.** The orchestrator does those.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter
block above AND the `## Handback` section below.** An unfilled handback
is an incomplete cycle, and `just handback-sync` will tell the
orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`
   and a ONE-PHYSICAL-LINE `notes:` scalar**.
2. Fill the `## Handback` section (the reflection is part of it, not
   optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Push the branch. **Do not open the PR** (Return Criterion 10).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked
   you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct
any of this — it transcribes it. The reflection questions are part of
completion.*

### Execution notes

- **Branch / PR:** `feat/spec-020-develop-oracle-vs-dnglab-srgb` pushed to
  `origin`, tip `db18c77642b67659b4376f06f53bc9495cb70100` (the code lands at
  `2e0d2c43126eed3095a5fdd82549f12156f88062`; this handback's own commit is
  on top, at the branch tip — git@github.com:jysf/irradiance.git). PR not
  opened, per Return Criterion 10.
- **Completed at:** 2026-09-06
- **All acceptance criteria met?** Yes — AC1–AC11 all met and tested. (AC7
  is a property of the other tests, not a separately-named one — confirmed
  by running the tier-A subset with `IRRADIANCE_CORPUS_DIR` unset, see
  below.)
- **Fixture chosen:** Synthetic, generated in code (candidate 2 from
  `## Implementation Context` — no licence-clean natural photograph was
  available without either sourcing a third-party image or decoding one via
  the forbidden `::image` crate). Fractional Brownian motion, hash-based
  value noise, 1024x768, base frequency 0.02, 5 octaves, contrast 1.6, seed
  1234 — `tests/perceptual_oracle.rs::synthetic_reference`, documented in
  full in `tests/oracle-fixtures/perceptual-oracle-fixture.md`.
- **Measured scores on the chosen fixture** (`IRRADIANCE_CORPUS_DIR`
  unset, `cargo test --all-features --test perceptual_oracle -- --nocapture`):
  - AC3 identical: **100.000** (≥ 99.9 ✓)
  - AC4 1-px shift: **61.823** (< 85 ✓ — caught)
  - AC5 missing warp: **−82.338** (< 85 ✓ — caught, emphatically)
  - AC6 gamma 1.05: **90.038** (≥ 85 ✓ — admitted)
- **`ssimulacra2` licence (from `cargo deny`):** `BSD-2-Clause` (the crate
  itself); full resolved graph (38 crates) is
  `MIT`/`Apache-2.0`/`BSD-2-Clause`/`Unicode-3.0` only — `cargo deny check
  licenses` → `licenses ok` in a probe workspace using this repo's real
  `deny.toml`. Corrects the design-time guess of "BSD-3-Clause OR MIT" — see
  `DEC-023`.
- **`src/` diff vs main:** empty. `git diff --stat main...HEAD -- src/`
  produces no output.
- **Eleven gates + `just lint-ci` locally** (all run on
  `2e0d2c4`, `IRRADIANCE_CORPUS_DIR` set for `just test`, unset separately
  for the tier-A confirmation below):
  - `cargo fmt --check` → clean
  - `cargo clippy --all-targets --all-features -- -D warnings` (unpinned,
    Homebrew 0.1.97) → clean
  - `just lint-ci` (pinned `~/.cargo/bin/cargo +stable`, prints
    **clippy 0.1.98 (88d9e12ae1 2026-08-18)** — CI's actual version) → clean
  - `just typecheck` (`cargo check --all-targets --all-features`) → clean
  - `just build` (`cargo build --release`) → clean
  - `just test` (`corpus-status` + `cargo test --all-features`, corpus
    present) → **172 tests total, 0 failed** — 66 lib unit tests + 8
    integration-test binaries (`corpus_manifest` 9, `develop` 7,
    `develop_oracle` 9, `ifd_reader` 12, `metadata_oracle` 30,
    `perceptual_oracle` **20 — new, this build**, `plane_oracle` 12,
    `plane_unpack` 7) + 0 doctests
  - `just deny` → `licenses ok`
  - `just deny-fuzz` → `licenses ok`
  - `just msrv` (`~/.cargo/bin/cargo +1.90.0 check --all-targets
    --all-features`) → clean
  - `just lint-red-proof` → `✓ lint policy red-proof: control clean (exit
    0) → injection rejected (exit 101)` — unaffected by this build, run to
    confirm no regression
  - `just lint-no-allow` (`cargo clippy --lib -F ...`) → clean
  - `./scripts/cost-audit.sh` → `✓` ; `./scripts/cost-audit-red-proof.sh` →
    `✓` ; `./scripts/decisions-index.sh --check` → clean (no committed
    `decisions/INDEX.md` on this branch — "nothing to keep in sync", exit 0)
  - **Confirmed the tier-A red-proof itself, with the corpus unset**
    (`env -u IRRADIANCE_CORPUS_DIR cargo test --all-features --test
    perceptual_oracle -- --nocapture`): all 20 tests pass, including the
    four fault tests, with the tier-B smoke test's `require()` printing
    `SKIP LEICA-Q2-MONO/L1021223.DNG — MISSING at …` and returning early —
    exactly the AC7 behaviour.
  - **No fuzz target added or run** — `src/` is unchanged and this spec adds
    no library-side input surface (Return Criterion 6 / `SPEC-020`
    Non-Goals); `ifd`/`plane`/`develop` fuzz targets are unaffected by
    test-only changes and `fuzz/` has its own, separate `Cargo.toml`
    (`DEC-011`) untouched by this build's root `Cargo.toml` edit.
  - Sum: **0 failures across every gate run.**
- **CI observed green on:** the code SHA
  `2e0d2c43126eed3095a5fdd82549f12156f88062` —
  <https://github.com/jysf/irradiance/actions/runs/34081190401> — all 9 jobs
  ✓, `"conclusion":"success"` via `gh run view --json`; AND, since
  constraints.yaml requires the gate observed green on the SHA actually
  shipping (not the last run before it), re-confirmed on this handback
  commit's own tip `db18c77642b67659b4376f06f53bc9495cb70100` —
  <https://github.com/jysf/irradiance/actions/runs/34081689175> — same 9
  jobs, `"conclusion":"success"`.

### Cost self-report

- **Tokens (total):** 36,917,935 (real, from this session's own transcript)
- **Estimated USD:** $15.23
- **Duration (minutes):** ~40
- **Source of the number:** own transcript (`.jsonl`), identified by this
  session's own scratchpad UUID
  (`d558fe94-e184-4f59-b109-1051c5325c96` — not by text-matching spec/task
  strings, which this project directory's memory
  `identify-own-transcript-for-cost-handback` warns can collide with a
  different session/model). 250 `usage` objects, **136 distinct
  `message.id`s** after dedup, all `message.model: claude-sonnet-5` (no
  pre-`/clear` contamination — the transcript starts at this delegation's
  own dispatch). Raw combined total **30,764,946** across the four
  components: input 272, cache-write (1h) 315,710, cache-read 30,335,786,
  output 113,178. Priced **per-component** at published `claude-sonnet-5`
  rates — $3 / $15 / $6 / $0.30 per Mtok for input / output / 1h-cache-write
  / cache-read respectively, the same rates `SPEC-005`'s `HANDOFF-022`
  handback used for the same model tier — giving $12.69, then **+20%**
  uplift (per Return Criterion 8's instruction, to cover the remaining
  turns spent finishing this handback) → **$15.23** and **36,917,935**
  tokens. `cost.sessions` itself is left untouched — only this `handback:`
  block is filled, so `just handback-sync` runs once cleanly.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-023` — `ssimulacra2` dev-dep sanction (required per `AC9`)
  - No second DEC for the mono-16→sRGB conversion: the chosen conversion
    (`sample as f32 / 65535.0`, no gamma applied — the samples are already
    sRGB-encoded from dnglab's output) is exactly `AC3`'s own pre-specified
    chain, not a subtler choice this build discovered, so `SPEC-020/##
    Outputs`'s conditional for a second DEC does not fire.
- **Deviations from spec:**
  1. Chose `tests/perceptual_oracle.rs` as a **new file** (Outputs option
     2), not `tests/develop_oracle.rs` extended with `mod perceptual {}`
     (option 1) — stated and reasoned in the file's own module doc: the two
     files share no fixture code (this spec's PNM/metric/perturbation
     concerns are wholly new), and `tests/develop_oracle.rs` is already
     43.8K.
  2. The fixture is generated **in code**, not committed as a binary blob
     to `tests/oracle-fixtures/` — that directory instead gained a markdown
     doc (`perceptual-oracle-fixture.md`) recording the generation
     parameters and measured scores. Follows this repo's own precedent
     (`tests/develop_oracle.rs`'s `FU-10` production-scale fixture is
     likewise in-test synthetic, not committed) rather than the literal
     reading of `## Outputs`' fixture-subdirectory bullet.
  3. The optional tier-B smoke test does **not** run a full end-to-end
     self-score against real `dnglab --srgb` output — see `FU-1` below.
- **Follow-up work identified** (this spec's own sequence, proposed
  dispositions per `AGENTS.md` §15):
  - **`FU-1` — the optional tier-B smoke test parses real `dnglab --srgb`
    output but does not score it end-to-end.** A full self-score at
    `L1021223.DNG`'s real resolution (8368x5584, 46.7 Mpixel) was measured
    at **~9.3s in `--release` alone** (a standalone probe, same call shape
    as `tests/support/ssimulacra2.rs::score`); `just test` runs in the
    `cargo test` debug profile, where this repo's own debug/release ratios
    for `ssimulacra2` scoring (measured on the 1024x768 fixture: 78ms
    release → 1.7s debug, ~22x) suggest **well over a minute**, paid on
    every local `just test` run that happens to have the corpus present,
    for a check the spec itself marks informational and non-gating.
    Proposed disposition: **`closed:`** — the substitute test
    (`dnglab_srgb_reader_parses_a_real_corpus_file`) already exercises the
    one load-bearing claim (the reader accepts the real defect shape, not
    just a hand-built one); the trigger that would prove this closure wrong
    is `SPEC-018`/`SPEC-019` needing a real end-to-end perceptual smoke
    test once a develop pipeline exists — that spec's own AC would say so
    and add it back, deliberately, rather than this one guessing at the
    cost/value tradeoff for a pipeline that doesn't exist yet.
  - **`FU-2` — `SPEC-020.md`'s `## Implementation Context` design-time
    citation of `ssimulacra2`'s dependency count ("Dependencies:
    `num-traits` only") is wrong.** Measured: 38 crates in the resolved
    graph (`rayon`, `yuvxyb`, `av-data`, `v_frame`, etc.) — recorded
    precisely in `DEC-023`. Proposed disposition: **`closed:`** — `DEC-023`
    carries the corrected count and the exact method (`cargo tree` in a
    probe workspace) any future citation of this crate's graph should
    re-run rather than trust. Residual risk named honestly: there is no
    mechanical trigger that would catch a *future* re-drift between a
    design doc's claim and reality of this same shape — only the habit of
    re-measuring, which this build followed and the next one must too.
  - **`FU-3` — the design-time licence guess ("BSD-3-Clause OR MIT") was
    also wrong** (measured: `BSD-2-Clause`). Proposed disposition:
    **`closed:`** — the real, mechanical trigger already exists and is
    exercised on every push: `just deny` / the CI `licenses` job would fail
    loudly on any future `ssimulacra2` release whose licence (or graph)
    turns non-permissive, independent of which SPDX id was originally
    guessed. `DEC-023` records the corrected figure.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two small things. First, `AC3`'s second required test name,
   `sixteen_to_eight_conversion_is_deterministic`, describes a conversion
   the AC's own prose does not perform — the pinned chain is 16-bit → `f32`
   directly (`sample as f32 / 65535.0`), with no 8-bit step anywhere (that
   language appears to be inherited from crustyimg's *8-bit* source image
   case, which this spec's 16-bit PNM input does not share). Kept the exact
   name per the Failing Tests header's own warning that a zero-match
   `cargo test <name>` exits 0 silently, and used it for the determinism
   check `AC3` actually describes. Second, the tier-A/tier-B split for the
   optional smoke test wasn't pre-registered with a cost budget, so the
   ~9.3s-per-full-score measurement (`FU-1`) was a build-time discovery
   rather than a known constraint going in.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** — No. `DEC-005`, `DEC-004`, `DEC-013`, `DEC-017` and
   `guidance/constraints.yaml`'s five blocking constraints covered
   everything this build touched; `DEC-002`'s `no rayon` proposal was the
   one worth double-checking against (`ssimulacra2`'s graph pulls `rayon`
   in) and `DEC-023` states explicitly why that rule doesn't fire here (it
   scopes the *library's* runtime graph, not a dev-only test tool's).
3. **If you did this task again, what would you do differently?** — Run
   the fixture-parameter probe (the standalone scratch harness against the
   real crate) even earlier — it took under a minute to get a clear answer
   that a wide swath of the parameter space satisfies all four invariants,
   which meant zero time was lost to picking-and-re-picking a fixture that
   turned out wrong, but that was closer to luck than plan given how the
   spec's own `## Implementation Context` flags this as a real risk
   ("if no fixture satisfies all four, stop and report").
