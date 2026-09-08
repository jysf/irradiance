---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-017
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
                                   # ⚠ RAISED from the frame stub's `medium`.
                                   # The Q2M's OpcodeList1 carries Flags=0 —
                                   # NOT optional. A decoder that produces
                                   # correct-looking pixels while silently
                                   # ignoring a mandatory opcode is shipping a
                                   # promise it does not keep, and it will not
                                   # be seen from output alone (STAGE-003's
                                   # note: a no-op opcode and an unexecuted
                                   # opcode are indistinguishable from the
                                   # output). Not `critical` — SPEC-018 keeps
                                   # that seat; missing this one degrades a
                                   # small fraction of pixels, not 6% of the
                                   # image width.
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   The stage estimated S; kept. Two
                                   #   surfaces: (i) OpcodeList byte-stream
                                   #   parser (new attack surface,
                                   #   panic-free-on-untrusted-input, its own
                                   #   fuzz target — §12 bar 2 fires) and (ii)
                                   #   the FixBadPixelsConstant applier itself
                                   #   (~80 lines: iterate plane, detect
                                   #   pixels == Constant, replace with 3×3
                                   #   median-of-neighbours). The applier is
                                   #   the smallest possible unit; the parser
                                   #   is the first stroke of infrastructure
                                   #   SPEC-018 extends.
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: punch-list  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-002, DEC-004, DEC-005, DEC-011, DEC-016, DEC-018]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, library-not-application, test-before-implementation]
  related_specs: [SPEC-003, SPEC-012, SPEC-014, SPEC-018, SPEC-020]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-020]             # SPEC-017 is a develop-pipeline mutation;
                                   # SPEC-020's oracle is the check that this
                                   # change did not make the score worse. See
                                   # AC7. SPEC-018 is a related_spec, not a
                                   # depends_on: whichever of SPEC-017/018
                                   # builds second extends src/opcode.rs
                                   # (Notes for the Implementer).

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-003's <capability>". Optional; null is acceptable.
value_link: "closes the smallest of STAGE-003's three opcode-shaped promises —
  the sensor's known-dead pixels are replaced, not passed through unchanged.
  Foundational: the same OpcodeList byte-stream parser lands here and extends
  for SPEC-018's WarpRectilinear."

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
  tokens_estimate: 60000000
  # Basis: SPEC-012 (strip location + sample unpack, shipped M) landed at
  # ~55M with one punch-list round. SPEC-017 is a smaller applier but adds
  # a fuzz target and the parser scaffolding SPEC-018 extends; comparable
  # scope, similar cost. 60M assumes build + verify + one punch-list round.
  # If it lands near 90M, the OpcodeList parser turned out to be more work
  # than "the smallest possible parser" implies — a signal SPEC-018's
  # tokens_estimate (130M) should be re-examined.
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 570000
      estimated_usd: 4.10
      duration_minutes: 90
      recorded_at: 2026-09-07
      notes: "AC5 measured HIT=0 on all 3 Q2M frames (SB-1, real finding); AC1-4/6/8/9 green, AC7 green; SPEC-018 landed opcode.rs first, extended it; see handback narrative"
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 27235932
      estimated_usd: 65.83
      duration_minutes: 30
      recorded_at: 2026-09-07
      notes: "PUNCH LIST on cd82ca8 (tip 5c1ba36 bookkeeping-only). CI run 34191738033 observed green 11/11 via gh, including fuzz smoke - opcode whose own CI log says Done 11549674 runs in 61 second(s), 0 crashes (the 13.7M figure is the BUILD LOCAL run, not CI). SB-1 DOWNGRADED to FU-3: independently verified with a C 14-bit unpacker written from the DNG packing rule (7 bytes -> 4 samples, MSB-first), which reproduces irr unpack first8 and minima exactly on all three frames - ZEROS inside ActiveArea = 0 and whole-plane minima 2/30/2, so the applier correctly returns 0 and there is NO code bug; AC5 rerun with --ignored reproduces constant=0 HIT-count=0 left-at-constant=0. NEW SB-2, SHIP-BLOCKING: nothing asserts develop_into USES the fixed plane - severing effective_src to src (md5 b3b922d0 -> 548e240f) compiles and leaves the whole tier-A suite green at 231 passed / 0 failed / 3 ignored, and tier-B is blind too, MEASURED not assumed: the mutant AC7 PASSES 1/0/0 on BOTH comparable frames - before=-60.169 after=-60.169 delta=+0.000 on L1021223 and before=-40.281 after=-40.281 delta=+0.000 on L1026192, both byte-identical to the honest tree's own numbers, L1026016 skipped for the orientation reason - because HIT=0 makes before and after the same image. Identical shape to SPEC-018/SB-1 and a direct hit on STAGE-003 the test must assert the branch was HIT, not merely that the image came out unchanged. One-line fix: assert dst1 centre == 1000 in develop_output_is_bit_identical_across_two_runs, whose fixture already makes normalize an identity map. Bar 9 discharged PERSONALLY, both directions: md5 b3b922d0 honest -> 74af14a4 mutant, compiles, AC4 RED at left 0 right 1000 -> reverted b3b922d0, AC4 green; all mutation in a scratchpad copy, working tree never touched and still clean. Bar 10 pass. Bar 11 passes on substance - class 1 verified by reading the DNG 1.6.0.0 PDF myself with pdftotext: Chapter 7 Opcode List Processing, FixBadPixelsConstant Opcode ID 4, DNG Version 1.3.0.0, params Constant/BayerPhase LONG, p.95, and Chapter 6 is Mapping Camera Color Space to CIE XYZ Space, so the build chapter correction is RIGHT - but the row is filed under src/opcode.rs while the median kernel lives in src/develop.rs, whose own row never mentions it (FU-5). Bar 12 pass: dependencies empty, cargo tree -e normal is irradiance v0.1.0 alone, Cargo.toml 0 lines changed. lint-ci green on PINNED clippy 0.1.98 (local unpinned 0.1.97), fmt clean, deny + deny-fuzz both licences ok, full suite 231/0/3 with corpus present. Build FU-1 confirmed accurate against the shipped parser. Build FU-2 is a RE-INSTANCE of SPEC-018/FU-6, reproduced: dnglab --srgb prints 8368 5584 for BOTH an Orientation-6 and an Orientation-1 frame, i.e. it ignores Orientation entirely; already absorbed by SPEC-021 Context, no new spec needed. FU-4 false rustdoc: every hand-built test Sensor carries opcode_list_1 None is wrong - AC9 own test sets Some and takes the copy path. FU-6 peak RSS measured 541261824 bytes post-SPEC-017 vs the ledger 465010688 the module doc points readers at. FU-7 in-place neighbour semantics undocumented and spec-silent, measured in-place by probe. FU-8 AC3/AC5/AC10 spec text now stale vs shipped reality. FU-9 tokens_total 570000 is a remaining-budget delta, not comparable with this repo transcript-sum figures (SPEC-018 build rounds were 116M/14.3M/5.6M) - evidence for token-counts-not-comparable. PR not opened, nothing repaired, handback-sync not run."
  totals:
    tokens_total: 27805932
    estimated_usd: 69.93
    session_count: 2
