---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-046
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   two-component scan + per-component predictor state + column
                                   #   de-interleave; could prove L — confirm at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: claude-opus-5          # HANDOFF-093 (build) → tier_map.build
  created_at: 2026-11-05

references:
  decisions: [DEC-037, DEC-035, DEC-016, DEC-018]   # two-component re-scope (DEC-037); clean-room/patent-clean (DEC-035); caller-owned buffer (DEC-016); u16 modular arithmetic (DEC-018)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-037, SPEC-043]   # the single-component decoder this extends (037); the routing that feeds it (043, shipped)

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-043]             # the Compression-7 routing (merged) feeds strips to the decoder this extends

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-011's <capability>". Optional; null is acceptable.
value_link: "Extends the SOF-3 decoder to two-component (Nf=2) column-interleave so both held Compression-7 files decode to one monochrome u16 plane — the prerequisite for STAGE-011's bit-exact oracle (SPEC-044)."

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
  tokens_estimate: 32000000        # ~32M — two-component interleaved decode + per-component predictor state + Td-keyed tables + the encoder extension + 6 tier-A tests; [M], comparable to SPEC-037
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-11-05
      notes: "main-session design (un-metered); §12 probe parsing both real strips' SOF3+DHT+SOS (one interleaved scan, Ns=2, both H=V=1, one Td=0 table, predictor 1, Pt=0, no DRI) + the ljpeg extension seam (parse_frame_header/parse_scan_header/Frame/reconstruct/decode_into); acceptance + failing tests + HANDOFF-093 + build prompt"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-046: Two-component Nf2 SOF-3 decode and column de-interleave

## Context

The re-plan spec (DEC-037). SPEC-043's build probed the real SOF3 headers and found
both held Compression-7 DNGs are **two-component** (`Nf=2`) column-interleave SOF-3 —
Leica M Typ 246 (P=12, X=2992, Nf=2) and Pentax K-3 III (P=14, X=3152, Nf=2), each X ≈
half the crop width — not the single-component the framing assumed. The shipped
STAGE-009 decoder is `Nf=1` only and returns `LjpegUnsupportedComponentCount { nf: 2 }`
for both, so neither held file decodes and SPEC-044's bit-exact oracle has no target.
This spec grows the decoder to two-component column-interleave so both held files decode
to one monochrome `u16` plane. It is the new second spec of `STAGE-011` (`PROJ-003`),
inserted between SPEC-043 (routing, shipped) and SPEC-044 (the oracle).

## Goal

Extend `ljpeg` to decode a two-component (`Nf=2`) column-interleave SOF-3 frame — two
per-component predictor streams — and de-interleave the two column components back into
one `u16` plane (width `2 × samples_per_line`), so both held Compression-7 DNGs decode,
proven by an analytic two-component encode→decode round-trip with a red-proof. The
single-component (`Nf=1`) path stays byte-for-byte unchanged.

## Inputs

- **Files to read:**
  - `src/ljpeg.rs` — the shipped single-component decoder; the exact extension points:
    `parse_frame_header` (≈ line 376) rejects `nf != 1` → `LjpegUnsupportedComponentCount`
    and reads one component spec; `parse_scan_header` (≈ 493) rejects `ns != 1` and reads
    one component selector + `Ss`(predictor) / `Al`(Pt) / `Td`; the `Frame` struct (≈ 182:
    `precision`, `lines`, `samples_per_line`, `point_transform`, `predictor`,
    `restart_interval` — **no component count yet**); `reconstruct` (≈ 855, "out must hold
    `lines × samples_per_line`") and its per-sample predictor loop; `decode_into` (≈ 992,
    length check `out.len() == X·Y` → `LjpegPlaneWrongLength`); `predict` (T.81 Table H.1);
    `HuffTable` (one DC table today).
  - `src/plane.rs` — SPEC-043's Compression-7 route already sizes the strip `out` slice as
    `rows × sensor.width` (the FULL width = `Nf·X`); confirm the Nf=2 decode fills it.
  - `STAGE-011`, `DEC-037`, `brief.md`.
