---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-043
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-091 (build) → tier_map.build
  created_at: 2026-11-05

references:
  decisions: [DEC-036, DEC-016, DEC-012]   # ljpeg visibility — the dev-only fuzzing feature may retire once this adds a real caller (DEC-036); caller-owned no-alloc buffer (DEC-016); malformed-tag / structural-gate rule for Compression recognition (DEC-012)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-037, SPEC-038, SPEC-028]   # the shipped SOF-3 decoder (037 completes it, 038 fuzzed it) + the multi-strip plane assembly (028) this routing reuses

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-011's <capability>". Optional; null is acceptable.
value_link: "Makes the shipped SOF-3 decoder reachable — recognises Compression 7 on the sensor IFD and routes strip/tile bytes through ljpeg::decode_into — the binding half of STAGE-011's deliverable."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 28000000        # ~28M — routing dispatch + tier-A SOF-3 fixtures (single + 2-strip) + characterization; [M], comparable to SPEC-037
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 40
      recorded_at: 2026-11-05
      notes: "main-session design (un-metered); §12 probe of the two held files (both single-strip, 12/14-bit, MM/II) + the unpack_into routing seam + the Frame/decode_into contract; acceptance + failing tests + HANDOFF-091 + build prompt"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-043: Compression-7 recognition + strip-path routing

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-043]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

STAGE-009 shipped `ljpeg::decode_into` (compressed bytes → `u16` plane,
bit-exact and panic-free) but kept it `pub(crate)` with **no caller** — a DNG
whose sensor IFD carries `Compression == 7` (SOF-3 lossless JPEG) still returns
`Error::UnsupportedCompression` at the plane gate. This is the first spec of
`STAGE-011`'s backlog (`PROJ-003`): it makes the decoder reachable by recognising
`Compression == 7` and routing the strip/tile bytes to it where the packed-bits
unpack sits today, for both container byte orders. SPEC-044 then proves the result
bit-exact against `dnglab`; SPEC-045 is the capstone. Builds on the shipped
SPEC-037 (the decoder), SPEC-038 (its fuzz hardening), and SPEC-028 (the
multi-strip plane assembly this reuses).

## Goal