---

# SPEC-017: FixBadPixelsConstant opcode

## Context

**Probed 2026-09-06 on all three decodable Q2M frames**
(`L1021223.DNG`, `L1026016.DNG`, `L1026192.DNG` under
`$IRRADIANCE_CORPUS_DIR/LEICA-Q2-MONO/`) with
`exiftool -b -OpcodeList1 <file> | xxd`. Every frame carried the
**same 28-byte OpcodeList1** payload, byte-for-byte:

```
00000000: 0000 0001 0000 0004 0103 0000 0000 0000
00000010: 0000 0008 0000 0000 0000 0002
```

Parsed against DNG 1.7.0.0 § Chapter 6 (opcode list — cited in
`src/ifd.rs:129`):

| Field | Bytes | Value | Meaning |
|---|---|---|---|
| Opcode count | `00 00 00 01` | 1 | one opcode |
| OpcodeID | `00 00 00 04` | 4 | FixBadPixelsConstant |
| DNGVersion | `01 03 00 00` | 1.3.0.0 | required decoder ≥ 1.3.0.0 |
| Flags | `00 00 00 00` | 0 | **NOT optional** |
| VariableLengthBytes | `00 00 00 08` | 8 | 8-byte parameter block |
| Constant | `00 00 00 00` | 0 | dead-pixel marker: value 0 |
| BayerPhase | `00 00 00 02` | 2 | (see below) |

Two facts the byte-level probe settles:

