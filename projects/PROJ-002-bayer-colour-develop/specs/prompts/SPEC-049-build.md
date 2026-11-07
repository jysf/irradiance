# SPEC-049 — build cycle prompt

You are the **implementer** for SPEC-049 (PROJ-002, STAGE-012), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (gh needs the sandbox disabled).

Read, in order: `projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-103-build-colour-develop-memory-fuse-and-f32-entry-point.md` (your context + the handback you must fill), `projects/PROJ-002-bayer-colour-develop/specs/SPEC-049-colour-develop-memory-fuse-and-f32-entry-point.md` (Acceptance + Failing Tests + Notes are the contract), `decisions/DEC-042-fuse-the-colour-develop-passes-and-add-an-f32-entry-point.md`, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change (DEC-042) — two parts in `src/color.rs`

**1. Fuse the passes.** Today: `normalize_mosaic` → a `normalized` mosaic; `demosaic_into` → a `camera_rgb` intermediate; `apply_matrix_into` reads `camera_rgb` → `dst`. Fuse to two passes:
- Pass 1: green at every pixel into a `w*h` green plane, **normalising each sample on read** (no `normalized` buffer).
- Pass 2: red/blue per pixel (reading the green plane + `src` normalised on read), assemble the camera-RGB triple in registers, apply the `f64` colour matrix, `quantize`, write linear-sRGB `u16` into `dst` (no `camera_rgb` buffer, no separate matrix pass).

Only the `w*h` green plane remains (~47 MB), down from ~187 MB.

**2. f32 entry point.** Add `pub fn develop_color_f32_into(sensor: &Sensor, src: &[u16], dst: &mut [f32]) -> Result<(), Error>` running the same fused pipeline, pass 2 writing `f32`. It must equal `to_linear_f32_into(develop_color_into(..))` **exactly** — and that divides the **quantised `u16`** by `65535.0`, so **quantise to `u16` first (identical to the integer path), then divide by `65535.0`**; do NOT divide the raw `f64` matrix result. Export it from `src/lib.rs`.

Keep the same integer ops in the same order so the `u16` output is **bit-identical**. Keep `develop_color_into`'s signature. No kernel/maths change, no `unsafe`, no dependency.

## Order of work (output-invariant, so prove it)

1. Write the failing tests first (in `tests/color_oracle.rs`): `fused_develop_color_is_byte_identical_to_the_reference` (fused `u16` output == the pre-fuse output, byte-for-byte, all four Bayer phases — the `dst_after == dst_before` proof) and `develop_color_f32_equals_to_linear_of_u16` (`develop_color_f32_into(..)` == `to_linear_f32_into(develop_color_into(..))` exactly). The existing colour goldens must stay green.
2. Implement; make them pass; keep the goldens green.
3. Measure: record a before/after **peak-RSS** for a representative develop in `docs/provenance-ledger.md` (target ~140 MB lower at 23 MP), and note develop wall-time before/after (it must not regress materially — the normalise-on-read recompute is the risk).
4. `just test` green (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`, `just validate`, `just cost-audit` green; no panic; determinism; no dependency.

## Hand back

Fill HANDOFF-103's `handback:` (REAL `tokens_total` from `/cost`) and the `## Handback` section, then open a PR against `main` (SPEC-049 only) referencing HANDOFF-103, SPEC-049, STAGE-012, PROJ-002, DEC-042, DEC-038. Commit message BODY plain prose (reword if the push ruleset rejects code tokens), ending with:
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back the PR number, the measured peak-RSS before/after (and develop time), and the true token cost.
