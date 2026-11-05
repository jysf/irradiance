---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-038
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  to_agent: claude-opus-5          # HANDOFF-089 (build) → tier_map.build
  created_at: 2026-11-05

references:
  decisions: [DEC-036, DEC-035]    # fuzzing-feature exposure (DEC-036); decoder is clean-room/patent-clean (DEC-035)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-036, SPEC-037]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-037]             # fuzzes the full decoder (SPEC-036 + SPEC-037)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-009's <capability>". Optional; null is acceptable.
value_link: "Hardens the full single-component SOF-3 decoder (SPEC-036 + SPEC-037) to panic-freedom on hostile, truncated, and oversized-table bitstreams — the safety half of STAGE-009's deliverable."

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
  tokens_estimate: 20000000        # ~20M — [S/M]: the fuzz target + feature exposure (DEC-036) + seed-gen + replay test
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-11-05
      notes: "main-session design (un-metered); fuzz target spec + DEC-036 (fuzzing-feature exposure) + HANDOFF-089 + build prompt"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-038: SOF-3 decoder fuzz + truncation hardening

## Context

RAW input is attacker-influenced, so the constraint `no-panics-on-untrusted-input`
governs the whole crate; a bitstream decoder is the highest-risk surface for it.
Once SPEC-036 + SPEC-037 form the complete single-component SOF-3 decoder, this
third spec of `STAGE-009`'s backlog adds a libFuzzer target over that decoder,
seeded from the round-trip encoder SPEC-037 builds, and drives it on hostile /
truncated / oversized-table bitstreams to prove it returns typed errors rather
than panicking, under `#![forbid(unsafe_code)]` and the five-lint deny policy.

- Parent: `STAGE-009` (SOF-3 lossless-JPEG core decoder), backlog position 3 of 3
- Project: `PROJ-003` (compressed planes — lossless JPEG)
- Blocked on SPEC-037 (the full decoder, and the encoder its seeds come from)
- A fuzz dependency is a design-time check against
  `no-new-top-level-deps-without-decision` — the existing fuzz harness is reused
  where possible

## Goal

Add a libFuzzer target over the full single-component SOF-3 decoder (SPEC-036 +
SPEC-037) that runs clean on hostile, truncated, and oversized-table LJ92
bitstreams — every path returning a typed `Error`, never a panic — with seeds
generated from SPEC-037's round-trip encoder.

## Inputs

- **The decoder being fuzzed (shipped, SPEC-036/037):** `src/ljpeg.rs` —
  `pub(crate) fn decode_into(data: &[u8], out: &mut [u16]) -> Result<Frame,
  Error>` (the full single-component decode), `parse_headers`, `reconstruct`, and
  the test-only `encode_lj92` / `encode_scan` (in `mod tests`) — the shape the
  seeds mirror.
- **`DEC-036`** — the exposure decision: a `fuzzing` cargo feature (NOT
  `cfg(fuzzing)`), because the `clippy — fuzz graph` gate builds the fuzz crate
  with plain `cargo clippy` (no `--cfg fuzzing`). Read it before wiring visibility.
