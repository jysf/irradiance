# SPEC-045 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-045 (PROJ-003 /
STAGE-011) — the **binding capstone**: a DNG Compression-7 provenance-ledger row, a
plane-level macOS/Linux determinism pin, the lossless-JPEG header-hardening the external
0.2.0 review flagged (`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces), and continuous
fuzzing of the **public** Compression-7 route. This is the **last STAGE-011 spec** — when it
ships, STAGE-011 and PROJ-003 close.

Read, in order, before writing code:

1. `AGENTS.md`.
2. `projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-097-build-binding-capstone-provenance-determinism-hardening.md`
   — your handoff; follow its "Context the Receiving Agent Needs" exactly. It lists the four
   gaps to close and what is already covered.
3. `projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` + the stage
   `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
4. `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-045-binding-capstone-a-provenance-ledger-entry-for-the.md`
   — the spec; `## Inputs` has the §12 probe + exact line refs, `## Outputs` /
   `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
5. `DEC-036` (the fuzzing feature/hook — the Validation decision is KEEP it, see the spec Notes),
   `DEC-035` (clean-room T.81), `DEC-003` (corpus never committed; a synthetic seed is fine),
   `DEC-018` (u16 modular arithmetic); the review triage `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`.
6. `/guidance/constraints.yaml` (`no-panics-on-untrusted-input`,
   `no-new-top-level-deps-without-decision`); `guidance/toolchain-brief.md`; and the code:
   `src/ljpeg.rs` (`parse_scan_header`, `parse_dht`, `find_table`), `src/plane.rs` (the SPEC-043
   route), `tests/support/tiff.rs` (the DNG builder to extend with `compression7_dng`),
   `tests/plane_unpack.rs`, `fuzz/fuzz_targets/{plane,develop,demosaic,ljpeg}.rs`,
   `src/lib.rs` `__fuzz_ljpeg`, `docs/provenance-ledger.md`.

Before coding, mark the build cycle `[~]` in the SPEC-045 timeline. If the spec is ambiguous
or needs architect judgment, mark it `[?]` with a one-line reason and message the orchestrator
— do not guess.

Implement (per `## Outputs`):
- `docs/provenance-ledger.md`: add the DNG Compression-7 **binding** row (`src/plane.rs` route
  → `ljpeg` per SPEC-043 / SPEC-044 / DEC-039; provenance class 1 — public TIFF/DNG + T.81
  specs, `dnglab` run-as-tool only), and correct the `src/plane.rs` row's stale "compressed
  files … never decoded" note.
- `src/ljpeg.rs`: reject `Pt >= P` (typed `Error`); reject `Tc != 0` / `Th > 3` in `parse_dht`
  (typed `Error`); make a later same-`Th` DHT win (`find_table` returns the last match, or
  `parse_dht` overwrites). No reconstruction/predictor/de-interleave change. Prefer reusing an
  existing `Ljpeg*` variant; add at most one additive variant.
- `tests/support/tiff.rs`: add `compression7_dng(order, …)` wrapping a SOF-3 strip from the
  `cfg(test)`-reexported `encode_lj92` in a `Compression = 7` single-strip DNG.
- `fuzz/`: drop the `require_uncompressed().is_err()` early-return in
  `fuzz/fuzz_targets/{plane,develop,demosaic}.rs`; add a synthetic Compression-7 SOF-3 seed to
  `fuzz/seeds/plane/`. **Keep** `__fuzz_ljpeg` + the `fuzzing` feature + `fuzz/fuzz_targets/ljpeg.rs`
  (DEC-036 Validation — record the keep-reasoning in the handback).
- Tests: make every SPEC-045 `## Failing Test` pass (the header gaps, later-DHT, a truncated
  Compression-7 strip through the public `plane::unpack_into`, a tiled rejection, a plane-level
  two-decode byte-identity, and a pin/cite of the existing SPEC-046 restart coverage).

Do NOT: retire the fuzz hook; change the reconstruction algorithm, routing, or oracle; add
`#[non_exhaustive]`, the opcode bound, `LinearizationTable` rejection, or `tag_name` in Display
(all explicit Non-Goals).

When done:
1. Fill `HANDOFF-097`'s `handback:` block + `## Handback` with a **real** `tokens_total` (run
   `/cost`; if your interface has no meter — the Claude Code desktop Code tab does not — report
   the `ccd_session_mgmt get_usage` occupancy as an explicit **FLOOR** with a note, as
   HANDOFF-089..096 did; do not invent). Reflection required, plus the DEC-036-keep reasoning.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-045 verify`
4. Open a PR from `claude/spec-045-build-<suffix>` against `main`, referencing PROJ-003,
   STAGE-011, SPEC-045, HANDOFF-097, DEC-036, DEC-035.
5. Mark build `[x]` in the timeline with the PR number, cost, and date.

Running the gates (the tier-B lesson): set
`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and run the
TARGETED suites — `cargo test --lib ljpeg::`, `cargo test --test plane_unpack`,
`cargo test --test plane_oracle`, `cargo test --test develop_oracle` — rather than a blind full
`just test`. Confirm `just fmt-check` / `just lint-ci` / `just validate` green locally; a short
`cargo +nightly fuzz run plane` on the new Compression-7 seed shows the public route is
panic-free. Lean on the PR's CI for the broad tier-A / lint sweep.
