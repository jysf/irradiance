---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-037
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-003
  stage: STAGE-009
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-087 (build) → tier_map.build
  created_at: 2026-11-05

references:
  decisions: [DEC-035, DEC-016, DEC-018]  # SOF-3 patent-clean/provenance (DEC-035); caller-owned buffer (DEC-016); u16 modular (DEC-018)
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, test-before-implementation]
  related_specs: [SPEC-036, SPEC-038]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-036]             # consumes the entropy front-end's difference stream

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-009's <capability>". Optional; null is acceptable.
value_link: "Lossless reconstruction (predictors 1–7, initial prediction, point transform, restart reset) + the analytic encode→decode round-trip oracle — turns SPEC-036's difference stream into the exact caller-owned u16 plane, completing the STAGE-009 decoder's correctness half."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 34000000        # ~34M — reconstruction + the in-harness LJ92 encoder oracle; [M], slightly over SPEC-036
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 40
      recorded_at: 2026-11-05
      notes: "main-session design (un-metered); reconstruction + round-trip oracle spec + HANDOFF-087 + build prompt"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-037: Lossless reconstruction (predictors, point transform, restart)

## Context

SPEC-036 decodes a SOF-3 scan down to per-sample (category → difference) values;
those differences are not samples until they are run through the lossless
predictor. This is the second of `STAGE-009`'s backlog and delivers
**reconstruction**: predictors 1–7, the initial prediction
`Px = 2^(P−Pt−1)`, the point transform `Pt`, and restart-interval resets,
writing the reconstructed samples into a caller-owned `u16` buffer
(DEC-016 / DEC-018). It also builds the stage's corpus-free independent oracle —
a tiny in-harness LJ92 encoder — so an encode→decode round-trip can assert
bit-exactness and be shown RED on a perturbed predictor, Huffman table, or point
transform (constraint `oracle-must-be-shown-red`). SPEC-036 + SPEC-037 together
are the complete single-component decoder; SPEC-038 then fuzzes it.