**Flags = 0 makes this a required opcode.** DNG § Chapter 6's Flags
semantics (bit 0 = optional-for-preview): with bit 0 clear, a
compliant decoder MUST apply the opcode or refuse the file. A
library that develops the plane without applying this opcode
produces a technically non-compliant render, and — since the Q2M
sensor really does have dead pixels near value 0 — a visibly
freckled one on any flat area.

**BayerPhase = 2 on a monochrome sensor.** DNG defines BayerPhase
for CFA sensors (0..3); on `SamplesPerPixel = 1 / Linear Raw` the
field is meaningless. The Q2M's DNG writer emits `2` regardless.
The applier reads the field but does not branch on it in the
monochrome case — see AC3 and the Notes below.

⚠ **The failure mode the STAGE-003 note guards against.** The stage
design explicitly says: *"The test must assert the branch was HIT,
not merely that the image came out unchanged — a no-op opcode and
an unexecuted opcode are indistinguishable from the output alone."*
This spec's primary AC (AC4) is a **replaced-pixel count**, not a
per-pixel value equality. On the Q2M, the count is unknown at design
time — measure and report — but it MUST be greater than zero on a
real frame, or the algorithm is not being applied.

## Goal

Parse an `OpcodeList1` byte stream, recognise `FixBadPixelsConstant`
(opcode ID 4) per DNG 1.7.0.0 § Chapter 6, and replace every plane
pixel equal to `Constant` with the median of its 3×3
non-bad-pixel neighbours — before levels normalisation runs
(`src/develop.rs`). Panic-free on adversarial opcode bytes, tested
by a replaced-pixel count (not an output equality), and shipped
with the first `src/opcode.rs` module SPEC-018 will extend.

## The design decision this spec rests on

⚠ **A HIT-count is the acceptance criterion, not an output diff.**
On any real Q2M frame this reduces to: after apply, the count of
pixels replaced must be strictly greater than zero — measured in
build, recorded in the handback, and asserted in the test. Two
subtler mutations are then observable:

- **Opcode not applied at all** → count == 0, test red.
- **Opcode applied with the wrong Constant** (e.g. hardcoded 65535
  instead of the value read from the opcode stream) → count == 0
  on real frames but > 0 on a spec-crafted tier-A fixture with a
  pixel at value 65535, test red on the tier-A fixture.

An output-equality test could pass on either mutation if the
downstream stages (levels, orientation) mask small local changes
into the tolerance. A COUNT cannot.

## Where the opcode module lives — coordination with SPEC-018

SPEC-018 also lands `src/opcode.rs`, for `OpcodeList3`
(WarpRectilinear). Whichever spec builds SECOND extends the module
the first created:

- **SPEC-017 → module skeleton.** `parse_opcode_list(bytes) ->
  Result<Vec<Opcode>, Error>` where `Opcode` is an enum whose
  first variant is `FixBadPixelsConstant { constant: u32,
  bayer_phase: u32 }`. `Opcode::Unknown { id, flags, params:
  Vec<u8> }` is the escape hatch that lets SPEC-018 (and any
  future opcode) extend without touching this spec's module.
- **SPEC-018 → adds `WarpRectilinear` variant.** Adds a variant to
  the enum, adds parsing logic, does not modify the module's error
  type without a `DEC-*` recording the change.

If SPEC-018 builds first (its `depends_on: [SPEC-020]` puts it in
the same waiting room as SPEC-017), it creates `src/opcode.rs`
with its own initial shape and SPEC-017 extends. Either direction
works; the handback names which happened.

**This is deliberate.** A separate `SPEC-021: OpcodeList parser
foundation` would drag both dependents by a shipping cycle each,
and the shape of "the opcode list parser" is small enough that
whichever spec lands first can hand it forward without a decision
record. Recorded here so a reviewer does not surface it as a
follow-up: the coordination is designed, not accidental.

## Inputs

- **The DNG 1.7.0.0 specification, § Chapter 6 "Opcode Lists"** —
  the authoritative source for the opcode-stream endianness
  (big-endian, unlike the TIFF payload the rest of the library
  handles), the FixBadPixelsConstant parameter layout, and the
  Flags semantics. Read this before writing the parser. The
  probed 28-byte payload in `## Context` is the anchor test
  case, not a substitute for the spec.
