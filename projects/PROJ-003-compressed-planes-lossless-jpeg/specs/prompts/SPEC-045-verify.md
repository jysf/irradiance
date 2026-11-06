# SPEC-045 VERIFY — prompt for the verifier (cycle: verify)

You're the verifier for a spec-driven workflow. Cycle: **verify**. SPEC-045
(PROJ-003 / STAGE-011) — the binding **capstone**: the DNG Compression-7
**binding** provenance-ledger row, a plane-level macOS/Linux determinism pin,
the lossless-JPEG header hardening the external 0.2.0 review flagged
(`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces), and continuous fuzzing of
the **public** Compression-7 route. This is the **last** STAGE-011 spec — with
it, STAGE-011 and PROJ-003 close. You are **not** the agent who built this;
review it cold. The build is already merged to `main` (PR #129, squash commit
`a2d6376`), so your verify PR is a thin superset — the handback, this prompt,
and the timeline update — cut from the current `main` tip. Your handback is
**HANDOFF-098**.

Read, in order:

1. `/AGENTS.md`.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-045-binding-capstone-a-provenance-ledger-entry-for-the.md`
   — `## Inputs` carries the design §12 probe (the four genuine gaps + the three
   already-covered legs); `## Outputs` / `## Acceptance Criteria` /
   `## Failing Tests` / `## Notes` are the contract.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-097-build-binding-capstone-provenance-determinism-hardening.md`
   — the build handback (what the implementer claims, incl. the DEC-036
   Validation disposition and the four deviations).
4. `DEC-036` first (keep the `fuzzing` feature + `__fuzz_ljpeg` hook, add
   public-path coverage — no new DEC), then `DEC-035` (clean-room T.81),
   `DEC-003` (corpus never committed; synthetic own-work seed is fine),
   `DEC-018` (u16 modular arithmetic), `DEC-037` / `DEC-039`.
5. The stage `STAGE-011-...md`, the brief, `guidance/toolchain-brief.md`
   (`just lint-ci` is the authoritative local clippy), and the external review
   `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` (§2.1 / §2.4 folded
   here).
6. The code: `src/ljpeg.rs` (the three new guards in `parse_headers` /
   `parse_dht` / `find_table`, the new `Error::LjpegInvalidPointTransform`, and
   the five new tier-A tests), `tests/plane_unpack.rs` (the public-route
   truncated/hostile + tiled tests and the synthetic round-trip),
   `tests/plane_oracle.rs` (the plane-level two-decode determinism pin),
   `tests/support/tiff.rs` (`lj92_strip` / `compression7_dng` own-work SOF-3
   builder), the three `fuzz/fuzz_targets/{plane,develop,demosaic}.rs`, the
   three `fuzz/seeds/*/compression7-sof3.tiff`, `src/lib.rs`
   (`__fuzz_ljpeg` kept), `docs/provenance-ledger.md`.

The verify contract — behavior-checked, not shape-checked:

- **Pt < P** is a typed `Error::LjpegInvalidPointTransform`, never a panic,
  checked in `parse_headers` where both `P` and `Pt` are known; `Pt = P - 1`
  still parses; the shipped point-transform round-trip is unchanged (tier A).
- **Tc != 0 or Th > 3** is a typed `Error::LjpegInvalidHuffmanTable` per table
  (including mid-segment), never a panic; destination 3 still parses (tier A).
- **A later DHT with the same `Th` replaces the earlier** — a stream whose valid
  table follows a bogus one decodes with the later; red-proven against
  first-wins; single-table Nf=1 and shared-`Td` Nf=2 decodes unchanged (tier A).
- **Plane-level determinism** — a held Compression-7 file decodes to a
  byte-identical `u16` plane across two independent `unpack_into` runs (second
  buffer pre-filled `0xFFFF`), equal to the pinned `dnglab` digest; corpus-gated,
  skips loudly; the integer-only / cross-platform basis is documented.
- **Public-route hardening** — every proper prefix of a synthetic SOF-3 strip,
  plus `Pt`/`Tc`/`Th` corruptions, is a typed `Error` through the public
  `plane::unpack_into`; a tiled Compression-7 plane is a clean
  `UnsupportedStripLayout` (tier A, synthetic container).
- **Public-route fuzzing** — the `require_uncompressed` early-return is gone from
  all three fuzz targets, a `compression7-sof3.tiff` seed is present in all three
  seed dirs, and the direct `__fuzz_ljpeg` hook + `fuzzing` feature are retained
  (DEC-036); a short run on each target is panic-free.
- **Provenance** — a DNG Compression-7 **binding** row is added, and the
  `src/plane.rs` row's "compressed files … never decoded" note is corrected to
  the SPEC-043 routing. `just validate` stays green.
- Decision drift (`just decisions-audit --changed a2d6376^`), constraint
  violations, non-trivial choices missing a DEC, honest build reflection.

Output exactly ONE verdict: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

Out of scope (do NOT punch-list): `#[non_exhaustive]` on the public enums
(SPEC-042), the opcode `MAX_TAG_VALUES` bound (PROJ-001), `LinearizationTable`
rejection (cross-cutting, no held file), `tag_name` in `Error` `Display`
(cosmetic), retiring the `__fuzz_ljpeg` hook (kept — DEC-036); Bayer/CFA colour
multi-component and tiled decode (DEC-037).

Gates: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`,
then `just test` (bakes in `--all-features` — run `just test`, never
`just test --all-features`), `just lint-ci`, `just fmt-check`, `just validate`,
`just decisions-audit --changed a2d6376^`, and bounded fuzz smokes on all three
targets (`just fuzz-plane 30`, `just fuzz-develop 30`, and `demosaic` directly —
always bound with `-max_total_time=` / `-runs=`, never an unbounded run).
Confirm PR #129's CI is green on its exact head for the broad sweep CI runs that
have no corpus (MSRV, cargo-deny both graphs, the lint/fuzz policy red-proofs,
the panic-free no-`#[allow]` escape job).

Bookkeeping: fill HANDOFF-098 with a real `tokens_total` (or an explicit FLOOR
with a note when the interface has no `/cost`), and append a verify
`cost.sessions` entry to the spec. On APPROVED: mark build `[x]` and verify
`[x]` in the timeline with the approved SHA and the HANDOFF-098 / prompt refs,
run `just advance-cycle SPEC-045 ship`, and open a PR against `main` referencing
PROJ-003, STAGE-011, SPEC-045, HANDOFF-098, DEC-036, DEC-035. Because SPEC-045 is
the last STAGE-011 spec, note that Prompt 1d (Stage Ship) and Prompt 1e (Project
Ship) come next. On PUNCH LIST / REJECTED: record findings precisely, set
statuses, set `task.blocked` if rejected, and message the orchestrator. Keep
commit-message bodies plain prose (the push ruleset rejects code tokens).
