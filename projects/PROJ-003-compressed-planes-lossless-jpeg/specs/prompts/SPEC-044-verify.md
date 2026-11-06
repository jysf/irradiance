# SPEC-044 VERIFY — prompt for the verifier (cycle: verify)

You're the verifier for a spec-driven workflow. Cycle: **verify**. SPEC-044
(PROJ-003 / STAGE-011) — the bit-exact `dnglab` corpus oracle for both held DNG
`Compression 7` files, plus the DEC-039 APPn/COM header-skip. You are **not** the
agent who built this; review it cold. The build is PR #124 (branch
`claude/spec-044-build-ab78fc`); cut your verify branch from it so the verify PR is a
clean superset. Your handback is **HANDOFF-096**.

Read, in order:

1. `/AGENTS.md`.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md`
   — `## Inputs` carries the design §12 probe; `## Acceptance Criteria` /
   `## Failing Tests` are what you check.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-095-build-bit-exact-corpus-oracle-both-held-compression-7.md`
   — the build handback (what the implementer claims).
4. `DEC-039` first (APPn/COM skip; mapping confirmed), then `DEC-037`, `DEC-030`,
   `DEC-003`, `DEC-035`, `DEC-016`, `DEC-018`.
5. The stage `STAGE-011-...md`, the brief, `guidance/toolchain-brief.md`
   (`just lint-ci` is the authoritative local clippy).
6. The code: `src/ljpeg.rs` (the `parse_headers` APPn/COM arm + tier-A fixtures),
   `tests/plane_oracle.rs` (`DECODABLE_COMPRESSED` oracle + `ljpeg` red-proof),
   `tests/develop_oracle.rs` (the develop smoke).

The verify contract — behavior-checked, not shape-checked:

- Both held files decode to a plane whose native-LE MD5 **equals** the pinned
  `raw_checksum` on the real corpus (Typ 246 `c72e7d9579e2adaef9c6247841b8ac1d`, K3III
  `98bae1339c25eb6e04fbac6b5f643189`), with no SKIP lines for them.
- The APPn/COM skip is real: the arm fires on a tier-A frame; a truncated/garbage APPn
  length is a typed `Error`; the `Nf=1` path and the Typ 246 decode are byte-for-byte
  unchanged (Typ 246 has no APPn).
- The red-proof goes RED on both real files and the control stays green; it mutates the
  `ljpeg` decode, not `src/plane.rs`.
- The three-list partition still covers the manifest; Compression-7 entries are
  compressed-but-decodable.
- The develop smoke runs a held file through `develop_into` with correct dimensions,
  deterministically.
- Decision drift (`just decisions-audit --changed`), constraint violations, non-trivial
  choices missing a DEC, honest build reflection.

Output exactly ONE verdict: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

Out of scope (do NOT punch-list): the provenance-ledger entry, the macOS/Linux
determinism assertion, the broader container→decoder hostile-strip hardening, and the
review §2.1 / §2.3 / §2.4 items — all SPEC-045; Bayer/CFA colour multi-component —
DEC-037.

Gates: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`,
then the targeted suites — `cargo test --lib ljpeg::`, `cargo test --test plane_oracle`,
`cargo test --test develop_oracle` — plus `just lint-ci`, `just fmt-check`,
`just validate`, `just decisions-audit --changed`. Confirm PR #124's CI is green on its
exact head rather than running a blind full `just test` (CI has no corpus; the
real-file oracle runs only locally).

Bookkeeping: fill HANDOFF-096 with a real `tokens_total` (or an explicit FLOOR with a
note when the interface has no `/cost`). On APPROVED: set statuses completed, run
`just advance-cycle SPEC-044 ship`, mark verify `[x]` in the timeline, and open a PR
against `main` referencing PROJ-003, STAGE-011, SPEC-044, HANDOFF-096, DEC-039, DEC-037.
On PUNCH LIST / REJECTED: record findings precisely, set statuses, set `task.blocked` if
rejected, and message the orchestrator. Keep commit-message bodies plain prose.
