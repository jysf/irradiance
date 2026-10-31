---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-028
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-024]             # exercised on the Bayer plane the CFA gate now accepts

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Unblocks the Bayer plane for SPEC-025's bit-exact oracle — the Nikon D300S plane is 12 strips, which the single-strip unpacker rejects today."

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
  tokens_estimate: null
  sessions:
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 43553730
      estimated_usd: 102.25
      duration_minutes: 35
      recorded_at: 2026-10-29
      notes: "all 7 Failing Tests + the corpus check pass; one resolved spec-text tension (per-strip layer-0 is byte-ceil, not literal `== rows*width*bits`, so sub-byte strips can carry their independent tail padding — see Deviations); tests placed in tests/plane_unpack.rs (the unpack_into home, AGENTS.md §12) not src/ifd.rs mod tests; tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated"
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 8884772
      estimated_usd: 21.10
      duration_minutes: 40
      recorded_at: 2026-10-29
      notes: "✅ APPROVED at commit 233ae5c (PR #41, already MERGED to main before this verify ran — a retrospective independent review; no build branch remained to push to, hence this small verify-bookkeeping PR). Every AC reproduced cold: full suite (14 binaries, 0 failed, corpus 7/7 so the Nikon tier-B test actually ran, not skipped), pinned CI clippy 0.1.98 -D warnings, cargo fmt --check, just validate, just lint-no-allow (the five panic lints), and `plane` fuzz 20,465,198 runs/91s 0 crashes. Adversarial mutation (ceil->floor on per-strip layer-0) flips a_two_strip_14bit red while 16-bit stays green, control green restored. Nikon 12-strip plane assembles to 4352x2868 then trips SampleExceedsWhiteLevel (sample 741623=3835>3808) — expected, SPEC-029's to relax. Both build deviations adjudicated ACCEPTABLE; one non-blocking follow-up SPEC-028/FU-1 (amend the spec Outputs formula to the byte-ceil form at ship). tokens_total summed from this verify session's own transcript JSONL (scratchpad-UUID anchored, deduped by message.id; all 52 usage records claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (8.64M of 8.88M); estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
  totals:
    tokens_total: 52438502
    estimated_usd: 123.35
    session_count: 2
shipped_at: 2026-10-29
---

# SPEC-028: Multi-strip mosaic plane assembly: unpack a Bayer plane stored as N strips (`plane::unpack_into` is single-strip-only, SPEC-012 `src/plane.rs:225`; the Nikon D300S is 12 strips)

> **DESIGN — `cycle: design`.** Approach + failing tests below; a build session
> makes the Failing Tests pass without redesigning. Blocks SPEC-025 (the oracle).

## Context

`plane::unpack_into` reads **one** strip: SPEC-012 scoped it single-strip-only
(`src/plane.rs:225` "tiles and multi-strip planes are a non-goal"), and `sensor()`
errors `expected exactly one strip, found N` (`src/lib.rs:544`) otherwise — every
PROJ-001 corpus file was single-strip. The Nikon D300S Bayer DNG stores its plane
in **12 strips** (`RowsPerStrip` 256×11 + 52), so it does not unpack today. This
spec extends the unpacker to assemble N strips into the contiguous plane buffer —
a general improvement the Bayer corpus surfaced, not CFA-specific — and is a
prerequisite for the bit-exact plane oracle (SPEC-025).

## Goal

Unpack a multi-strip sensor plane — N strips located by `StripOffsets` /
`StripByteCounts`, each `RowsPerStrip` rows of a packed bitstream — into the plane
buffer, bounds-checked and panic-free, so the Nikon D300S (12 strips) unpacks where
it errored before. The single-strip path stays byte-for-byte unchanged.

## Inputs

