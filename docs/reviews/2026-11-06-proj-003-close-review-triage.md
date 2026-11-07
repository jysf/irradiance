# PROJ-003 close ultrareview — findings triage

- **Reviewed:** the PROJ-003 decoder surface, via the `review/proj-003-code` scaffold branch (code-only diff `60f232f..review/proj-003-code`, 29 files / 5,116 lines).
- **Triaged by:** the orchestrator session (claude-opus-4-8), 2026-11-06, against current `main` (tip after the PROJ-003 close, PR #132 / #133).
- **Outcome:** 9 findings (3 "normal", 6 "nits"). **4 are false positives** created by the scaffold; **5 are real but latent** — none affects the shipped corpus (both held Compression-7 files decode bit-exact vs `dnglab`, and the two unexercised paths below have no held file).

## The scaffold caveat (why 4 findings are false positives)

The review ran against a code-only branch that carried `src/ljpeg.rs`, `src/plane.rs`, `src/lib.rs`, the tests, the fuzz targets, and `examples/fuzz-seeds.rs` — but **not** `Cargo.toml`, `fuzz/Cargo.toml`, `docs/`, or `src/bin/irr.rs`. The reviewers therefore read the *baseline* (pre-PROJ-003) versions of those omitted files and flagged things already fixed on `main`. Lesson recorded as `review-scaffold-must-carry-the-whole-code-tree`.

## Verdict table

| # | Sev | Finding | Verdict | Evidence on `main` |
|---|-----|---------|---------|--------------------|
| 1 | normal | Restart-interval prediction only resets the first sample; the "first line uses Ra" rule is gated on global row 0 (`ljpeg.rs:1056`, two-component `:1134`). The encoder and reference decoder share the rule, so the round-trip can't catch it. | **REAL (latent)** | Confirmed: `reconstruct:1054-1067`, `encode_lj92:1859`, `reference_decode:2077` all use `r == 0`. **No held file has a `DRI`**, so the restart path never runs on real data; row-0 behavior is bit-exact vs dnglab. The live defect is the **oracle tautology**; T.81 restart conformance is an open question. |
| 2 | normal | `fuzzing` feature never declared (`Cargo.toml` has no `[features]`). | **FALSE POSITIVE** | `Cargo.toml:53` — `[features]` / `fuzzing = []`. Scaffold omitted `Cargo.toml`. |
| 3 | normal | `ljpeg` fuzz target not registered. | **FALSE POSITIVE** | `fuzz/Cargo.toml:26` `features = ["fuzzing"]`, `:70` `[[bin]] name = "ljpeg"`. Scaffold omitted `fuzz/Cargo.toml`. The 3.1M-run coverage is real. |
| 4 | nit | Sampling factors ignored — `Hi`/`Vi` read and discarded (`ljpeg.rs:468`); `H`/`V ≠ 1` silently garbles. | **REAL (latent)** | Confirmed and documented as intentional (held files are `H=V=1`, `DEC-037`). Defensive gap: should return a typed error, not garble. No held file subsamples. |
| 5 | nit | No provenance-ledger row. | **FALSE POSITIVE** | `docs/provenance-ledger.md` has three rows (SPEC-036 front-end, SPEC-037 reconstruction, SPEC-043/046/044/045 binding). Scaffold omitted `docs/`. |
| 6 | nit | Stale error text — `Ljpeg*` messages say "Nf = 1 only". | **REAL** | `lib.rs:973` message still says "only single-component (Nf = 1) scans are supported"; `LjpegPlaneWrongLength` doc (`:711`) cites the single-component length. Nf=2 accepted since SPEC-046. |
| 7 | nit | `irr info` prints "unpackable no" for Compression 7 via `require_uncompressed()`. | **FALSE POSITIVE** | `irr.rs:227-236` matches `sensor.compression` and prints "unpackable yes (Compression 7 …)". SPEC-043 fixed it; scaffold omitted `irr.rs`. |
| 8 | nit | Tier-B test accepts the old rejection; comment stale (`plane_unpack.rs:541`). | **REAL** | `plane_unpack.rs:544-555` accepts `Err(LjpegUnsupportedComponentCount)` as a pass with a pre-SPEC-046 comment; both held files now decode `Ok(())`, so that arm is dead and too permissive. The real bit-exact check lives in `plane_oracle.rs`. |
| 9 | nit | Duplicated strip-layout validation (`plane.rs`) with divergent errors; a second SOF-3 encoder in `examples/fuzz-seeds.rs`. | **REAL (maintainability)** | Plausible DRY/consistency smell; not correctness. |

## The 5 real findings → the 0.2.0 ljpeg-hardening scope

All latent; none blocks the shipped PROJ-003. Suggested home: a small ljpeg-conformance/hardening spec in the 0.2.0 wave.

1. **Restart-boundary conformance + a non-shared oracle (highest value).** Settle the T.81 §H.1.2.1 restart-interval prediction rule against the standard text, and verify it with an **independent** check — `dnglab` on a synthetic restart-bearing SOF-3 file, or hand-computed restart fixtures — so the oracle is not tautological. (This is the N=2 instance of `shared-path-oracle-tautology`: the encoder, the production decoder, and the "independent" `reference_decode` all share the `r == 0` rule.)
2. **Reject `Hi`/`Vi ≠ 1`** with a typed error in `parse_frame_header` rather than silently garbling a subsampled frame.
3. **Refresh the `Ljpeg*` error text** — `LjpegUnsupportedComponentCount` message and `LjpegPlaneWrongLength` doc, both still single-component-only.
4. **Tighten the stale tier-B route test** (`plane_unpack.rs`) to require `Ok(())` for the held files and drop the dead `LjpegUnsupportedComponentCount` arm / stale comment.
5. **De-duplicate** the strip-layout validation across the uncompressed and JPEG paths (reconcile the divergent error types), and the second SOF-3 encoder in `examples/fuzz-seeds.rs`.

Plus, raised during triage:

6. **Examine splitting `src/ljpeg.rs`** (maintainability, not correctness). The file is 3,283 lines but ~1,957 (60%) is the inline `#[cfg(test)] mod tests`; the production decoder is ~1,327 lines. Lowest-risk first move: extract the test harness to its own file (checking the `pub(crate)` encoders' cross-file use first). Optional deeper split into a `ljpeg/` submodule (`headers`/`huffman`/`scan`/`reconstruct`) — internal-only since `ljpeg` is `pub(crate)` — gated on a no-behavior-change proof (the bit-exact oracle + round-trip).

## Recorded signals

- `shared-path-oracle-tautology` — bumped to **N=2** (bar met) with the restart instance; codify at the 0.2.0 hardening stage-close.
- `review-scaffold-must-carry-the-whole-code-tree` — new lesson (N=1): a cherry-picked review scaffold reads omitted-but-related files at their pre-change state, producing false positives (4 of 9 here).
- `unexercised-sof3-paths-garble-silently` — new risk: restart intervals and `Hi`/`Vi` subsampling are parsed-but-unvalidated and corpus-unexercised; a conforming such file would silently mis-decode. Mitigated by findings 1–2 above.
