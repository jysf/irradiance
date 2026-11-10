# SPEC-050 — build cycle prompt

You are the **implementer** for SPEC-050 (PROJ-002, STAGE-012), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (gh needs the sandbox disabled).

Read, in order: `projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-105-build-colour-develop-crop-to-active-area-and-geometry-parity.md` (your context + the handback you must fill), `projects/PROJ-002-bayer-colour-develop/specs/SPEC-050-colour-develop-crop-to-active-area-and-geometry-parity.md` (Acceptance + Failing Tests + Notes are the contract), `decisions/DEC-043-crop-the-colour-path-to-full-geometry-parity-with-develop-into.md`, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change (DEC-043) — crop at the SINK, don't touch the kernel

Today `develop_color_into` / `develop_color_f32_into` (`src/color.rs`) develop the **whole plane** and the sink writes `dst_triples[pixel]`, so `dst` is `width·height·3` and carries the masked border, with neither `DefaultCrop` nor `Orientation`. The colours are already correct (SPEC-047's ActiveArea-origin offset is inside `develop_color_fused`). Make the output match `develop_into`'s geometry:

**1. Leave `develop_color_fused` (`src/demosaic.rs`) byte-for-byte.** It still develops every plane pixel with the same neighbours in the same order, so every developed value stays identical to SPEC-047. The crop is a sink concern.

**2. Crop + orient in the sink.** Resolve geometry once (`resolve_geometry` / `output_dimensions`, `src/develop.rs`). In the sink closure, map each plane pixel `(x, y)` (recoverable from `pixel = y·width + x`): if it lies inside the `ActiveArea` **and** inside the `DefaultCrop` rectangle, scatter its matrixed triple to `dst` at the output position given by the `DefaultCrop` offset + the **inverse of `crop_source_coords`** for the orientation; otherwise drop it. Size `dst` to `output_dimensions(sensor)·3` and fix the length checks + `Error::RgbBufferWrongLength` accordingly. Document the new contract in the doc comments.

**3. Add the inverse-orientation helper** `output_coords_from_crop(orientation, crop_x, crop_y, crop_w, crop_h) -> (out_x, out_y)` in `src/develop.rs` next to `crop_source_coords`, and pin it with a round-trip unit test over all eight orientations. Orientations 1–4 keep `crop_w×crop_h`; 5–8 transpose to `crop_h×crop_w`.

**4. f32 in lockstep.** `develop_color_f32_into` scatters the **quantised `u16`** then divides by `65535.0` (not the raw `f64`), so it stays exactly `to_linear_f32_into(develop_color_into(..))` at the new dims.

Do NOT introduce an active-area or full-plane RGB buffer (that re-adds the ~140 MB SPEC-049 removed — scatter straight into `dst`). Do NOT re-scope the kernel to the active window / active-edge reflect (DEC-043 rejects it: it would change near-edge values). No kernel/maths change, no `unsafe`, no dependency.

## Order of work (write the red tests first)

1. Failing tests first (each must fail against today's whole-plane output):
   - `colour_output_has_output_dimensions_geometry` — a `fixture_sensor` with a non-trivial `ActiveArea` + `DefaultCrop` + `Orientation`; `develop_color_into` with `dst` = `output_dimensions·3` succeeds and the border is gone.
   - `colour_geometry_matches_the_mono_path` — colour output dims equal `output_dimensions(sensor)`.
   - `colour_crop_pixels_equal_the_whole_plane_develop_at_their_source` — the kept pixels are byte-identical to today's whole-plane `develop_color_into` at their `(DefaultCrop + inverse-orient)` source (the crop moves the frame, not the colours; SPEC-047 grey-stays-grey still holds).
   - `output_coords_from_crop_is_the_inverse_of_crop_source_coords` — the round-trip, all eight orientations.
   - `develop_color_f32_still_equals_to_linear_at_new_dims`.
2. Implement; make them pass. Regenerate the colour goldens for the new geometry (reason in the commit); a trivial-geometry fixture (full-plane ActiveArea, no crop, orientation 1) must stay byte-identical.
3. Bump `DEVELOP_VERSION` 3 → 4 (`src/lib.rs`). Record a peak-RSS before/after for a representative develop in `docs/provenance-ledger.md` — it must **not regress** from SPEC-049 (no new buffer; `dst` shrinks).
4. `just test` green (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`, `just validate`, `just cost-audit` green; no panic; determinism; no dependency.

## Hand back

Fill HANDOFF-105's `handback:` (REAL `tokens_total` from `/cost`) and the `## Handback` section, then open a PR against `main` (SPEC-050 only) referencing HANDOFF-105, SPEC-050, STAGE-012, PROJ-002, DEC-043, DEC-040, DEC-019, DEC-038. Commit message BODY plain prose (reword if the push ruleset rejects code tokens like `#[...]`, `->`, `--flag`), ending with:
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back the PR number, confirmation the kept pixels are byte-identical to the whole-plane develop at their source, the measured peak-RSS before/after, and the true token cost.
