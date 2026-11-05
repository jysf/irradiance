# SPEC-037 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-037-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (reconstruction + round-trip oracle spec + HANDOFF-087 + build prompt; no new DEC — governed by DEC-035/016/018)
- [x] **build** — PR #102, 2026-11-05 (HANDOFF-087); reconstruction (predictors 1–7 / §H.2.1 boundaries / point transform / restart reset) + `decode_into` + `Error::LjpegPlaneWrongLength` + the `encode_lj92` round-trip oracle & perturbed-predictor red-proof; four gates green; no new DEC. Cost: 271,544 tok (FLOOR — context occupancy, no `/cost`; see HANDOFF-087 handback)
- [x] **verify** — ⚠ PUNCH LIST → RESOLVED at ship 2026-11-05 (HANDOFF-088; cold review of PR #102 @ `e5a1062`). All four gates green (`just test` ljpeg 18/18, `lint-ci` clippy 0.1.98, `fmt-check`, `validate` 46) and all 8 acceptance criteria met; code is correct. Four adversarial mutations confirm the round-trip oracle bites (EXTEND + Pt go RED) and the suite is non-tautological (a shared-`predict` break is caught by the hand-computed test). ONE low-severity item blocks approval: **no non-tautological test pins `floor_div2`'s floor-toward-negative-infinity for predictors 5/6/7** (a truncate-toward-zero regression ships all-green — the sign trap the spec flagged). Add one hand-computed assertion with a negative odd `>>1` intermediate; no production change. Not `[x]`, no ship prompt. RESOLVED at ship: the orchestrator added `reconstruct_floors_negative_odd_predictor_intermediate` (predictor 5, Rb−Rc = −3 → `floor_div2` must floor to −2) and mutation-confirmed it goes RED on truncate-toward-zero; no production change; all four gates re-run green.
- [x] **ship** — 2026-11-05 (no ship prompt — verify was ⚠, shipped directly after resolving the one item). Costs RECOVERED from the cold transcripts — build 9,852,623 tok / $27.21 (HANDOFF-087), verify 9,560,701 tok / $22.59 (HANDOFF-088), design + ship un-metered; **totals 19,413,324 tok / $49.80**; complexity_actual M.

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