- **Files to read:**
  - `src/plane.rs:235` `unpack_into` — the whole function: the single-strip gate
    (`:265`), layer-0 (`:287`), the one strip read (`:305`), and the two unpack
    paths (`unpack_byte_aligned` / `unpack_bitstream`, `:310`).
  - `src/lib.rs:538` `Error::UnsupportedStripLayout`; `Sensor.strip_offsets` /
    `strip_byte_counts` / `rows_per_strip`.
  - DEC-008 (byte-aligned vs sub-byte unpack — each strip is byte-padded
    independently for sub-byte depths); SPEC-012 (the single-strip unpacker).
  - The `plane` tests + the `Build` TIFF DSL (`src/ifd.rs mod tests`) — extend it to
    write multiple strips.
- **Corpus:** `NIKON-D300S/_DSC4615.DNG` (12 strips, 16-bit); `LEICA-Q2-MONO`
  (14-bit) + `LEICA-M-MONOCHROM` (16-bit) single-strip for the regression.

## Outputs

- **Files modified:** `src/plane.rs` (+ `src/lib.rs` if an error shifts).
- **Multi-strip assembly in `unpack_into`:** replace the `strip_offsets.len() != 1`
  gate (`:265`) with a loop over N strips (`N == strip_offsets.len() ==
  strip_byte_counts.len()`, N ≥ 1). For strip `i`:
  - `rows_i = min(rows_per_strip, height - i*rows_per_strip)` (last strip is the
    remainder). `rows_per_strip` is **required** when N > 1 — typed error if absent.
  - Per-strip layer-0 (byte-ceil form, SPEC-028/FU-1): `strip_byte_counts[i] ==
    ((rows_i * width * bits) + 7) / 8` — each strip pads to a byte boundary
    **independently**, so its byte count is the *ceiling* of its bit count over 8
    (the exact `* 8 ==` holds only when `rows_i * width * bits` is a multiple of 8).
    This is why strips unpack per-strip, not by concatenating bytes.
  - Read that strip's bytes (bounds-checked → `Truncated`) and unpack via the
    existing byte-aligned / bitstream path into
    `dst[i*rows_per_strip*width .. + rows_i*width]`.
  - Total still satisfies `sum(strip_byte_counts)*8 == packed_bits`.
- **Single-strip (N == 1) stays the exact current path** — a degenerate loop
  iteration, byte-for-byte identical output.
- `UnsupportedStripLayout` fires only on a genuine mismatch (`offsets.len() !=
  byte_counts.len()`, or N == 0); a typed error if `rows_per_strip` is missing on a
  multi-strip plane. **No tiles** (`TileOffsets`) — still unsupported.

## Acceptance Criteria

- [ ] A synthetic 2-strip **16-bit** plane unpacks to the same samples as the
  single-strip equivalent (byte-aligned, per-strip assembly).
- [ ] A synthetic 2-strip **14-bit** plane (sub-byte; each strip byte-padded
  independently) unpacks correctly — strip 2's rows are not shifted by strip 1's padding.
- [ ] An uneven last strip (`height % rows_per_strip != 0`) unpacks the right remainder.
- [ ] Single-strip planes (Q2M 14-bit, M Monochrom 16-bit) unpack **byte-identically**
  to before (regression).
- [ ] `rows_per_strip` absent on a multi-strip plane → a typed error, not a panic.
- [ ] A strip past EOF → `Error::Truncated`; a strip byte count ≠ its rows →
  `Error::PackedSizeMismatch`. Panic-free on both.
- [ ] (corpus) `NIKON-D300S/_DSC4615.DNG` unpacks into a full `width*height` plane,
  no `UnsupportedStripLayout`.

## Failing Tests

Extend the `Build` TIFF DSL to write a plane as **multiple strips** (N
`StripOffsets`/`StripByteCounts` + `RowsPerStrip`).