- Parent: `STAGE-009` (SOF-3 lossless-JPEG core decoder), backlog position 2 of 3
- Project: `PROJ-003` (compressed planes — lossless JPEG)
- Blocked on SPEC-036 (its difference stream is this spec's input)
- Standard: ITU-T T.81 (1992) Annex H — a provenance-ledger row lands here

## Goal

Reconstruct a single-component SOF-3 plane from SPEC-036's decoded difference
stream — applying predictors 1–7, the initial prediction, the point transform,
and restart resets — into a caller-owned `u16` sample buffer, and prove it with
an analytic encode→decode round-trip oracle that is bit-exact on honest input
and demonstrably RED on a perturbed predictor, Huffman table, or point
transform.

## Inputs

- **Standard (implement from this):** ITU-T T.81 (1992) — lossless JPEG. **Annex
  H.1** (the lossless mode: prediction, the predictor set, modular reconstruction),
  **§H.2.1** (prediction at line/scan/restart boundaries), and the point transform
  `Pt`. `RECEIVE`/`EXTEND` and the entropy layer are already done in SPEC-036.
  Patent/provenance posture is `DEC-035`.
- **Code to read / build on (the SPEC-036 interface, already on `main` — the §12
  probe; use these EXACT names):**
  - `src/ljpeg.rs` — `pub(crate) struct Frame { precision: u8, lines: u16,
    samples_per_line: u16, point_transform: u8, predictor: u8, restart_interval:
    u16 }`; `pub(crate) fn parse_headers(&[u8]) -> Result<(Frame, HuffTable,
    usize)>` (the `usize` is the entropy-data offset); `ScanReader::new(entropy:
    &[u8])`, `ScanReader::decode_difference(&HuffTable) -> Result<i32>` (one signed
    difference — `SSSS==0 → 0`, `SSSS==16 → 32768` already handled),
    `ScanReader::restart() -> Result<()>` (byte-aligns + consumes the next `RSTm`
    in the `D0..D7` cycle; **resetting the predictor is this spec's job**). This
    spec adds the reconstruction that turns those differences into samples.
  - `src/lib.rs` (`pub enum Error`) — the 7 `Ljpeg*` variants SPEC-036 added, and
    the `*WrongLength` convention (e.g. `PlaneBufferWrongLength`, ~line 347): the
    one new variant here follows both (`LjpegPlaneWrongLength`).
  - `src/plane.rs` — the `u16` plane is **row-major, one sample per pixel**,
    uncropped/un-normalised (the representation STAGE-010 will checksum against
    `dnglab`); match it.
- **Cross-reference only (read for structure; never copied/linked — clean-room,
  `DEC-035`):** `oxideav-mjpeg`, `photocraft-raw`.
- **No external crate, no corpus file:** reconstruction is proven on hand-built
  scans and the in-harness round-trip encoder (STAGE-010 adds the real-file
  `dnglab --raw-checksum` check).

## Outputs

- **Files modified:** `src/ljpeg.rs` (the reconstruction, same `pub(crate)`
  module — SPEC-036 is its front-end); `src/lib.rs` (one new `Error` variant);
  `docs/provenance-ledger.md` (a row for the lossless reconstruction — predictors
  / initial prediction / point transform, T.81 Annex H.1, `DEC-035`).
- **New items (crate-internal):**
  - `pub(crate) fn reconstruct(frame: &Frame, table: &HuffTable, reader: &mut
    ScanReader<'_>, out: &mut [u16]) -> Result<(), Error>` — the core. Requires
    `out.len() == frame.lines as usize * frame.samples_per_line as usize` (else
    `LjpegPlaneWrongLength`), then fills `out` **row-major** by decoding one
    difference per sample (`reader.decode_difference(table)`) and adding the
    prediction. **No allocation** on this path (caller-owned buffer, `DEC-016`).
  - `pub(crate) fn decode_into(data: &[u8], out: &mut [u16]) -> Result<Frame,
    Error>` — the one-call single-component decoder STAGE-010 uses:
    `parse_headers` → length check → `ScanReader::new(&data[offset..])` →
    `reconstruct`; returns the `Frame`.
- **Reconstruction (from T.81 Annex H.1 — the build implements the exact
  arithmetic; this is the contract):**
  - **Neighbours** at sample (row `r`, col `c`), in the point-transformed domain:
    `Ra` = left `(r, c-1)`, `Rb` = above `(r-1, c)`, `Rc` = above-left `(r-1,
    c-1)`.
  - **Predictor `Px`** (`frame.predictor`, T.81 Table H.1): `1`→`Ra`, `2`→`Rb`,
    `3`→`Rc`, `4`→`Ra+Rb−Rc`, `5`→`Ra+((Rb−Rc)>>1)`, `6`→`Rb+((Ra−Rc)>>1)`,
    `7`→`(Ra+Rb)>>1`.
  - **Boundary rules (§H.2.1):** the first sample of the scan **and** the first
    sample after each restart use `Px = 2^(P−Pt−1)` (`P = precision`, `Pt =
    point_transform`); the rest of the **first line** uses `Ra`; the **first
    column** of each later line uses `Rb`; every other sample uses the selected
    predictor.
  - **Sample** = `(Px + difference) mod 2^16` (explicit `wrapping_*`, `DEC-018`),
    computed in the point-transformed domain; the stored output is that value
    **`<< Pt`** (for the held files `Pt == 0`, so output == reconstructed; the
    general `Pt` is covered and STAGE-010's `dnglab` check confirms it).
  - **Restart:** when `frame.restart_interval > 0`, after every `restart_interval`
    samples call `reader.restart()` and reset the prediction to the scan-start rule
    for the next sample; `restart_interval == 0` → no restarts.
  - `frame.predictor` is already validated `1..=7` by `parse_headers` — reconstruct
    trusts that but still never panics on an unexpected value.
- **New `Error` variant:** `LjpegPlaneWrongLength { expected: usize, got: usize }`
  — the `out` buffer does not match `lines × samples_per_line` (follows the
  `Ljpeg*` family and the crate's `*WrongLength` convention).
- **No public API, no CLI, no flags** (`irr` unchanged; STAGE-010 wires the
  container and may add an `irr` path).

## Acceptance Criteria

- [ ] Each predictor `1..=7` reconstructs an interior sample correctly from
  hand-built differences + neighbours (`Ra`/`Rb`/`Rc`), per T.81 Table H.1.
- [ ] The §H.2.1 boundary rules hold: the first sample of the scan is
  `2^(P−Pt−1)`; the rest of the first line uses `Ra`; the first column of later
  lines uses `Rb`; the first sample after a restart is `2^(P−Pt−1)`.
- [ ] The point transform is applied: with `Pt > 0`, output `== reconstructed <<
  Pt` and prediction arithmetic is in the point-transformed domain; with `Pt == 0`
  output `==` reconstructed. (tested at `Pt = 0` and a `Pt > 0` case)
- [ ] A scan with `restart_interval > 0` reconstructs across `RSTm` boundaries
  with the predictor reset at each; `restart_interval == 0` decodes with no
  restarts.
- [ ] **The analytic encode→decode round-trip is bit-identical** for honest input
  across predictors `1..=7`, precisions `{8, 12, 16}`, `Pt ∈ {0, k}`, and with and
  without a restart interval.
- [ ] **`oracle-must-be-shown-red`:** a deliberately perturbed decoder (a predictor
  formula, the `EXTEND`, or the `Pt` shift) makes the round-trip assertion go
  **RED**, with the honest control green — proving the assertions constrain the
  real reconstructed values (the `plane_oracle.rs` no-op lesson).
- [ ] `out.len() != lines × samples_per_line` returns `Error::LjpegPlaneWrongLength`
  and **never panics**; the reconstruction path allocates nothing (`DEC-016`).
- [ ] Reconstruction arithmetic is modular `2^16` via explicit `wrapping_*`
  (`DEC-018`); compiles under `#![forbid(unsafe_code)]` + the five-lint deny
  policy; `just test`, `just lint-ci -D warnings`, `just fmt-check`, `just
  validate` green.
- [ ] `docs/provenance-ledger.md` carries the reconstruction row (T.81 Annex H.1,
  `DEC-035`).

## Failing Tests

All in `src/ljpeg.rs` under `#[cfg(test)] mod tests` (with SPEC-036's). The
in-harness **LJ92 encoder** (`fn encode_lj92(samples, cols, rows, precision,
predictor, point_transform, restart_interval) -> Vec<u8>` — SOI / SOF3 / DHT /
optional DRI / SOS / Huffman-coded + `EXTEND`ed + `0xFF`-stuffed scan + `RSTm` /
EOI) is the oracle, and also the seed generator for SPEC-038.

- **`src/ljpeg.rs` (`mod tests`)**
  - `"reconstruct applies each predictor 1..=7"` — hand-built `Frame` + scan with
    known differences; assert `out` matches the hand-computed samples for each
    predictor at an interior position.
  - `"reconstruct honours the H.2.1 boundary rules"` — first sample `= 2^(P−Pt−1)`,
    first line `= Ra`, first column `= Rb`.
  - `"reconstruct applies the point transform"` — a `Pt > 0` frame: assert `out ==
    reconstructed << Pt` and that prediction used the point-transformed neighbours.
  - `"reconstruct resets prediction at each restart"` — `restart_interval > 0`:
    reconstruct across an `RSTm` and assert the sample after it used `2^(P−Pt−1)`.
  - `"round_trip is bit-identical across predictors, precisions and Pt"` —
    `encode_lj92(samples, …)` → `decode_into` → `assert_eq!(out, samples)` over the
    matrix (predictors `1..=7` × precision `{8,12,16}` × `Pt {0,2}` × restart
    on/off).
  - `"round_trip red-proof: a perturbed predictor breaks the round-trip"` — a
    staged mutant decoder with one predictor formula altered makes the round-trip
    `assert_eq!` fail; the honest control passes (assert mutant `!=` honest).
  - `"wrong out length is a typed error"` — `out.len()` wrong →
    `Error::LjpegPlaneWrongLength`, no panic.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The marker/frame parse, JPEG bit reader, and DC Huffman decode — that is
  SPEC-036 (this spec consumes its output).
- The fuzz / truncation-hardening target — SPEC-038.
- Real-file verification against `dnglab --raw-checksum` and the DNG
  `Compression 7` container binding — STAGE-010.
- Multi-component interleave, lossy DCT, arithmetic coding, Nikon NEF-lossless,
  Pentax PEF.

## Notes for the Implementer

- **Build on SPEC-036, don't re-do it.** `parse_headers`, `Frame`, `HuffTable`,
  and `ScanReader` are already shipped (`src/ljpeg.rs`, `pub(crate)`). Call
  `reader.decode_difference(table)` once per sample and `reader.restart()` at each
  restart boundary; do not re-parse markers or re-decode Huffman.
- **The prediction domain is point-transformed.** Do all of `Ra/Rb/Rc` + the
  predictor + the initial `2^(P−Pt−1)` in the `(P−Pt)`-bit domain, modular `2^16`
  with explicit `wrapping_add`/`wrapping_sub` (`DEC-018` — satisfy
  `arithmetic_side_effects` by intent, never a silencing cast). The stored output
  is `reconstructed << Pt`. The held corpus files are `Pt == 0`; implement the
  general `Pt` anyway (cheap, and STAGE-010's `dnglab` check would catch it).
- **The `>>1` in predictors 5/6/7 is a floor on the signed intermediate** — mind
  sign (use `i32` working values, not `u16`), per T.81 H.1.
- **Caller-owned, no alloc (`DEC-016`).** `reconstruct` writes into the caller's
  `&mut [u16]`; the only length check is `out.len() == lines × samples_per_line`.
  No per-sample allocation, no intermediate difference buffer.
- **RAW is attacker-influenced (`no-panics-on-untrusted-input`).** Every neighbour
  access and restart is bounded; a short scan / wrong restart surfaces as a typed
  `Error` from `ScanReader`, never a panic. SPEC-038 fuzzes the whole decoder.
- **The round-trip oracle must be shown RED (`oracle-must-be-shown-red`).** Don't
  just assert green round-trips — include the perturbed-decoder red-proof so a
  broken predictor/`EXTEND`/`Pt` is provably caught (the `plane_oracle.rs` no-op
  lesson, and SPEC-030/031's perturbed-kernel pattern). The `encode_lj92` harness
  also seeds SPEC-038's fuzz corpus — keep it clean and parameterised.
- **Clean-room from T.81 (`DEC-035`).** Implement the predictors / boundary rules
  / point transform from Annex H; the cross-reference crates (`oxideav-mjpeg`,
  `photocraft-raw`) are read-for-structure only, never copied or linked. Add the
  provenance-ledger row.
- **Pre-named L-risk split (the build's escape hatch).** Designed whole at [M]. If
  it runs long, split along: (a) `reconstruct` (predictors + boundary rules + `Pt`
  + restart) with direct hand-built-scan tests, then (b) the `encode_lj92`
  round-trip oracle + the red-proof. Ship (a) then (b); don't expand scope.
- **§12 probe (done at design against `main`):** the real surfaces are the shipped
  SPEC-036 interface (`Frame` fields, `ScanReader::decode_difference -> i32`,
  `restart`, `parse_headers -> (Frame, HuffTable, usize)`) and the `*WrongLength`
  error convention in `src/lib.rs`. No runtime baseline — correctness, not
  throughput, is the target.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