- **The fuzz harness to mirror:**
  - `fuzz/fuzz_targets/plane.rs` — the closest analog (a decoder target that
    bounds a hostile `width×height` with `MAX_PIXELS` before allocating its own
    buffer; "declining to chase OOM is the target's choice, not a library
    guarantee weakened"). Copy its shape.
  - `fuzz/Cargo.toml` — the `[[bin]]` list + `[dependencies.irradiance] path =
    ".."` (add `features = ["fuzzing"]`).
  - `examples/fuzz-seeds.rs` — the committed seed writer; it pulls shared
    fixtures via `#[path = "../tests/support/*.rs"]` and builds some seeds
    locally (`plane_fixture`, "kept in sync by hand" — the precedent for a
    local SOF-3 seed builder).
  - `.github/workflows/ci.yml` (`clippy-fuzz` / `licenses-fuzz`) — the gates that
    auto-cover a new target; `tests/opcode.rs` — the seed-replay panic-free test
    pattern (here it becomes an in-crate unit test, since `decode_into` is
    `pub(crate)`).
- **No corpus file, no external crate.** `libfuzzer-sys` is the existing fuzz
  dev-dependency; seeds are own-work SOF-3 bitstreams (`DEC-003`).

## Outputs

- **`Cargo.toml`:** a `[features]` section with `fuzzing = []` (empty — exposure
  only, no deps; `DEC-036`).
- **`src/lib.rs`:** `#[cfg(feature = "fuzzing")] #[doc(hidden)] pub fn
  __fuzz_ljpeg(data: &[u8])` — parse the header, **bound** `lines ×
  samples_per_line` with checked arithmetic against a `MAX_PIXELS` cap (mirror
  `plane.rs`), allocate a capped `u16` buffer, call `ljpeg::decode_into`, and drop
  the typed `Result`. Reaches the `pub(crate)` decoder (same crate); satisfies the
  five-lint policy (it is compiled under `--all-features`). Default builds compile
  it out — `ljpeg` stays `pub(crate)`.
- **`fuzz/Cargo.toml`:** `[dependencies.irradiance]` gains `features =
  ["fuzzing"]`; a new `[[bin]] name = "ljpeg" path = "fuzz_targets/ljpeg.rs" test
  = false doc = false bench = false`.
- **`fuzz/fuzz_targets/ljpeg.rs`:** `#![no_main]` + `fuzz_target!(|data: &[u8]| {
  irradiance::__fuzz_ljpeg(data); })`, with a header stating the contract (every
  byte string returns; nothing panics) + the `cargo +nightly fuzz run ljpeg
  fuzz/corpus/ljpeg fuzz/seeds/ljpeg -- -max_total_time=60` recipe.
- **`examples/fuzz-seeds.rs`:** a `ljpeg_seeds()` writer → `fuzz/seeds/ljpeg/`,
  from a local SOF-3 builder that mirrors `encode_lj92` (hand-synced like
  `plane_fixture`, or a shared `tests/support/ljpeg.rs`): valid scans across
  predictors / precision / `Pt` / restart, plus the hostile shapes (truncated
  header, truncated scan, oversized DHT, `Nf≠1`, bad marker).
- **`fuzz/seeds/ljpeg/*`:** the committed seed files.
- **`src/ljpeg.rs` (`mod tests`):** the panic-free replay test (below) — a UNIT
  test, so it reaches `decode_into` directly without the feature.
- **No new runtime dependency, no public API** (`ljpeg` stays `pub(crate)`; the
  `fuzzing` feature is dev-only exposure, `DEC-036`). **No CI yaml change
  required** — the `clippy — fuzz graph` + `license policy — fuzz graph` gates
  build the whole fuzz crate and so cover the new target; a `fuzz smoke — ljpeg`
  job is optional (the smoke set is curated — only `opcode` + `warp_opcode`).

## Acceptance Criteria

- [ ] A `ljpeg` libFuzzer target (`fuzz/fuzz_targets/ljpeg.rs` + the `[[bin]]`)
  drives the full decoder via the `fuzzing`-feature entry; `cargo +nightly fuzz
  run ljpeg fuzz/corpus/ljpeg fuzz/seeds/ljpeg -- -max_total_time=60` runs clean
  (documented; the long run is manual, not CI).
- [ ] The `clippy — fuzz graph (-D warnings)` and `license policy — fuzz graph`
  gates stay green with the new target — i.e. plain `cargo clippy --manifest-path
  fuzz/Cargo.toml --all-targets` compiles it (the `fuzzing` **feature**, not
  `cfg(fuzzing)`, per `DEC-036`).
- [ ] A **panic-free replay unit test** runs every committed `fuzz/seeds/ljpeg/`
  seed **and** a set of hostile / truncated / oversized-table inputs through the
  decoder and asserts **no panic** — each returns `Ok` or a typed `Error`. Runs in
  `just test`.
- [ ] `cargo run --example fuzz-seeds` (re)writes `fuzz/seeds/ljpeg/` with valid
  scans (predictors, precision, `Pt`, restart) + the hostile shapes; the seeds are
  committed and own-work (`DEC-003`).
- [ ] `ljpeg` stays `pub(crate)`: a default-feature `cargo build` / `cargo doc`
  exposes no `ljpeg` symbol and no `__fuzz_ljpeg`; the entry is `#[doc(hidden)]`
  and compiled out without the feature (`DEC-036`).
- [ ] `#![forbid(unsafe_code)]` + the five-lint deny policy hold on the
  feature-gated entry (compiled under `--all-features`); `just test`, `just
  lint-ci`, `just fmt-check`, `just validate`, `just lint-fuzz`, `just deny-fuzz`
  all green.
- [ ] No new runtime dependency (the `fuzzing` feature is empty; `libfuzzer-sys`
  is the existing fuzz dev-dep) — `no-new-top-level-deps-without-decision`
  satisfied, recorded in `DEC-036`.

## Failing Tests

The panic-free guarantee is enforced in `just test` by in-crate unit tests (they
reach `pub(crate) decode_into` directly — no feature needed); the libFuzzer run
itself is manual (`cargo +nightly fuzz run`).

- **`src/ljpeg.rs` (`mod tests`)**
  - `"fuzz_seed_corpus_decodes_without_panic"` — read every file in
    `<CARGO_MANIFEST_DIR>/fuzz/seeds/ljpeg/`, run each through `decode_into` into a
    bounded buffer, and assert each **returns** (`Ok` or a typed `Error`) — i.e.
    the whole committed seed corpus is panic-free. (Also asserts the corpus is
    non-empty, so a missing seed-gen fails loudly.)
  - `"hostile_bitstreams_are_typed_errors_not_panics"` — a table of crafted
    inputs, each asserted to be a typed `Error` (never a panic): empty input; a
    valid header then a truncated scan; a `SOF3` with `Nf = 3`; a DHT whose `BITS`
    oversubscribe (sum > codes / overruns 16 bits); a forged `65535 × 65535`
    dimension; a bad/again-wrong `RSTm`.
- **`examples/fuzz-seeds.rs`** — `cargo run --example fuzz-seeds` writes a
  non-empty `fuzz/seeds/ljpeg/` (verified by the corpus test above, which the
  seed-gen must satisfy).

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The decoder itself — SPEC-036 (front-end) and SPEC-037 (reconstruction); this
  spec only exercises them.
- Correctness / bit-exactness verification (the round-trip oracle is SPEC-037;
  real-file `dnglab` checksum is STAGE-010) — this spec targets panic-freedom,
  not output values.
- Fuzzing the DNG container path or multi-component scans — STAGE-010 and a
  later wave.

## Notes for the Implementer

- **The exposure is a `fuzzing` cargo FEATURE, not `cfg(fuzzing)` — `DEC-036`.**
  The `clippy — fuzz graph` gate runs plain `cargo clippy --manifest-path
  fuzz/Cargo.toml` (no `--cfg fuzzing`, no nightly), so a `cfg(fuzzing)` entry
  would be absent there and the target wouldn't compile → gate red. Enable the
  feature on the fuzz crate's `irradiance` dep.
- **The panic-free replay is an in-crate UNIT test** (`src/ljpeg.rs mod tests`),
  so it reaches `decode_into` directly — do **not** route it through the feature
  entry or widen `decode_into`.
- **Bound dimensions in `__fuzz_ljpeg`** with checked arithmetic (cap pixels, as
  `plane.rs` does with `MAX_PIXELS`) before allocating — the target declines to
  chase an OOM on `65535×65535`; that is the target's choice, not a weakened
  library guarantee (`decode_into` would reject a too-small buffer with
  `LjpegPlaneWrongLength` anyway). The entry must be five-lint clean (it compiles
  under `--all-features`): no `unwrap`/`expect`/indexing/`panic`, checked/wrapping
  arithmetic, drop the `Result` with `let _ =`.
- **Seeds are own-work SOF-3 bitstreams (`DEC-003`)** — build them in
  `examples/fuzz-seeds.rs` from a local builder mirroring `encode_lj92`
  (hand-synced, like `plane_fixture`) or a shared `tests/support/ljpeg.rs`. Cover
  valid (each predictor, 8/12/16-bit, `Pt`, restart) + the hostile shapes. Never
  commit a corpus file.
- **No CI yaml change needed** — the fuzz-graph clippy + license gates already
  build the whole fuzz crate. A `fuzz smoke — ljpeg` job is optional (the smoke
  set is curated: only `opcode` + `warp_opcode`; `plane`/`ifd`/`develop`/`demosaic`
  have none). The millions-of-runs clean is the manual `cargo +nightly fuzz run
  ljpeg`.
- **Scope:** fuzz the single-component decoder only — no DNG container path, no
  multi-component, no bit-exactness (that is SPEC-037's oracle / STAGE-010's
  `dnglab` check). Don't touch the decoder's logic; this spec only exercises it.
- **Last STAGE-009 spec** — when it ships, STAGE-009 stage-ships (Prompt 1d).
- **§12 probe (done at design against `main`):** the real surfaces are
  `decode_into`/`parse_headers` (`src/ljpeg.rs`), the `plane.rs` fuzz target +
  `MAX_PIXELS` pattern, `examples/fuzz-seeds.rs`'s `#[path]`/local-builder seed
  mechanism, `fuzz/Cargo.toml`'s `[[bin]]` shape, and the `clippy-fuzz` CI job's
  plain-`cargo` build (the reason for the feature, `DEC-036`).

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
