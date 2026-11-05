# SPEC-037 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-037 —
**lossless reconstruction** (predictors, point transform, restart) on top of
SPEC-036's shipped SOF-3 entropy front-end.

Read files in order, before writing code:

1. `/AGENTS.md` — conventions you must follow.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-087-build-lossless-reconstruction-predictors-point-transform-restart.md`
   — your handoff. Follow its "Context the Receiving Agent Needs" exactly.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` — project context.
4. `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
   — the stage.
5. The decisions in the handoff: `DEC-035` (patent/clean-room + provenance),
   `DEC-016` (caller-owned buffer), `DEC-018` (u16 modular arithmetic).
6. `/guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
   `oracle-must-be-shown-red`, `provenance-recorded-per-algorithm`,
   `test-before-implementation`.
7. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart.md`
   — the spec; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
8. `./guidance/toolchain-brief.md` — this repo's test/lint/run quirks.
9. The shipped `src/ljpeg.rs` — SPEC-036's `Frame`, `HuffTable`, `parse_headers`,
   and `ScanReader` (`decode_difference`, `restart`). You build on these; read them.

Before coding, mark the build cycle `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart-timeline.md`.
If something needs architect judgment (constraint unclear, scope drift, spec
ambiguity), change it to `[?]` with a one-line reason and **stop** — ask; don't guess.

Implement (all in `src/ljpeg.rs`, same `pub(crate)` module):

- `reconstruct(frame, table, &mut reader, out)` + `decode_into(data, out) ->
  Result<Frame>` per `## Outputs`: predictors 1–7 (T.81 Table H.1), the §H.2.1
  boundary rules (first sample / first line / first column / post-restart), the
  point transform (`out == reconstructed << Pt`, prediction in the
  point-transformed domain), and restart reset. Call
  `reader.decode_difference(table)` once per sample and `reader.restart()` at each
  restart boundary — **do not** re-decode or re-parse.
- Add `Error::LjpegPlaneWrongLength { expected, got }` (the `out`-length check);
  follow the `Ljpeg*` + `*WrongLength` style. Caller-owned, **no allocation**
  (`DEC-016`); modular `2^16` via explicit `wrapping_*` (`DEC-018`).
- The `encode_lj92` test harness + the bit-exact **round-trip oracle** + the
  **perturbed-decoder red-proof** (must be shown RED — `oracle-must-be-shown-red`;
  the `plane_oracle.rs` no-op lesson). The harness also seeds SPEC-038, so keep it
  clean and parameterised.
- Add the `docs/provenance-ledger.md` reconstruction row (T.81 Annex H.1, `DEC-035`).
- Clean-room from T.81 — the cross-reference crates are read-for-structure only,
  never copied/linked. **Do not** touch `src/ifd.rs` or `plane.rs`'s unpacker
  (STAGE-010), and **do not** fuzz (SPEC-038).
- `#![forbid(unsafe_code)]` + the five-lint deny policy. Pre-named L-risk escape
  hatch: if it runs long, ship (a) `reconstruct` + direct tests, then (b) the
  `encode_lj92` round-trip oracle + red-proof; don't expand scope.

When done:

1. Fill the handoff's `handback:` block **and** `## Handback` section — INCLUDING a
   real `tokens_total` (run `/cost`; if your interface has no meter, report the
   `ccd_session_mgmt get_usage` context-occupancy as an explicit FLOOR with a note,
   as HANDOFF-085/086 did — do not invent a cumulative). The reflection is required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-037 verify`
4. Open a PR from `claude/spec-037-build-<suffix>` against `main`.
5. PR description references: `PROJ-003`, `STAGE-009`, `SPEC-037`, `HANDOFF-087`,
   `DEC-035`/`DEC-016`/`DEC-018`, and any new `DEC-*`.
6. Mark build `[x]` in the timeline with the PR number, cost, and date.

Gates before the PR — all green: `just test` · `just lint-ci -D warnings` ·
`just fmt-check` · `just validate`.