Recognise `Compression == 7` on the sensor IFD and route the strip/tile bytes
through `ljpeg::decode_into` (reusing SPEC-028's multi-strip assembly), for both
container byte orders, so a Compression-7 DNG decodes to a `u16` plane instead of
returning `UnsupportedCompression` — while the uncompressed / monochrome / Bayer
paths stay byte-for-byte unchanged and the binding path stays panic-free.

## Inputs

- **Files to read:**
  - `src/plane.rs` — `unpack_into(sensor, byte_order, file, dst)` (≈ line 261) and
    its per-strip assembly loop; `sensor.require_uncompressed()?` is the first line
    (≈ 267) and today rejects Compression 7. **This is the routing seam.**
  - `src/ljpeg.rs` — `pub(crate) fn decode_into(data: &[u8], out: &mut [u16]) ->
    Result<Frame, Error>` (≈ 992) and the `Frame` struct (≈ 182: `precision`,
    `lines` = Y/height, `samples_per_line` = X/width, `point_transform`,
    `predictor`, `restart_interval`). The decoder this spec feeds. **Do not change
    its algorithm.**
  - `src/ifd.rs` — the `Compression` enum (`Uncompressed=1`, `Jpeg=7`,
    `Other(u32)`; ≈ 480), `Sensor::require_uncompressed` (≈ 758), and the `Sensor`
    fields `compression`, `width`, `height`, `bits_per_sample`, `strip_offsets`,
    `strip_byte_counts`, `rows_per_strip`.
  - `src/bin/irr.rs` — the four `plane::unpack_into` call sites (ifd / develop /
    demosaic / color subcommands) that must transparently get a decoded plane.
  - `STAGE-011` stage file + `brief.md`.
- **§12 probe (done at design, on the REAL held files via `irr ifd --entries`):**
  - `LEICA-M-MONOCHROM-TYP246/M2462362.DNG`: big-endian (MM), `bits_per_sample 12`,
    `compression 7`, **single strip** (`strip_offsets [10686]`,
    `strip_bytes [21311750]`), `rows_per_strip 4000`, crop 5976×3992.
  - `PENTAX-K3III-MONO/K3III.DNG`: little-endian (II), `bits_per_sample 14`,
    `compression 7`, **single strip** (`strip_offsets [181248]`,
    `strip_bytes [31047287]`), `rows_per_strip 4224`, crop 6192×4128.
  - **Both held Compression-7 files are single-strip** → the single-strip route is
    the corpus-verified path (SPEC-044). Multi-strip SOF-3 (N>1) has **no held
    file**, so it is tier-A-only.
- **Related code paths:** `src/plane.rs`, `src/ljpeg.rs`, `src/ifd.rs`,
  `src/bin/irr.rs`.

## Outputs

- **Files modified:**
  - `src/plane.rs` — `unpack_into` dispatches on `sensor.compression` in place of
    the unconditional `require_uncompressed()?`: `Compression::Uncompressed` → the
    existing sub-byte path **unchanged**; `Compression::Jpeg` → the new per-strip
    SOF-3 route; anything else → `Error::UnsupportedCompression` (unchanged). The
    SOF-3 route reuses the existing strip iteration + row-band slicing but
    **skips** the sub-byte unpack and the layer-0 packed-byte-count check
    (meaningless for a compressed strip — the probe shows declared bytes ≪ packed
    bits), decoding each strip with `ljpeg::decode_into(strip_bytes, strip_dst)`
    into its `rows_in_strip × width` row-band slice.
  - `src/ljpeg.rs` — update only the stale doc comment ("the entry point STAGE-010
    wires `Compression 7` to") to name STAGE-011 / this spec. `decode_into` is
    already `pub(crate)` (same crate) — no visibility change; the public-surface /
    `fuzzing`-hook-retirement decision stays SPEC-045's (DEC-036).
- **New functions:** a private helper in `plane.rs` (e.g. `decode_jpeg_strip`) or
  an inline `Jpeg` arm — single-component SOF-3 strip → row band via `decode_into`.
  **No new public API; `unpack_into`'s signature is unchanged** — a Compression-7
  file now returns `Ok(())` with a decoded plane instead of `UnsupportedCompression`.
- **New `Error` variants:** **none** — `decode_into`'s typed `Ljpeg*` /
  `LjpegPlaneWrongLength` / `Truncated` cover decode failures; `UnsupportedCompression`
  still covers non-1/7 codes.
- **No new flags, no new runtime dependency, no CLI change** — the `irr`
  subcommands gain Compression-7 support transparently through `unpack_into`.
- **Strip-count scope (pinned at design):** route **per strip, N≥1 uniform** — a
  Compression-7 strip is an independent SOF-3 datastream decoded into its own row
  band (the same independent-per-strip rule as SPEC-028). Single-strip is
  corpus-verified (both held files; SPEC-044); N>1 is covered **tier-A only** (a
  synthetic 2-strip fixture) and flagged, since no held file is multi-strip. Tiled
  Compression-7 stays a non-goal, as tiles already are for the uncompressed path.

## Acceptance Criteria

- [ ] `plane::unpack_into` on a **single-strip Compression-7** sensor fills `dst`
  with the decoded `u16` plane via `ljpeg::decode_into` instead of returning
  `UnsupportedCompression` — exercised on a tier-A hand-built single-strip SOF-3
  fixture (emitted by the STAGE-009 encoder) for **both** container byte orders,
  **bit-exact** against the known samples.
- [ ] The **uncompressed path is byte-for-byte unchanged** — every existing
  `plane` / `develop` / `demosaic` / `color` test stays green, and a
  characterization test pins that a `Compression::Uncompressed` plane is identical
  before/after this change.
- [ ] A **multi-strip (N>1) Compression-7** plane decodes each strip independently
  into its row band — exercised on a tier-A 2-strip synthetic fixture, bit-exact.
- [ ] A Compression-7 strip whose **SOF-3 frame dimensions disagree** with its
  `rows_in_strip × width` slice returns a typed `Error` (`LjpegPlaneWrongLength`),
  never a panic.
- [ ] A **truncated / malformed Compression-7 strip** returns a typed `Error` (the
  `Ljpeg*` / `Truncated` family), never a panic — the binding inherits the
  decoder's panic-freedom.
- [ ] A non-1/7 `Compression` code still returns `Error::UnsupportedCompression`
  with the code (unchanged); `#![forbid(unsafe_code)]` + the five-lint deny policy
  hold; `just test`, `just lint-ci`, `just fmt-check`, `just validate` all green.

## Failing Tests

Tier-A only (hand-built SOF-3 bitstreams via the STAGE-009 encoder; no corpus).
The tier-B `dnglab --raw-checksum` bit-exact proof on the two real files is
**SPEC-044**, not here.

- **`src/plane.rs` (`mod tests`), or `tests/plane_compression7.rs`:**
  - `"compression7_single_strip_decodes_bit_exact"` — a small known plane encoded
    as one SOF-3 strip; a minimal `Sensor { compression: Jpeg, one strip }`;
    assert `unpack_into` fills `dst` bit-exact with the known samples, for both
    byte orders.
  - `"uncompressed_path_unchanged_after_routing"` — an uncompressed `Sensor`
    unpacks identically to the pre-change behaviour (characterization).
  - `"compression7_multi_strip_decodes_each_band"` — two independent SOF-3
    datastreams as two strips decode into their respective row bands, bit-exact.
  - `"compression7_frame_dimension_mismatch_is_typed_error"` — a SOF-3 strip whose
    `samples_per_line × lines` ≠ the sized slice returns `LjpegPlaneWrongLength`,
    no panic.
  - `"compression7_truncated_strip_is_typed_error"` — a truncated SOF-3 strip
    returns a typed `Error`, no panic.
  - `"non_jpeg_compression_still_rejected"` — `Compression::Other(65535)` →
    `UnsupportedCompression { compression: 65535 }`.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The bit-exact corpus oracle vs `dnglab --raw-checksum` — **SPEC-044**.
- The provenance-ledger entry, the macOS/Linux determinism assertion, and the
  container→decoder hardening fuzz — **SPEC-045**.
- Multi-component / Bayer-in-LJ92 interleave, and OpcodeList2 GainMap — deferred
  (no held file; PROJ-003 roadmap).
- Any change to the SOF-3 decoder algorithm itself — that is STAGE-009 (shipped);
  this spec only feeds it real bytes.

## Notes for the Implementer

- **The routing seam is the top of `unpack_into`:** replace the unconditional
  `sensor.require_uncompressed()?` with a `match sensor.compression`. Keep the
  `Uncompressed` arm byte-for-byte identical (same sub-byte path, same per-strip
  layer-0 check). Add a `Jpeg` arm; keep `other =>` returning
  `UnsupportedCompression { compression: other.code() }`.
- **A Compression-7 strip is an independent SOF-3 datastream** (DNG/TIFF JPEG):
  decode each strip's bytes with `decode_into(strip_bytes, strip_dst)` into its
  `rows_in_strip × width` row-band slice — the same independent-per-strip rule
  SPEC-028 established. **Do NOT run the layer-0 packed-byte-count check or the
  sub-byte unpack on a Compression-7 strip** — the probe shows declared compressed
  bytes ≪ `width × rows × bits / 8`; that check is uncompressed-only.
- `decode_into` already length-checks `out.len() == X·Y` → `LjpegPlaneWrongLength`,
  so sizing the row-band slice to `rows_in_strip × width` makes the
  frame-dimension consistency check free. Asserting `frame.samples_per_line ==
  width` as well only buys a clearer error; optional.
- **`bits_per_sample` for a Compression-7 plane is the SOF-3 precision** (12 for
  Typ246, 14 for K3III), which `decode_into` reads from the frame — do **not** gate
  the Compression-7 path on `SUPPORTED_BITS` (that is the sub-byte unpacker's
  table). Size `dst` as `u16` × width × height regardless.
- **Both held files are single-strip** (the §12 probe). Keep the N>1 route a thin
  loop over the existing strip iteration — not a new mechanism — and cover it with
  the tier-A 2-strip fixture; it has no real-file oracle.
- `decode_into` stays `pub(crate)`; do not widen it. The public-surface decision
  and retiring the `fuzzing` hook (DEC-036) are **SPEC-045's** call, not this spec's.
- Bit-exactness against the real files (`dnglab --raw-checksum`), the determinism
  assertion, provenance, and binding-path fuzz are **SPEC-044 / SPEC-045**.
- `git grep "STAGE-010"` in `src/` will surface the stale "STAGE-010 wires
  Compression 7" comment in `src/ljpeg.rs`; update it to STAGE-011 / this spec.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
