# SPEC-022 — build dispatch

Build cycle for **SPEC-022: develop warp branch must develop the fixed plane
not raw src** (STAGE-005, PROJ-001). Delegated via **HANDOFF-054**.

Read first, in order:
1. `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-054-build-develop-warp-branch-must-develop-the-fixed-plane-not-raw-src.md`
2. The spec — its **Inputs** (fixture recipe), **AC1–AC4**, and **Notes for the Implementer**.
3. `guidance/toolchain-brief.md`.

This is a **test-only** spec — the exact warp-branch mirror of SPEC-017's SB-2
assertion. Deliver two tier-A tests in `tests/develop.rs` and discharge the
red-proof. Four traps that sink this build:

1. **Do NOT change `src/develop.rs`.** The warp branch already develops
   `effective_src`. The committed diff touches `tests/develop.rs` only. The sole
   src edit is the throwaway red-proof mutation, fully reverted.
2. **Reuse the helpers** (`single_fix_bad_pixels_constant_list`,
   `single_warp_rectilinear_list`, `minimal_sensor`) — don't hand-encode opcode
   bytes. Keep the bad pixel at the optical centre `(2,2)` of the 5×5 plane so
   `developed[12]` is exactly `1000` (fixed) / `0` (raw).
3. **Red-proof in a scratchpad COPY**, never the working tree. Record the
   honest→severed→reverted md5 triple for `src/develop.rs`; show the new test
   RED and AC9 GREEN under the sever. Use `cargo test --no-fail-fast` for the
   suite total — plain `cargo test` fail-stops at the develop binary and skips
   ~99 tests. Read exit codes directly (zsh: `$pipestatus`, not bash
   `PIPESTATUS`).
4. **Fill the handback** (`handback:` block + `## Handback`) with a REAL
   `tokens_total` before reporting done, open a PR against `main`, and confirm
   CI green by observation.
