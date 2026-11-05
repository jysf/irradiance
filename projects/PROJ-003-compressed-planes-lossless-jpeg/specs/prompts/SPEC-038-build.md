# SPEC-038 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-038 —
a **libFuzzer target over the shipped single-component SOF-3 decoder**, proving it
is panic-free on hostile input. Last spec in STAGE-009.

Read files in order, before writing code:

1. `/AGENTS.md` — conventions you must follow.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-089-build-sof-3-decoder-fuzz-truncation-hardening.md`
   — your handoff. Follow its "Context the Receiving Agent Needs" exactly.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` + the stage
   `STAGE-009-sof-3-lossless-jpeg-core-decoder.md`.
4. `DEC-036` (the `fuzzing`-feature exposure — **read it first; it is the crux**)
   and `DEC-035` (clean-room / patent-clean).
5. `/guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
   `no-new-top-level-deps-without-decision`.
6. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening.md`
   — the spec; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
7. `./guidance/toolchain-brief.md`; `fuzz/fuzz_targets/plane.rs` (the analog +
   `MAX_PIXELS`); `examples/fuzz-seeds.rs` (the seed writer); the shipped
   `src/ljpeg.rs` (`decode_into`, `encode_lj92`).

Before coding, mark the build cycle `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening-timeline.md`.
If something needs architect judgment or the spec is ambiguous, change it to `[?]`
with a one-line reason and **stop** — ask; don't guess.

Implement (per `## Outputs`):

- **Exposure via a `fuzzing` cargo FEATURE, not `cfg(fuzzing)`** (`DEC-036` — the
  `clippy — fuzz graph` gate builds `fuzz/Cargo.toml` with plain `cargo clippy`, so
  `cfg(fuzzing)` would break it). Add `[features] fuzzing = []` to `Cargo.toml`;
  add `#[cfg(feature = "fuzzing")] #[doc(hidden)] pub fn __fuzz_ljpeg(data:
  &[u8])` to `src/lib.rs` that bounds `lines × samples_per_line` (checked
  arithmetic, a `MAX_PIXELS` cap like `plane.rs`), allocates a capped buffer,
  calls `ljpeg::decode_into`, and drops the `Result`. It compiles under
  `--all-features`, so keep it five-lint clean (no `unwrap`/`expect`/indexing/
  `panic`; checked/wrapping arithmetic).
- `fuzz/Cargo.toml`: `[dependencies.irradiance]` gains `features = ["fuzzing"]`;
  add `[[bin]] name = "ljpeg" path = "fuzz_targets/ljpeg.rs" test=false doc=false
  bench=false`. `fuzz/fuzz_targets/ljpeg.rs`: `#![no_main]` +
  `fuzz_target!(|data: &[u8]| { irradiance::__fuzz_ljpeg(data); })` + a header doc.
- `examples/fuzz-seeds.rs`: a `ljpeg_seeds()` writer → `fuzz/seeds/ljpeg/`, from a
  local SOF-3 builder mirroring `encode_lj92` (hand-synced like `plane_fixture`,
  or a shared `tests/support/ljpeg.rs`): valid scans (predictors, 8/12/16-bit,
  `Pt`, restart) + hostile shapes (truncated header, truncated scan, oversized
  DHT, `Nf≠1`, bad marker). Commit the seeds (own work, `DEC-003` — no corpus file).
- Panic-free **unit tests** in `src/ljpeg.rs mod tests` (reach `decode_into`
  directly — do NOT route through the feature entry): replay every
  `fuzz/seeds/ljpeg/` seed asserting no panic + non-empty corpus; a table of
  hostile inputs each a typed `Error`, never a panic.
- `ljpeg` stays `pub(crate)` — a default-feature `cargo build`/`cargo doc` exposes
  no `ljpeg` symbol and no `__fuzz_ljpeg`. No new runtime dep. No CI yaml change
  needed (the fuzz-graph clippy + license-fuzz gates cover the new target).

When done:

1. Fill the handoff's `handback:` block + `## Handback` section with a **real**
   `tokens_total` (run `/cost`; if your interface has no meter, report the
   `ccd_session_mgmt get_usage` context-occupancy as an explicit FLOOR with a note,
   as HANDOFF-085/086/087/088 did — do not invent a cumulative). Reflection required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-038 verify`
4. Open a PR from `claude/spec-038-build-<suffix>` against `main`, referencing
   `PROJ-003`, `STAGE-009`, `SPEC-038`, `HANDOFF-089`, `DEC-036`/`DEC-035`.
5. Mark build `[x]` in the timeline with the PR number, cost, and date.

Gates before the PR — all green: `just test` · `just lint-ci` · `just fmt-check` ·
`just validate` · `just lint-fuzz` · `just deny-fuzz`. (Optionally smoke the
target: `cargo +nightly fuzz run ljpeg fuzz/corpus/ljpeg fuzz/seeds/ljpeg --
-max_total_time=60`.)
