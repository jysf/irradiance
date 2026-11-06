# SPEC-046 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-046 —
**extend `ljpeg` to decode a two-component (`Nf=2`) column-interleave SOF-3 frame**
and de-interleave the two column components into one monochrome `u16` plane, so both
held Compression-7 DNGs decode. The re-plan spec (DEC-037); it unblocks SPEC-044's
bit-exact oracle. The `Nf=1` path stays byte-for-byte unchanged.

Read files in order, before writing code:

1. `/AGENTS.md`.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-093-build-two-component-nf2-sof-3-decode-and-column-de-interleave.md`
   — your handoff; follow its "Context the Receiving Agent Needs" exactly.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` + the stage
   `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
4. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-046-two-component-nf2-sof-3-decode-and-column-de-interleave.md`
   — the spec; `## Inputs` carries the §12 probe (one interleaved scan, Ns=2, both
   H=V=1, one Td=0 DC table, predictor 1, Pt=0, no DRI), and `## Outputs` /
   `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
5. `DEC-037` (the two-component re-scope — read first), `DEC-035` (clean-room),
   `DEC-016` (caller-owned buffer), `DEC-018` (u16 modular arithmetic).
6. `/guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
   `no-new-top-level-deps-without-decision`.
7. `./guidance/toolchain-brief.md`; the shipped `src/ljpeg.rs` — `parse_frame_header`
   (rejects `nf != 1`), `parse_scan_header` (rejects `ns != 1`), the `Frame` struct,
   `reconstruct`, `decode_into` (length check), `predict`, `HuffTable`, and
   `encode_lj92` / `encode_scan` in `mod tests` (the encoder to extend).

Before coding, mark the build cycle `[~]` in the SPEC-046 timeline. If the spec is
ambiguous or needs architect judgment, change it to `[?]` with a one-line reason and
**stop** — ask; don't guess.

Implement (per `## Outputs`):

- `Frame` gains a component count (e.g. `components: u8`).
- `parse_frame_header`: accept `Nf ∈ {1,2}` (read `Nf` component specs; `Lf == 8 + 3·Nf`);
  reject `Nf` outside `{1,2}` with `LjpegUnsupportedComponentCount`.
- `parse_scan_header`: accept `Ns ∈ {1,2}` (read `Ns` selectors with their `Td`;
  `Ls == 6 + 2·Ns`); require `Ns == Nf`; build up to two DC Huffman tables keyed by `Td`
  and have each component select its own (do NOT hard-code one table).
- `reconstruct`: for `Nf == 2`, decode the interleaved scan — at each `(x, y)` decode
  comp0 then comp1 — with **per-component** predictor state (each its own Ra/Rb/Rc and the
  §H.2.1 boundary / `2^(P−Pt−1)` rules), writing into a `2·X`-wide row column-interleaved
  (`out[y·2X + 2x] = comp0`, `out[y·2X + 2x + 1] = comp1`). Keep the `Nf=1` path
  byte-for-byte unchanged.
- `decode_into`: length check `out.len() == Nf · samples_per_line · lines`
  (`LjpegPlaneWrongLength` otherwise). Stays `pub(crate)`. No new `Error` variant, no new
  dependency.
- Extend the in-harness `encode_lj92` to emit a two-component interleaved scan so the
  analytic round-trip covers `Nf=2`.
- Make every SPEC-046 `## Failing Test` pass (tier-A). The analytic round-trip proves the
  decode self-consistent + per-component prediction; it does NOT confirm the real-file
  de-interleave mapping (it shares the convention — `shared-path-oracle-tautology`); that is
  SPEC-044's `dnglab` check.

When done:

1. Fill `HANDOFF-093`'s `handback:` block + `## Handback` with a **real** `tokens_total`
   (run `/cost`; if your interface has no meter — the Claude Code desktop Code tab does not
   — report the `ccd_session_mgmt get_usage` occupancy as an explicit **FLOOR** with a note,
   as HANDOFF-089..092 did; do not invent). Reflection required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-046 verify`
4. Open a PR from `claude/spec-046-build-<suffix>` against `main`, referencing `PROJ-003`,
   `STAGE-011`, `SPEC-046`, `HANDOFF-093`, `DEC-037`.
5. Mark build `[x]` in the timeline with the PR number, cost, and date.

Gates before the PR — all green: `just test` · `just lint-ci` · `just fmt-check` ·
`just validate`.