- **§12 probe (done at design, parsing the real strips' SOF3 + DHT + SOS):** both held
  files are **one interleaved scan, `Ns=2`**, identical structure —
  - Typ246: P=12, Y=4000, X(per-component)=2992, Nf=2. K3III: P=14, Y=4224, X=3152, Nf=2.
  - Both component specs **H=1 V=1**, Tq=0; a **single** DC Huffman table (Tc=0 Th=0),
    shared — both components reference **Td=0** in the SOS; `Ss`(predictor)=**1** (Ra = left
    neighbour), Se=0, Ah=0, **Al(Pt)=0**; **no DRI** (no restart).
  - So each MCU is one comp0 sample then one comp1 sample at the same grid position; the two
    components are the two column phases of the `2·X`-wide monochrome plane.
- **Related code paths:** `src/ljpeg.rs` (decoder), `src/plane.rs` (the route that calls it).

## Outputs

- **`src/ljpeg.rs` modified:**
  - `Frame` gains a component count (e.g. `components: u8`, 1 or 2).
  - `parse_frame_header`: accept `Nf ∈ {1, 2}` (read `Nf` component specs; `Lf == 8 + 3·Nf`);
    reject `Nf` outside `{1,2}` with `LjpegUnsupportedComponentCount` (Bayer/CFA 3+ stays
    rejected — DEC-037).
  - `parse_scan_header`: accept `Ns ∈ {1, 2}` (read `Ns` selectors, each with its `Td`;
    `Ls == 6 + 2·Ns`); require `Ns == Nf`; keep `Ss`(predictor) / `Al`(Pt) scan-level.
    Build up to two DC Huffman tables keyed by `Td` and have each component select its own
    (both held files use `Td=0` — one shared table — but do not hard-code "one table").
  - `reconstruct`: for `Nf == 2`, decode the **interleaved** scan — at each `(x, y)` decode
    comp0 then comp1 — with **per-component** predictor state (each component's own
    `Ra`/`Rb`/`Rc` neighbours and the §H.2.1 boundary / initial-prediction `2^(P−Pt−1)` rules
    applied independently), writing into a `2·X`-wide row in column-interleave order
    (`out[y·2X + 2x] = comp0`, `out[y·2X + 2x + 1] = comp1` — the natural MCU order). The
    `Nf == 1` path stays **byte-for-byte unchanged**.
  - `decode_into`: the length check becomes `out.len() == Nf · samples_per_line · lines`
    (`LjpegPlaneWrongLength` otherwise).
- **No new `Error` variant** (reuse `LjpegUnsupportedComponentCount` for `Nf>2`,
  `LjpegPlaneWrongLength`, and the existing `Ljpeg*`); `decode_into` stays `pub(crate)`;
  **no new runtime dependency**, no CLI change.
- **The in-harness encoder** (`encode_lj92` in `mod tests`) gains a two-component
  interleaved encode so the analytic round-trip covers Nf=2.
- **De-interleave caveat (record it):** the analytic two-component round-trip CANNOT
  confirm the real-file column-interleave *mapping* (encode and decode share the
  convention — `shared-path-oracle-tautology`); it proves the decode self-consistent and
  per-component prediction correct. The exact mapping (component order + column phase) on a
  real file is confirmed by **SPEC-044's `dnglab --raw-checksum`** oracle.

## Acceptance Criteria

- [ ] `parse_headers` accepts a two-component (`Nf=2`, `Ns=2`, both `H=V=1`) interleaved
  SOF-3 frame and sets `Frame.components = 2`; `Nf` outside `{1,2}` still returns
  `LjpegUnsupportedComponentCount`.
- [ ] `decode_into` on a hand-built two-component interleaved scan fills a `2·X × Y` `u16`
  plane **bit-exact** against the known samples, column-interleaved (`out[2x]=comp0`,
  `out[2x+1]=comp1`), each component using its own predictor-1 neighbour state; exercised at
  P=12 and P=14.
- [ ] The analytic **two-component encode→decode round-trip** is bit-identical on honest
  input and goes **RED** on a perturbed predictor / Huffman table / swapped-component encode
  (`oracle-must-be-shown-red`).
- [ ] The **single-component (`Nf=1`) path is byte-for-byte unchanged** — every shipped
  `ljpeg` test stays green, pinned by a characterization decode identical pre/post.
- [ ] `decode_into` returns a typed `Error`, never a panic, on a malformed two-component
  scan — `Ns ≠ Nf`, a component referencing an undefined `Td`, a scan truncated
  mid-second-component, and `out.len() != Nf·X·Y` (`LjpegPlaneWrongLength`).
- [ ] `#![forbid(unsafe_code)]` + the five-lint deny policy hold; `just test`, `just
  lint-ci`, `just fmt-check`, `just validate` green; no new runtime dependency.

## Failing Tests

Tier-A only (hand-built two-component SOF-3 via the extended encoder; no corpus). SPEC-044's
`dnglab` real-file check confirms the de-interleave mapping.

- **`src/ljpeg.rs` (`mod tests`):**
  - `"two_component_interleaved_decodes_bit_exact"` — encode a small known `2·X × Y` plane
    as a two-component interleaved scan (comp0 = even columns, comp1 = odd), decode, assert
    the `2·X`-wide plane is bit-identical; at P=12 and P=14.
  - `"two_component_round_trip_red_on_perturbed_predictor"` — mutating a predictor / the
    Huffman table / swapping the component order turns the round-trip RED (shown red, then
    reverted).
  - `"single_component_decode_unchanged"` — an `Nf=1` scan decodes identically to the
    shipped behaviour (characterization).
  - `"nf_three_still_unsupported"` — `Nf=3` → `LjpegUnsupportedComponentCount { nf: 3 }`.
  - `"two_component_wrong_out_length_is_typed_error"` — `out.len() != 2·X·Y` →
    `LjpegPlaneWrongLength`, no panic.
  - `"two_component_truncated_scan_is_typed_error"` — a scan truncated mid-second-component
    returns a typed `Error`, no panic.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The bit-exact `dnglab --raw-checksum` corpus oracle on the two real files — **SPEC-044**.
- The provenance-ledger entry, determinism, and container→decoder hardening — **SPEC-045**.
- The Compression-7 recognition + routing — **SPEC-043** (shipped); this spec only
  extends what `decode_into` accepts.
- **Bayer / CFA colour multi-component** (3+ colour components in one frame) — still out
  (DEC-037); no held colour file, no oracle.
- More than two components, or a non-column interleave the held files do not use.

## Notes for the Implementer

- **The interleave is regular** (both components `H=V=1`): each MCU is exactly one comp0
  sample then one comp1 sample, no subsampling. Decode order per row `y`: for `x` in
  `0..X` → decode comp0[x], decode comp1[x]; write `out[y·2X + 2x] = comp0`,
  `out[y·2X + 2x + 1] = comp1`.
- **Per-component predictor state is the crux.** Predictor 1 (Ra) is the *same component's*
  left neighbour — `comp_k[y][x-1]`, i.e. `out[y·2X + 2(x-1) + k]`, two output columns back.
  The §H.2.1 first-column / first-line / initial-prediction (`2^(P−Pt−1)`) rules apply
  **independently per component**. Leave the `Nf=1` path on its existing single-state code
  so it stays byte-for-byte identical.
- **Td lookup:** both held files share one DC table at `Td=0`, but build the DC table(s) by
  `Td` and have each component select its own — do not hard-code a single table.
- **`decode_into` length check keys on `Nf`:** `out.len() == Nf · samples_per_line · lines`.
  SPEC-043's route already sizes `out` as `rows × sensor.width` with `sensor.width == Nf·X`,
  so the binding is already compatible — confirm it, don't change the routing.
- **The de-interleave mapping is NOT provable by the analytic round-trip** (encode and
  decode share the convention — `shared-path-oracle-tautology`). This spec proves the decode
  self-consistent + per-component prediction; **SPEC-044's `dnglab --raw-checksum`** pins the
  exact component order + column phase on the real files. If SPEC-044 later shows the mapping
  is left/right halves rather than even/odd columns, that is a one-line change to the
  write-index, not a decode-logic change — note it, don't pre-engineer for it.
- **Scope:** `Nf=2` interleaved, `H=V=1`, Td-keyed DC table(s), `Pt=0`, no restart — exactly
  what both held files use (§12 probe in `## Inputs`). `Nf>2`, subsampled components
  (`H`/`V`≠1), and Bayer/CFA colour stay out (DEC-037). Don't touch SPEC-043's routing or the
  container.
- **Patent/clean-room:** still DEC-035 (T.81 Annex H, clean-room); two-component interleave
  is the same standard, no new algorithm family.

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
