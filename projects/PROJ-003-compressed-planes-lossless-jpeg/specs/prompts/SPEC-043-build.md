# SPEC-043 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-043 —
**route a DNG `Compression 7` (SOF-3 lossless JPEG) sensor plane through the
shipped `ljpeg::decode_into`**, so a Compression-7 file decodes to the `u16` plane
instead of returning `UnsupportedCompression`. First spec of STAGE-011 — it makes
the STAGE-009 decoder reachable on real files.

Read files in order, before writing code:

1. `/AGENTS.md` — conventions you must follow.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-091-build-compression-7-recognition-strip-path-routing.md`
   — your handoff. Follow its "Context the Receiving Agent Needs" exactly.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` + the stage
   `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
4. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-043-compression-7-recognition-strip-path-routing.md`
   — the spec; `## Inputs` / `## Outputs` / `## Acceptance Criteria` / `## Failing
   Tests` / `## Notes for the Implementer` are the contract. `## Inputs` carries
   the §12 probe of both held files (both **single-strip**, 12/14-bit, MM/II).
5. `DEC-016` (caller-owned no-alloc buffer), `DEC-012` (structural gate rule),
   `DEC-036` (`ljpeg` stays `pub(crate)` — do **NOT** widen it or retire the
   `fuzzing` hook; that is SPEC-045), `DEC-003` (own-work fixtures — no corpus).
6. `/guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
   `no-new-top-level-deps-without-decision`.
7. `./guidance/toolchain-brief.md`; the shipped `src/plane.rs` (`unpack_into` + the
   per-strip loop), `src/ljpeg.rs` (`decode_into`, the `Frame` struct, and
   `encode_lj92` / `encode_scan` in `mod tests` — your fixture shape), `src/ifd.rs`
   (the `Compression` enum, `require_uncompressed`, the `Sensor` fields).

Before coding, mark the build cycle `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-043-compression-7-recognition-strip-path-routing-timeline.md`.
If something needs architect judgment or the spec is ambiguous, change it to `[?]`
with a one-line reason and **stop** — ask; don't guess.

Implement (per `## Outputs`):

- **Dispatch on `sensor.compression` at the TOP of `plane::unpack_into`**, in
  place of the unconditional `sensor.require_uncompressed()?`:
  `Compression::Uncompressed` → the existing sub-byte path **byte-for-byte
  unchanged**; `Compression::Jpeg` → the new per-strip SOF-3 route; `other =>
  Err(Error::UnsupportedCompression { compression: other.code() })` (unchanged).
- **The `Jpeg` route** reuses the existing strip iteration + row-band slicing but
  **SKIPS** the sub-byte unpack and the layer-0 packed-byte-count check
  (meaningless for a compressed strip — see the spec's probe), decoding each strip
  with `ljpeg::decode_into(strip_bytes, strip_dst)` into its `rows_in_strip ×
  width` slice. N≥1 uniform (both held files are single-strip; N>1 is tier-A only).
- `decode_into` stays `pub(crate)`; `unpack_into`'s signature is unchanged. **No
  new `Error` variant** (the decoder's typed `Ljpeg*` / `LjpegPlaneWrongLength` /
  `Truncated` cover it). No new runtime dep, no CLI change.
- Update the stale `src/ljpeg.rs` doc comment ("the entry point STAGE-010 wires
  `Compression 7` to") to name STAGE-011 / this spec.
- **Tier-A fixtures** via the STAGE-009 encoder (`encode_lj92`): make every
  SPEC-043 `## Failing Test` pass — single-strip bit-exact for both byte orders,
  the uncompressed-unchanged characterization, a 2-strip multi-strip case, the
  dimension-mismatch typed error, the truncated-strip typed error, and the
  non-JPEG-still-rejected case.

When done:

1. Fill `HANDOFF-091`'s `handback:` block + `## Handback` section with a **real**
   `tokens_total` (run `/cost`; if your interface has no meter — the Claude Code
   desktop Code tab does not — report the `ccd_session_mgmt get_usage`
   context-occupancy as an explicit **FLOOR** with a note, as HANDOFF-085..090
   did; do not invent a cumulative). Reflection required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-043 verify`
4. Open a PR from `claude/spec-043-build-<suffix>` against `main`, referencing
   `PROJ-003`, `STAGE-011`, `SPEC-043`, `HANDOFF-091`, and
   `DEC-016`/`DEC-012`/`DEC-036`/`DEC-003`.
5. Mark build `[x]` in the timeline with the PR number, cost, and date.

Gates before the PR — all green: `just test` · `just lint-ci` · `just fmt-check` ·
`just validate`.
