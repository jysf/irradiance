---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-045
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   framed [S] (provenance + determinism + hardening); the
                                   #   external-review ljpeg header items (Pt<P, Tc/Th, later-DHT)
                                   #   and the public-path fuzz coverage push it to M. Several legs
                                   #   are already groundwork-covered (determinism at develop level
                                   #   by SPEC-044; two-component restart by SPEC-046; tiled already
                                   #   a clean UnsupportedStripLayout), so no single leg is large.
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8      # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-097 (build) → tier_map.build
  created_at: 2026-11-06

references:
  decisions: [DEC-036, DEC-003, DEC-035, DEC-018]   # retire-or-keep the dev-only fuzzing feature/hook once a public Compression-7 entry exists (DEC-036 Validation); corpus never committed / own-work seeds (DEC-003); clean-room T.81 for the header-hardening (DEC-035); u16 modular arithmetic (DEC-018)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-043, SPEC-044, SPEC-046, SPEC-038]   # the routing (043) + its bit-exact verification (044) this capstone closes; the two-component decode (046, whose restart already satisfies §2.3); the fuzz harness (038) this extends to the public path

# Blocking dependencies: specs that must SHIP before this one can start.
depends_on: [SPEC-044]             # the capstone ships after the binding (043) is bit-exact-verified (044)

claimed_by: null

value_link: "Closes the binding — a provenance-ledger entry, macOS/Linux determinism, container→decoder hardening, and continuous fuzzing of the public Compression-7 route — the capstone of STAGE-011 and PROJ-003."

cost:
  tokens_estimate: 16000000        # ~16M — an M capstone of small cohesive legs (several groundwork-covered); comparable to SPEC-044, a touch less
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 50
      recorded_at: 2026-11-06
      notes: "main-session design (un-metered); §12 probe found the determinism develop-level test already exists (SPEC-044, develop_oracle.rs), the two-component restart reset is already done + tested (SPEC-046), and tiled Compression-7 is already a clean UnsupportedStripLayout — so the NEW work is the provenance binding row, a plane-level determinism pin, the ljpeg header-hardening gaps (parse_scan_header does NOT check Pt<P; parse_dht does NOT reject Tc!=0/Th>3; find_table returns the FIRST same-Th table), and fuzzing the PUBLIC Compression-7 route (drop the require_uncompressed early-return in the plane/develop/demosaic fuzz targets + a synthetic SOF-3 seed built from tests/support/tiff.rs + encode_lj92), keeping the direct __fuzz_ljpeg hook (DEC-036 Validation). Acceptance + failing tests + HANDOFF-097 + build prompt."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-045: Binding capstone — provenance, determinism, and hardening

## Context