- **`src/plane.rs` / `src/ifd.rs mod tests`**
  - `a_two_strip_16bit_plane_unpacks_like_one_strip`
  - `a_two_strip_14bit_plane_respects_per_strip_byte_padding` — strip 2's first
    sample is correct (not shifted by strip 1's tail padding)
  - `an_uneven_last_strip_unpacks_the_remainder`
  - `a_single_strip_plane_is_unchanged` (regression, Q2M / M Monochrom)
  - `a_multi_strip_plane_without_rows_per_strip_is_a_typed_error` (no panic)
  - `a_truncated_strip_is_truncated_not_a_panic`;
    `a_strip_byte_count_mismatch_is_packed_size_mismatch`
- **Regression:** the existing single-strip plane + develop oracles stay green.

## Non-Goals

- Tiled planes (`TileOffsets` / `TileByteCounts`) — still out unless a corpus file needs it
- The WhiteLevel-exceedance policy — SPEC-029
- The bit-exact oracle assertion vs `dnglab` — SPEC-025
- Demosaic / colour / any pixel interpretation — later stages

## Notes for the Implementer

- **Per-strip byte padding is the trap.** For sub-byte depths (12/14) each strip is
  an independent bitstream padded to a byte boundary, so unpack each strip into its
  own row slice — do NOT concatenate strip bytes and bit-unpack the whole. For 8/16
  concatenation would happen to work, but write the per-strip loop once for both. The
  Nikon corpus file is 16-bit, so the 14-bit per-strip case is covered only by the
  synthetic test — keep it.
- **Keep N == 1 the current path** (a degenerate iteration) so the regression holds
  byte-for-byte; don't fork mono vs multi.
- Panic-free: every strip offset/length bounds-checked into `Truncated`, per-strip
  layer-0 into `PackedSizeMismatch`. General improvement the Bayer corpus surfaced —
  not CFA-specific; benefits mono multi-strip too.
- WhiteLevel-exceedance is **SPEC-029**'s job; a Bayer file may still trip
  `SampleExceedsWhiteLevel` here until 029 lands (independent; SPEC-025 needs both).

---

## Reflection

*Backfilled at the STAGE-006 close, 2026-10-30, from this spec's timeline +
HANDOFF-067 (verify). The prose reflection step was skipped at the original ship
(back-to-back with SPEC-029); front-matter (`complexity_actual`,
`verify_verdict`) was stamped then. Recorded as signal
[[spec-reflection-shipped-unfilled]].*

1. **What would I do differently next time?**
   — Probe the real corpus files at **frame** time, not at the capstone's design
   time. The whole reason this spec exists is that SPEC-025 was framed assuming the
   Bayer plane already unpacked, and the Nikon D300S turned out to be 12 strips
   while `plane::unpack_into` was single-strip-only (SPEC-012). A ten-minute
   "does `unpack_into` return a plane for these two files?" spike at STAGE-006's
   framing would have surfaced this precursor (and SPEC-029) before the capstone
   was specced. SPEC-025's reflection names the same miss from the other side.

2. **Does any template, constraint, or decision need updating?**
   — No DEC. The single slip was spec-prose: the Outputs section's byte-ceiling
   wording (SPEC-028/FU-1), caught cold at verify and filed non-blocking — a doc
   nit, not policy. The N-strip assembly and `Error::MissingRowsPerStrip` matched
   the design.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. SPEC-029 (the WhiteLevel-exceedance relaxation) was the sibling precursor
   discovered alongside this one and was already framed; once both landed, the
   capstone SPEC-025 (bit-exact mosaic-plane oracle) could run.

4. **Where was the worst defect caught?**
   — `verify` — no code defect (all 7 ACs reproduced at `233ae5c`; the single-strip
   regression held byte-for-byte on Q2M 14-bit + M Monochrom 16-bit; both build
   deviations ruled acceptable). The worst thing caught was the spec-Outputs
   wording imprecision (FU-1), surfaced by the cold review and non-blocking.

5. **What can a user do now that they couldn't before?**
   — Before, a Bayer plane stored as N strips errored `UnsupportedStripLayout` —
   the Nikon D300S (12 strips) could not be read at all. After,
   `plane::unpack_into` assembles an N-strip mosaic (with per-strip row handling
   and `Error::MissingRowsPerStrip` for a truncated strip table); the Nikon
   4352×2868 12-strip plane assembles, single-strip reads (Q2M 14-bit, M Monochrom
   16-bit) stay byte-for-byte unchanged, and the `plane` fuzz target ran 20,465,198
   runs / 91s with 0 crashes.
