# SPEC-036 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-036 —
the SOF-3 lossless-JPEG **entropy front-end** (markers, bit reader, DC Huffman).

Read files in order, before writing code:

1. `/AGENTS.md` — conventions you must follow.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-085-build-sof-3-entropy-front-end-markers-bit-reader-dc-huffman.md`
   — your handoff. Follow its "Context the Receiving Agent Needs" exactly.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` — project context.
4. `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
   — the stage.
5. Every decision in the handoff's references: `DEC-035` (patent-clean +
   provenance), `DEC-018` (u16 modular arithmetic), `DEC-016` (caller-owned buffers).
6. `/guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
   `provenance-recorded-per-algorithm`, `test-before-implementation`.
7. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman.md`
   — the spec; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
8. `./guidance/toolchain-brief.md` — this repo's test/lint/run quirks.

Before coding, mark the build cycle `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman-timeline.md`.
If you hit something needing architect judgment (constraint unclear, scope
drift, spec ambiguity), change it to `[?]` with a one-line reason and **stop** —
ask first; don't guess.

Implement:

- Make the spec's `## Failing Tests` pass and satisfy every `## Acceptance Criterion`.
- New module `src/ljpeg.rs` (`mod ljpeg;` in `src/lib.rs`, items `pub(crate)`).
  **Mirror** `src/plane.rs`'s `BitReader` (checked arithmetic, no `<<`/`+`,
  `pow2`, `Error::Truncated`) and **add** `0xFF` destuffing + marker detection —
  a separate reader, not a reuse.
- Add the new `Error` variants to `src/lib.rs` in the documented,
  `#[non_exhaustive]` struct-variant style; reuse `Error::Truncated` for EOF.
- Add the provenance-ledger row (T.81 Annexes B/C/F/H, `DEC-035`).
- **Clean-room from T.81.** The cross-reference crates (`oxideav-mjpeg`,
  `photocraft-raw`) are read-for-structure only — never copied or linked.
- **Do not** touch `src/ifd.rs` or `plane.rs`'s unpacker (STAGE-010), and **do
  not** implement reconstruction/predictors/point-transform/the u16 plane
  (SPEC-037). Stop at the difference stream + parsed `Frame`.
- `#![forbid(unsafe_code)]` + the five-lint deny policy; explicit
  `wrapping_*`/`checked_*` (no silencing casts for `arithmetic_side_effects`).
- Pre-named L-risk split (escape hatch): if it runs long, ship (a) marker/frame
  parse → `Frame` + entropy offset, then (b) `HuffTable` + `ScanReader`; don't
  expand scope.

When done:

1. Fill the handoff's `handback:` block **and** `## Handback` section —
   INCLUDING a real `tokens_total` (run `/cost` in Claude Code; if your interface
   has no meter, `null` + why in `notes` — do not invent one). The reflection is
   required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-036 verify`
4. Open a PR from `claude/spec-036-build-<suffix>` against `main`.
5. PR description references: `PROJ-003`, `STAGE-009`, `SPEC-036`, `HANDOFF-085`,
   `DEC-035`/`DEC-018`/`DEC-016`, and any new `DEC-*`.
6. Mark build `[x]` in the timeline with the PR number, cost, and date.

Gates before the PR — all green: `just test` · `just lint-ci -D warnings` ·
`just fmt-check` · `just validate`.