- **`docs/measured-q2m-dng.md`** — the OpcodeList1 line and the
  pipeline order (opcodes run on the raw plane before levels
  normalisation).
- **`decisions/DEC-002-*.md`** — no `rayon` on the algorithmic
  path, output determinism pinned within a `develop_version`. The
  bad-pixel replacement is on the algorithmic path.
- **`decisions/DEC-016-*.md`** — caller-owned-buffer shape used by
  `develop_into` / `unpack_into`. The applier mutates the u16
  plane in place; no new allocator.
- **`decisions/DEC-018-*.md`** — the u16 full-scale representation
  with clamped levels. The replacement pixel is a u16; the median
  is computed and rounded at the u16 boundary.
- **`src/plane.rs`** — `unpack_into` writes the uncropped,
  un-normalised u16 plane the applier receives.
- **`src/ifd.rs`** — the `Sensor` type carries `OpcodeList1`
  presence (line 550: "Presence only — decoding the opcode ..."
  is what SPEC-017 delivers). The parser reads from a byte slice
  the container hands over; the shape of "does `Sensor` hold the
  bytes eagerly or lazily?" is chosen in build (see `##
  Implementation Context`).
- **`src/develop.rs`** — `develop_into` is the integration point.
  The FixBadPixelsConstant applier runs on the raw plane BEFORE
  the levels normalisation pass.
- **A real Q2M file** (`IRRADIANCE_CORPUS_DIR/LEICA-Q2-MONO/*.DNG`)
  — for the tier-B replaced-pixel count. The three decodable
  frames all carry the same OpcodeList1 bytes (probed in `##
  Context`); at least two are needed for AC5.

## Outputs

- **`src/opcode.rs`** — new module. The opcode-list byte-stream
  parser: reads the count, iterates opcodes, extracts
  FixBadPixelsConstant's parameter block for `OpcodeID == 4`, and
  returns `Opcode::Unknown { id, flags, params }` for anything
  else. Panic-free (typed errors, bounds-checked slice reads,
  `checked_*` arithmetic — `no-panics-on-untrusted-input`).
  Small `pub` surface: `parse_opcode_list(bytes: &[u8]) ->
  Result<Vec<Opcode>, Error>` plus the `Opcode` enum and its
  error variant. **See "Where the opcode module lives" above:
  SPEC-018 extends this module.**
- **`src/develop.rs`** — modified. `develop_into` gains an
  `OpcodeList1` application step **before** `normalize`. The
  step calls `apply_fix_bad_pixels_constant(plane_mut, constant,
  active_area) -> Result<usize, Error>` where the `usize` is the
  **replaced-pixel count** — surfaced through the develop return
  value or a companion function (build chooses; state which in
  the handback) so tests can assert on it.
- **`src/ifd.rs`** — modified. `Sensor` gains an `opcode_list_1:
  Option<Vec<u8>>` field, populated from the IFD tag at the same
  read path SPEC-014's tags come through. **The bytes are held
  eagerly**; the parse into `Vec<Opcode>` happens inside
  `develop_into`. That satisfies AGENTS.md §11's "unread field"
  rule: `opcode_list_1` is read by `develop_into` in the same
  change.
- **`Cargo.toml` (root)** — no new dependencies. The 3×3 median
  and the u32 parsing are hand-written.
- **`fuzz/fuzz_targets/opcode.rs`** — new fuzz target on the
  opcode-list byte-stream parser. Seeded from the probed Q2M
  bytes (28 bytes above) plus three hand-crafted variants (see
  `## Implementation Context`).
- **`fuzz/Cargo.toml`** — one new `[[bin]]` entry for `opcode`.
- **`examples/fuzz-seeds.rs`** — extend the seed generator to
  emit `fuzz/seeds/opcode/*`. `just fuzz-seeds` already runs
  this and now regenerates the new target's seeds too.
- **`app.just`** — one new recipe `fuzz-opcode` following the
  shape of `fuzz-plane` / `fuzz-develop` (the `+toolchain` trap
  applies — `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo
  +nightly fuzz run opcode ...`), and one line in AGENTS.md §6's
  code block naming the new recipe (§6's rule 8: every recipe's
  commands appear in the block).
- **`tests/develop.rs`** and **`tests/opcode.rs`** (new) — the
  failing tests below. `tests/develop.rs` gains the
  replaced-count and end-to-end assertions; `tests/opcode.rs`
  covers the parser in isolation and the applier's algorithm.
- **`docs/provenance-ledger.md`** — **one row**: FixBadPixels-
  Constant applier and the OpcodeList parser (class 1 — DNG
  1.7.0.0 spec § Chapter 6). Class 1, published spec.
- **A `DEC-*`** for the replaced-count surfacing shape only if
  build finds a non-obvious choice. If the choice is "the
  applier returns `Result<usize, Error>` and `develop_into`
  ignores the count for its return but a companion
  `develop_into_with_stats` returns it" or similar, that is
  behind the u16 boundary and worth recording. If it is "add a
  second parameter to `develop_into` that receives an `&mut
  DevelopStats`", record why per DEC-016.

## Acceptance Criteria

- [ ] **AC1 — parser round-trips the probed Q2M OpcodeList1
      bytes.** Given the exact 28 bytes recorded in `## Context`,
      the parser returns
      `[Opcode::FixBadPixelsConstant { constant: 0, bayer_phase: 2 }]`
      — one opcode, the two u32s equal to the values in the
      byte table. Tier A, from a hex fixture committed at
      `tests/oracle-fixtures/opcodelist1-q2m.hex`.
      **Test:** `parse_opcode_list_reads_the_q2m_fixbadpixels_bytes`.
- [ ] **AC2 — parser is panic-free on adversarial input
      (`no-panics-on-untrusted-input`).** No `unwrap` / `expect`
      / `panic` / indexing-that-can-fault on any parser path.
      Verified by (i) clippy's SPEC-006 lint policy forbidding
      these on the library, and (ii) the fuzz target `opcode`
      runs 60 s local during build with no crashes (CI runs the
      smoke — same shape as `fuzz-plane` / `fuzz-develop`).
      Seed corpus: the 28 real bytes plus at least three
      hand-crafted variants — a truncated header (count present,
      body missing), an unknown opcode id with `Flags = 0`
      (Optional bit clear), and a length prefix that overflows.
      **Test:** `opcode_parser_fuzz_seeds_do_not_panic`
      (asserts each seed parses to `Ok(...)` OR `Err(...)`,
      never a panic — tier A, runs without `cargo fuzz`).
- [ ] **AC3 — parser records `Opcode::Unknown` for unknown IDs
      and preserves their bytes.** A stream containing opcode
      ID 99 with a 4-byte payload returns `[Opcode::Unknown
      { id: 99, flags: 0, params: [b0, b1, b2, b3] }]` for
      round-tripping / future dispatch. The
      applier's `develop_into` step then treats any
      `Opcode::Unknown` with **Flags & 1 == 0** (mandatory) as
      an error (see also AC6) and any with Flags & 1 == 1 as
      a warning-worthy skip. This is the design decision that
      lets SPEC-018 extend without editing this spec's parser.
      **Test:** `opcode_parser_returns_unknown_for_ninetynine`.
- [ ] **AC4 — `apply_fix_bad_pixels_constant` returns a
      replaced-pixel count.** Signature:
      `apply_fix_bad_pixels_constant(plane: &mut [u16],
      width: u32, height: u32, active_area: ActiveArea,
      constant: u32) -> Result<usize, Error>`. On a
      **hand-built** 5×5 plane with one pixel at `value == 0`
      surrounded by pixels at `value == 1000`, the result is
      `Ok(1)` and the plane's centre pixel is now `1000` (the
      median of eight identical neighbours).
      **Test:** `apply_replaces_one_isolated_bad_pixel_with_median`
      (tier A).
- [ ] **AC5 — HIT count on real Q2M frames is strictly positive
      (the STAGE-003 discipline).** On each of the three
      decodable frames, running the applier with
      `constant = 0` returns a count `> 0`. The three counts
      are **recorded in the handback**, per frame, as evidence
      the opcode was actually applied. If any frame's count is
      zero the applier is not being reached, or `constant` is
      being read from the wrong place — a defect, not a
      threshold to relax.
      **Test:** `q2m_frames_have_at_least_one_replaced_pixel`
      (tier B — the corpus is required; skip loudly when absent
      per SPEC-002).
- [ ] **AC6 — `develop_into` errors when it hits an
      `Opcode::Unknown` with mandatory Flags.** A hand-built
      OpcodeList1 with opcode ID 99 and `Flags = 0` returns a
      typed error naming the unknown ID from `develop_into`.
      Same list with `Flags = 1` returns success, applier
      count 0.
      **Tests:** `develop_errors_on_mandatory_unknown_opcode`
      and `develop_skips_optional_unknown_opcode` (tier A).
- [ ] **AC7 — the SPEC-020 develop-oracle score is at least
      as good with FixBadPixelsConstant applied as without.**
      On each of the three decodable Q2M frames, the SSIMULACRA2
      score against `dnglab analyze --srgb` must be **at least as
      high** as the pre-SPEC-017 baseline (which SPEC-020 records
      before this spec builds). Reasoning: dnglab's `--srgb`
      applies the same opcode, and matching it can only close
      the gap — a decrease means the applier is doing the wrong
      thing. The three deltas are recorded in the handback.
      **Test:** `q2m_develop_oracle_score_not_worse_after_fixbadpixels`
      (tier B).
- [ ] **AC8 — the red-proof: mutating the applier to skip the
      replacement turns AC4 red WITHOUT the corpus
      (`oracle-must-be-shown-red`).** A test that mutates a TEST
      COPY of the applier (or applies a `#[cfg(test)]` shim)
      so that the "replace with median" branch is a no-op,
      then asserts the mutated version returns `Ok(0)` OR the
      plane's centre pixel stays at `0` — either observation
      proves the algorithm change is caught. Tier A; runs
      with `IRRADIANCE_CORPUS_DIR` unset.
      **Test:** `red_proof_no_op_applier_leaves_bad_pixel_at_zero`.
- [ ] **AC9 — no `rayon`, no runtime SIMD dispatch, output
      determinism pinned within `develop_version` (DEC-002).**
      The applier runs single-threaded. Verified by (i)
      `Cargo.toml` has no `rayon`, (ii) no
      `is_x86_feature_detected` / runtime feature-dispatch
      call in `src/opcode.rs` or the applier, and (iii)
      running `develop_into` twice on the same input produces
      bit-identical output including the same replaced-pixel
      count.
      **Test:** `develop_output_is_bit_identical_across_two_runs`
      (tier A).
- [ ] **AC10 — provenance row for the FixBadPixelsConstant
      applier and the OpcodeList parser.** Single row in
      `docs/provenance-ledger.md`: module `src/opcode.rs`,
      source *DNG 1.7.0.0 specification § Chapter 6 (Opcode
      Lists; FixBadPixelsConstant, OpcodeID 4)*, class 1
      (published specification). Honest per §16 rule 3.
- [ ] **AC11 — eleven gates + `just lint-ci` + `just
      fuzz-opcode`, CI observed green on the shipping SHA.**
      The `fuzz-opcode` recipe is new; its 60 s smoke run must
      be added to CI in the same PR (per §12 bar 2: fuzz
      targets arrive with the parser, not retrofitted).

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target,
sum across all.

- `parse_opcode_list_reads_the_q2m_fixbadpixels_bytes` — AC1, tier A
- `opcode_parser_fuzz_seeds_do_not_panic` — AC2, tier A
- `opcode_parser_returns_unknown_for_ninetynine` — AC3, tier A
- `apply_replaces_one_isolated_bad_pixel_with_median` — AC4, tier A
- `q2m_frames_have_at_least_one_replaced_pixel` — AC5, tier B
- `develop_errors_on_mandatory_unknown_opcode` — AC6, tier A
- `develop_skips_optional_unknown_opcode` — AC6, tier A
- `q2m_develop_oracle_score_not_worse_after_fixbadpixels` — AC7, tier B
- `red_proof_no_op_applier_leaves_bad_pixel_at_zero` — AC8, tier A
- `develop_output_is_bit_identical_across_two_runs` — AC9, tier A

## Non-Goals

- **Any opcode other than FixBadPixelsConstant.** `OpcodeList3`'s
  WarpRectilinear is `SPEC-018`; `OpcodeList2` is empty on Q2M and
  not addressed here. `FixBadPixelsList` (opcode ID 5) is a
  different opcode not present in any decodable frame; its parser
  variant is not written until a corpus file needs it. Unknown
  opcodes are dispatched via `Opcode::Unknown` per AC3.
- **A Bayer-aware neighbour selection.** BayerPhase is read (AC1)
  but not branched on — the Q2M is monochrome, and any Bayer-aware
  branch would ship as unread code. When PROJ-002's first Bayer
  camera lands, that spec adds the branch (and the read).
- **A general-purpose median-of-N routine.** The 3×3 median is
  hand-written for this one opcode; if SPEC-019 or a later spec
  needs medians, it can share this or grow its own.
- **Bit-identity with dnglab's applied plane.** `--raw-checksum`
  attaches BEFORE all opcodes (per `docs/oracle-contract.md`); it
  cannot see this change. `--srgb` attaches AFTER develop; AC7's
  "not worse" is the check, not "bit-identical" — for the same
  reason SPEC-018 rejects bit-identity (single-sourced oracle).
- **A crustyimg-side integration probe.** SPEC-018 already carries
  this note for STAGE-004; SPEC-017 shares it.

## Notes for the Implementer

- **The opcode stream is BIG-ENDIAN**, unlike the TIFF payload the
  reader already handles (little-endian on the Q2M per DEC-008).
  SPEC-018's Notes carry the same warning; this is the parser's
  first defect trap. Write one round-trip test first, on the AC1
  hex fixture, and treat any parser change that alters its output
  as a defect.
- **Panic-free applies to the WHOLE parser, not just the entry
  point.** Slice reads use `.get(range).ok_or(err)?`; `usize`
  conversions use `.try_into().map_err(err)?`; multiplication that
  could overflow uses `checked_mul` (parameter counts CAN be
  attacker-controlled). SPEC-003's `ifd.rs` is the precedent.
- **The 3×3 median with fewer than 8 valid neighbours.** At the
  plane edge, or when the bad pixel is adjacent to *another* bad
  pixel, the window shrinks. Rules (pre-registered here so build
  is not making them up under time pressure):
  1. Neighbours outside the plane extent are excluded.
  2. Neighbours whose value == `constant` are excluded (they are
     themselves bad).
  3. If ≥ 1 valid neighbour remains, replace with the median of
     the valid ones (odd count → the middle; even count → the
     lower of the two middles, i.e. deterministic tie-break).
  4. If 0 valid neighbours (a bad pixel surrounded entirely by
     bad pixels or the plane edge), leave the value at
     `constant` and count it as "replaced but no valid median".
     Record that sub-count separately in the handback; do NOT
     silently pass. AC5's "count > 0" refers to the count in
     rules 1–3 combined.
- **Where the applier reads `active_area`.** For the Q2M,
  `active_area = 0 0 5632 8392` and the plane is `5632 × 8424`
  — the padding columns `8392..8424` are outside the active
  area. Whether to run the fix on the padding is a spec-silence
  point: pre-register `run inside active_area only` (padding is
  not a real image region and matching dnglab's boundary is
  reasonable). If a Q2M frame or a future camera shows dead
  pixels in the padding that dnglab visibly fixes, that is a
  finding, not a threshold to relax.
- **The tier-B corpus test skips loudly when the corpus is
  absent.** AC5's count assertion is on the real frames; AC8 is
  the tier-A red-proof CI can see.
- **`just lint-ci`, not `just lint`.** Homebrew's clippy is
  0.1.97; CI floats at 0.1.98 — the divergence has cost this repo
  17 consecutive red runs.
- **The fuzz target's PATH= prefix is the `+toolchain` trap in a
  new suit.** `~/.cargo/bin/cargo +nightly fuzz run opcode ...`
  alone fails (the INNER `cargo build` inside `cargo fuzz` still
  resolves to Homebrew's stable). Match the shape of `just
  fuzz-plane` in `app.just`.
- **Coordinate on `src/opcode.rs` with SPEC-018.** If SPEC-018
  built first (its `depends_on: [SPEC-020]` puts it in the same
  queue), the module already exists. Extend the `Opcode` enum
  with `FixBadPixelsConstant` and add the parser branch — do NOT
  rewrite the module. If SPEC-017 built first, name that in the
  handback so SPEC-018's build knows.
- **AC5's counts are DATA, not diagnostics.** Report the three
  numbers in the handback; those are the first real evidence
  this repo has of how many dead pixels a Q2M carries, and
  future decisions (Should the applier warn on unusually high
  counts? Should the count be part of an image metadata surface?)
  will read them.

## Implementation Context

> This section carries the design-time probes required before
> build (AGENTS.md §12 "Design-time probe / measure-before-build").
> The byte-level facts are measured, not asserted.

### Confirmed in design (probed 2026-09-06)

- **The 28-byte OpcodeList1 payload** for all three decodable
  Q2M frames is byte-identical (see `## Context` for the hex
  and the parsed field values). Reproduce with:

  ```
  exiftool -b -OpcodeList1 <file>.DNG | xxd -c 16
  ```

  on `L1021223.DNG`, `L1026016.DNG`, `L1026192.DNG` under
  `$IRRADIANCE_CORPUS_DIR/LEICA-Q2-MONO/`.

- **The opcode ID for FixBadPixelsConstant is 4**, not 5. The
  probe settled this; do not carry a from-memory "opcode 5"
  through build. DNG 1.7.0.0 § Chapter 6 assigns IDs; verify
  against the actual spec in build.

- **The pipeline order.** `OpcodeList1` runs on the raw plane
  BEFORE any levels normalisation. Cited by
  `docs/measured-q2m-dng.md` line 44, and by DNG 1.7.0.0
  § Chapter 6's own pipeline diagram. `src/develop.rs` today
  goes plane → normalise → geometry → orientation; SPEC-017
  inserts the opcode step between plane and normalise.

- **The Q2M plane extent is 5632 × 8424 u16 samples** (SPEC-012),
  with `active_area = 0 0 5632 8392` (SPEC-014). The applier
  runs inside `active_area` per Notes.

### Required before build handoff (the build cycle probes)

- **Read DNG 1.7.0.0 § Chapter 6 in full.** The 28-byte parse
  in `## Context` is right for the observed field, but the
  `Flags` semantics, the mandatory-vs-optional dispatch (AC6),
  the treatment of `DNGVersion`, and the exact FixBadPixels-
  Constant algorithm's tie-break rule (AC4) all come from the
  spec, not from this design. Cite the exact clause for each
  in the DEC-* if one is needed, or in the module's doc
  comment.

- **Measure the replaced-pixel count on each decodable frame.**
  Record in the handback: `L1021223.DNG: <n>`, `L1026016.DNG:
  <n>`, `L1026192.DNG: <n>`. These are the first three real
  data points on Q2M dead-pixel density.

### Required in build (pre-registered)

- **The `Opcode` enum's initial shape** (see "Where the opcode
  module lives" above):

  ```rust
  pub enum Opcode {
      FixBadPixelsConstant { constant: u32, bayer_phase: u32 },
      Unknown { id: u32, flags: u32, params: Vec<u8> },
  }
  ```

  `bayer_phase` is present because AC1 asserts round-trip; not
  branched on in the monochrome case. `Unknown` carries `params`
  as owned bytes because a subsequent SPEC-018 handoff parses
  them into WarpRect — the parser hands them across
  intact.

- **The fuzz target's seed set.** Four seeds:
  1. The 28-byte real payload.
  2. Truncated header: `00 00 00 01 00 00 00 04` — count says
     "one opcode" but no opcode body follows.
  3. Unknown opcode with `Flags = 0` (mandatory):
     `00 00 00 01 00 00 00 63 01 00 00 00 00 00 00 00
      00 00 00 04 DE AD BE EF`.
  4. Length overflow: a `VariableLengthBytes` field of
     `0xFFFFFFFF` after a valid header. The parser must NOT
     allocate 4 GB.

- **The tier-A hex fixture format.** `tests/oracle-fixtures/
  opcodelist1-q2m.hex` — one hex byte per line, or a raw
  binary file the test reads with `include_bytes!`. State
  which; do not silently pick.

### Not measured, and why

- **The exact dead-pixel LOCATIONS in each Q2M frame.**
  Interesting data but not required for the applier's
  correctness — the algorithm operates on pixel values, not
  coordinates. If AC5's counts turn out to vary meaningfully
  frame-to-frame (they should not, on the same sensor), that
  is a signal worth investigating.

- **Whether OpcodeList2 also exists on any Q2M frame.**
  `docs/measured-q2m-dng.md` says both OpcodeList1 and
  OpcodeList3 are present; OpcodeList2 is empty. This spec
  does not enumerate it. If a future frame carries an
  OpcodeList2, that camera's spec adds the branch.

## Follow-ups

*Appended during **ship**. Every `FU-N` / `SB-N` raised across this
spec's cycles, with its disposition (§15). No follow-up crosses this
ship undecided.*

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
