# SPEC-021 — build dispatch

Build cycle for **SPEC-021: narrow SPEC-020's oracle scope excluding warp-bearing
pixels** (STAGE-005, PROJ-001). Delegated via **HANDOFF-058**.

Read first, in order:
1. `projects/PROJ-001-monochrome-dng-develop/specs/prompts/SPEC-021-build.md` (this).
2. `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-058-build-narrow-spec-020-oracle-scope-excluding-warp-bearing-pixels.md`.
3. The spec (SPEC-021) — Inputs (the file/line recipe), AC1–AC6, Notes (the key numbers).
4. `guidance/toolchain-brief.md`.

This is a **documentation + test-reframe** spec. Deliverables: a new DEC-026
amending DEC-005, a `docs/oracle-contract.md` carve-out, and reframing the two
`#[ignore]`'d SPEC-018 tests in `tests/warp.rs`. Four traps that sink this build:

1. **0 `src/` LOGIC lines.** The warp render is correct — AC4/AC10 prove it. The
   only `src/` edits are comment-only cross-ref fixes (`src/warp.rs:~72`,
   `src/lib.rs`). Assert `git diff` shows 0 non-comment `src/` lines (AC4).
2. **Reframe the `#[ignore]`s, don't delete them.** AC8 (`score >= 85`) → a
   PASSING characterization test on Orientation-1 frames asserting the
   warp-bearing score is *below* the no-warp baseline (`-60.169 < +83.145`,
   SPEC-015 AC5(a) shape). The Orientation-6 frame (L1026016) → an EXPLICIT
   documented skip with a loud reason — not the latent `assert_eq!` panic at
   `tests/warp.rs:~583`. Keep the measured numbers in rustdoc. Repo-wide
   `#[ignore]` drops 3 → 1.
3. **Do NOT add a red-proof.** `oracle-must-be-shown-red` is already discharged
   by the shipped, CI-visible AC10 — cite it. The reframed tests are
   characterizations.
4. **Amend DEC-005 via a NEW DEC-026** (DEC-022 shape: quote verbatim + "why
   amend not revert"); `just decisions-index`; keep `just decisions-audit` clean.
   Cite DEC-024 Finding 2 for the warp evidence; record the Orientation leg from
   SPEC-018/FU-6 + SPEC-017/FU-2 (DEC-024 does not have it).

Fill the handback with a REAL `tokens_total`; open a PR against `main`; confirm
CI green by observation. zsh: `$pipestatus`, not `PIPESTATUS`.