The capstone of `STAGE-011` (`PROJ-003`): the Compression-7 binding exists (SPEC-043)
and is proven bit-exact against `dnglab` (SPEC-044). Three closing obligations remain —
a provenance-ledger entry for the DNG Compression-7 **binding** (distinct from
STAGE-009's T.81 Annex H **decode** rows), a determinism assertion that the same
compressed DNG yields byte-identical plane output across macOS and Linux, and hardening
of the container→decoder boundary against truncated / hostile Compression-7 strips. The
external 0.2.0 review (`docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`) folds its
lossless-JPEG header items here: §2.4 (`Pt < P`, `Tc == 0 && Th <= 3`) and §2.1 (a later
DHT with the same `Th` must replace the earlier). With this spec, STAGE-011 and PROJ-003
close.

**The design §12 probe found three of the framed legs already groundwork-covered**, so
the new work is smaller than the frame implies:

- **Determinism** — SPEC-044 already added a develop-level byte-identity check on a held
  Compression-7 file (`tests/develop_oracle.rs`, "two develops of the same file are
  byte-identical"). This spec adds the **plane-level** counterpart and documents the
  integer-only / cross-platform basis.
- **§2.3 (two-component restart reset)** — already implemented and tested by SPEC-046
  (`reconstruct_two_component` resets both components at `RSTm`; the `ri = x` case in
  `two_component_interleaved_decodes_bit_exact`). No new work; pinned by the existing test.
- **Tiled Compression-7** — already a clean typed `UnsupportedStripLayout` (`src/plane.rs`
  rejects a strip-location mismatch / zero strips; tiles are a stated non-goal). Pinned by
  a test; no new code.

The genuine gaps the probe surfaced: `parse_scan_header` validates the predictor but **not
`Pt < P`**; `parse_dht` does **not** reject `Tc != 0` or `Th > 3`; `find_table` returns the
**first** same-`Th` table (§2.1); and the **public** Compression-7 route is not yet fuzzed
(the plane/develop/demosaic fuzz targets skip it via `require_uncompressed().is_err()`).

## Goal

Add the DNG Compression-7 **binding** provenance-ledger row and correct the now-stale
"compressed files are rejected, never decoded" note in the plane row; assert plane-level
byte-identical determinism for a held Compression-7 file; harden the lossless-JPEG header
(`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces) as typed errors, never panics; and
fuzz the **public** Compression-7 route (plane / develop / demosaic) continuously —
keeping the direct `__fuzz_ljpeg` hook for deep decoder coverage (DEC-036 Validation).
Close STAGE-011 and PROJ-003.

## Inputs

What the implementer will read or consume.

- **Files to read:**
  - `src/ljpeg.rs` — `parse_scan_header` (≈ 572): reads `al` (point transform) and validates
    `ss` (predictor) in `1..=7` but leaves `al` unchecked — add `al < precision`. `parse_dht`
    (≈ 456): computes `tc`/`th` and only builds `tc == 0` tables, silently ignoring other
    classes and any `th` — add `tc == 0 && th <= 3` validation. `find_table` (≈ 401): returns
    the **first** `(th, _)` match — change to the **last** (or have `parse_dht` overwrite an
    existing `th` entry) so a later DHT with the same `Th` wins. `precision` is on `Frame`
    and is read before SOS in `parse_headers`, so `parse_scan_header` needs it passed in (or
    validate `al < precision` in `parse_headers` after both are known).
  - `src/plane.rs` — the SPEC-043 Compression-7 route (`unpack_into` dispatch on
    `Compression`, the per-strip `ljpeg::decode_into`); the existing `UnsupportedStripLayout`
    / `UnsupportedCompression` typed rejections (tiles already rejected here).
  - `tests/support/tiff.rs` — the TIFF/DNG builder (`tiff`, `Ifd`, `Entry`, `long`,
    `cfa_dng_16bit` as the strip-bearing-DNG pattern). Add a `compression7_dng(...)` helper
    that wraps a SOF-3 strip (from the `cfg(test)`-reexported `encode_lj92`) in a
    `Compression = 7` DNG, for the hardening tests and the fuzz seed.
  - `tests/develop_oracle.rs` (the existing Compression-7 develop determinism check to mirror
    at plane level), `tests/plane_unpack.rs` (the AC7 hostile-input home + the SPEC-043
    `compression7_planes_route_to_the_decoder` test).
  - `fuzz/fuzz_targets/{plane,develop,demosaic}.rs` — each has
    `if sensor.require_uncompressed().is_err() { return; }` (plane ≈ 47) that **skips**
    Compression-7; `fuzz/fuzz_targets/ljpeg.rs` + `src/lib.rs` `__fuzz_ljpeg` (≈ 161, behind
    the `fuzzing` cargo feature, DEC-036) + `fuzz/seeds/plane/` (has `compressed-plane.tiff`,
    `truncated-strip.tiff`).
  - `docs/provenance-ledger.md` — the `src/plane.rs` row (SPEC-012/028) whose note "The three
    compressed corpus files … are rejected before any strip read … never decoded" is now
    **stale**; the two `src/ljpeg.rs` decode rows (SPEC-036 entropy, SPEC-037 reconstruction)
    whose "container binding … is STAGE-010/011" scope this binding row closes.
  - `STAGE-011`, `DEC-036`, `DEC-003`, `DEC-035`, `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`.
- **§12 probe (done at design — transcribe, don't re-derive):** the four gaps above with
  their line references; determinism's develop-level test and §2.3's restart test already
  exist; tiled is already `UnsupportedStripLayout`. `tests/support/tiff.rs` + `encode_lj92`
  make a synthetic Compression-7 container cheap.
- **Related code paths:** `src/ljpeg.rs`, `src/plane.rs`, `fuzz/`, `docs/provenance-ledger.md`.

## Outputs

What the implementer will produce.

- **`docs/provenance-ledger.md`:** a new row for the **DNG Compression-7 binding**
  (`src/plane.rs` route → `ljpeg` per SPEC-043 / SPEC-044 / DEC-039: `Compression == 7`
  recognition, per-strip dispatch, two-component column de-interleave, APPn/COM header
  tolerance; provenance class 1 — public TIFF/DNG + T.81 specs, `dnglab` run-as-tool only),
  and a correction to the `src/plane.rs` row's stale "compressed files … never decoded" note.
- **`src/ljpeg.rs`:**
  - `parse_scan_header` (or `parse_headers` once `precision` is known) rejects `Pt >= P` with
    a typed `Error` (reuse an existing `Ljpeg*` variant or add one additive variant — no panic).
  - `parse_dht` rejects `Tc != 0` and `Th > 3` with a typed `Error` (AC / lossless uses only
    DC class-0 tables, destinations 0..=3).
  - A later DHT with the same `Th` **replaces** the earlier (`find_table` returns the last
    match, or `parse_dht` overwrites the `th` entry); the shipped single-table and shared-`Td`
    two-component behaviour is unchanged.
  - No change to the reconstruction algorithm, the predictors, or the de-interleave.
- **`tests/support/tiff.rs`:** a `compression7_dng(order, …)` helper (SOF-3 strip from
  `encode_lj92` wrapped in a `Compression = 7` single-strip DNG).
- **`fuzz/`:** drop the `require_uncompressed().is_err()` early-return in
  `fuzz/fuzz_targets/{plane,develop,demosaic}.rs` so the **public** Compression-7 route is
  fuzzed; add a synthetic Compression-7 SOF-3 seed to `fuzz/seeds/plane/` (reused by
  develop/demosaic). **Keep** the `__fuzz_ljpeg` hook, the `fuzzing` cargo feature, and
  `fuzz/fuzz_targets/ljpeg.rs` (DEC-036 Validation — see Notes).
- **Tests** (see Failing Tests): plane-level determinism pin, the two header-hardening gaps
  and the later-DHT rule, a truncated/hostile Compression-7 strip through the **public**
  `plane::unpack_into` (typed error, no panic), a tiled Compression-7 rejection pin, and the
  §2.3 restart pin (cite the existing SPEC-046 test).
- **No new public API, no new runtime dependency**; `ljpeg` stays `pub(crate)`;
  `#![forbid(unsafe_code)]` + the five-lint deny policy hold.

## Acceptance Criteria

- [ ] `docs/provenance-ledger.md` carries a DNG Compression-7 **binding** row, and the
  `src/plane.rs` row's "compressed files … never decoded" note is corrected to reflect the
  SPEC-043 routing. `just validate` and any provenance gate stay green.
- [ ] A held Compression-7 file decodes to a **byte-identical** `u16` plane across two
  independent decodes (plane-level determinism), complementing SPEC-044's develop-level
  check; the integer-only / cross-platform basis is documented. Corpus-gated, skips loudly.
- [ ] `parse_scan_header`/`parse_headers` rejects `Pt >= P` with a typed `Error`, never a
  panic (tier-A, hand-built).
- [ ] `parse_dht` rejects `Tc != 0` and `Th > 3` with a typed `Error`, never a panic (tier-A).
- [ ] A later DHT with the same `Th` **replaces** the earlier — a two-table stream whose
  second `Th=0` table is the valid one decodes correctly; the shipped single-table and
  shared-`Td` two-component decodes are unchanged (tier-A).
- [ ] A **truncated / hostile** Compression-7 strip fed through the **public**
  `plane::unpack_into` (synthetic container) returns a typed `Error`, never a panic; a tiled
  Compression-7 plane is a clean `UnsupportedStripLayout`.
- [ ] The plane / develop / demosaic fuzz targets exercise the **public** Compression-7 route
  (the `require_uncompressed` early-return is gone and a Compression-7 seed is present); the
  direct `__fuzz_ljpeg` hook is retained; a short fuzz run on the Compression-7 seed is
  panic-free.
- [ ] `#![forbid(unsafe_code)]` + the five-lint deny policy hold; `just test`, `just lint-ci`,
  `just fmt-check`, `just validate` green; no new runtime dependency.

## Failing Tests

Tier-A runs everywhere; tier-B is corpus-gated (skips loudly by name).

- **`src/ljpeg.rs` (`mod tests`)** — tier A, hand-built via `encode_lj92`:
  - `"point_transform_at_or_above_precision_is_a_typed_error"` — a scan with `Al (Pt) >= P`
    returns a typed `Error`, no panic.
  - `"dht_with_nonzero_class_or_bad_destination_is_a_typed_error"` — `Tc != 0` (AC table) and
    `Th > 3` each return a typed `Error`, no panic.
  - `"a_later_dht_with_the_same_th_replaces_the_earlier"` — a stream with two `Th=0` DC tables
    (the first bogus, the second valid) decodes using the **second**; reverting to
    first-wins turns it red.
  - `"single_table_and_shared_td_two_component_decode_unchanged"` — characterization: the
    shipped one-table and two-component shared-`Td` decodes are byte-identical pre/post.
- **`tests/plane_unpack.rs`** — tier A (synthetic container via `compression7_dng`):
  - `"truncated_compression7_strip_through_unpack_into_is_a_typed_error"` — a Compression-7
    DNG whose SOF-3 strip is truncated mid-scan returns a typed `Error`, no panic.
  - `"tiled_compression7_plane_is_unsupported_strip_layout"` — a Compression-7 IFD with no
    `StripOffsets` (tiled) is a clean `UnsupportedStripLayout`.
- **`tests/plane_oracle.rs`** (or `develop_oracle.rs`) — tier B (corpus):
  - `"compression7_plane_is_byte_identical_across_two_decodes"` — a held Compression-7 file's
    `u16` plane is byte-identical across two independent `unpack_into` runs.
  - `"two_component_restart_is_pinned"` — reference the existing SPEC-046 restart coverage
    (no new decode, a pin/pointer) OR a small two-component-with-DRI round-trip if the
    implementer finds the existing coverage thin.

## Non-Goals

Explicit scope limits. If any of these needs doing, write a new spec, don't expand this one.

- `#[non_exhaustive]` on `Sensor` / `Cfa` / `WarpRect` / `Opcode` (external review §4) —
  **PROJ-002 0.2.0 release (SPEC-042)**, semver-scoped, not a STAGE-011 binding concern.
- The opcode-list `MAX_TAG_VALUES` bound (§4) — PROJ-001 opcode subsystem, a different path.
- `LinearizationTable` typed rejection (§4) — a cross-cutting plane concern (applies to
  uncompressed too), and no held Compression-7 file carries one; its own follow-up.
- `tag_name` in `Error` `Display` (§4) — cosmetic error-message polish; its own tiny follow-up.
- **Retiring** the `__fuzz_ljpeg` hook / `fuzzing` feature — explicitly **kept** (DEC-036
  Validation; see Notes). Reversing that is a future decision, not this spec.
- The SOF-3 reconstruction algorithm (STAGE-009 / SPEC-046), the routing (SPEC-043), the
  bit-exact oracle (SPEC-044) — shipped; this spec changes none of them.
- Bayer / CFA colour multi-component, OpcodeList2 GainMap — out (DEC-037).
- The external review's §3B colour geometry (PROJ-002) and §5 perf (its own stage).

## Notes for the Implementer

- **DEC-036 Validation — keep the direct hook.** The frame said the `fuzzing` feature +
  `__fuzz_ljpeg` hook "may be retired once the real public entry exists." The design decision
  is to **keep** them and **add** public-path coverage: the direct hook fuzzes `decode_into`
  on arbitrary bytes (deep decoder states), which the container-gated public route reaches far
  less efficiently; the hook is `#[doc(hidden)]`, feature-gated, and absent from the published
  crate, so it is zero public surface. A hardening capstone should not shrink fuzz coverage.
  Record this reasoning in the handback (it dispositions DEC-036's open Validation clause; no
  new DEC).
- **The header-hardening is straight T.81 conformance** (Annex B/C), clean-room (DEC-035): a
  lossless scan's `Pt` must be `< P`, DC Huffman tables are class 0 destinations 0..=3, and a
  redefined table wins — all parse-level guard clauses returning typed errors, no algorithm
  change. Prefer reusing an existing `Ljpeg*` error variant; add at most one additive variant
  if none fits (the enum is `#[non_exhaustive]`).
- **Don't widen scope on the fuzz targets:** drop only the `require_uncompressed` early-return
  and add the Compression-7 seed; the uncompressed path they already fuzz stays.
- **The synthetic Compression-7 container** is cheap: `encode_lj92` (cfg(test)) emits the
  SOF-3 strip; `tests/support/tiff.rs`'s `tiff`/`Ifd`/`long` build the `Compression = 7` DNG
  around it (mirror `cfg_dng_16bit`). One strip, single component, is enough for the hardening
  tests and the seed.
- **Determinism** is asserted the way the repo already does it (DEC-002 / SPEC-044): two
  independent decodes byte-identical, plus the integer-only (float-free) design argument and
  the pinned `dnglab` checksum as the cross-implementation ground truth. The decoder is
  integer-only by construction — assert it, don't just assume it.
- **This is the last STAGE-011 spec.** After it ships, run Prompt 1d (Stage Ship) for
  STAGE-011, then Prompt 1e (Project Ship) for PROJ-003.

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
   real, greppable result, not a blank.
   — <answer | none>
