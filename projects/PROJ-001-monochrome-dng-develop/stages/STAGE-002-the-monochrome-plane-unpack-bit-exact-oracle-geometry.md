---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-002                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: 2026-09-06

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Proves the pixel half of the thesis, and proves it EXACTLY: our decoded sensor plane is byte-identical to an independent implementation's.
  delivers:
    - "Packed 14-bit → u16 unpack of the full sensor plane"
    - "A bit-exact plane oracle against dnglab"
    - "Black/white normalization and the three-stage geometry"
  explicitly_does_not:
    - "Opcodes — bad pixels and lens warp are STAGE-003"
    - "Tone curve or any output encoding"
    - "Demosaic — there is no CFA in a Monochrom file"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)
orchestration_cost:
  sessions:
    - tokens_total: 84200000
      estimated_usd: 214.20
      recorded_at: 2026-09-06
      notes: "Stage grain, not split across specs (DEC-013 section 5). One orchestrator session covering SPEC-014's reconciliation through SPEC-015's design, three handoffs, four reconciliations and this close - all of it STAGE-002 work, so the whole session attributes here. Measured floor 70,231,231 deduped by message.id from the orchestrator's own transcript (e078417d-f832-4765-bc7b-2b8493e01419.jsonl, identified by scratchpad-dir uuid), 260 unique assistant turns, all claude-opus-5; 96.9% cache-read. Priced per-component at published Opus rates ($15/$75/$30-write/$1.50-read) = $178.50, then both figures rounded up 20% to cover the turns writing this close - the same uplift this stage's handoffs required of every delegated session, measured there at 9.9%, 15.4% and 19.2% low. NOTE THE RATIO: orchestration ran ~84.2M against 187.0M of delegated spec cost across SPEC-012 through SPEC-015, so roughly 31% of this stage's total spend is orchestration that no spec would have recorded."
---

# STAGE-002: The monochrome plane: unpack, bit-exact oracle, geometry

## What This Stage Is

When this stage ships, `irradiance` produces a correct linear monochrome
sensor plane from a Q2M DNG — and proves it, because the plane's MD5 equals what
`dnglab analyze --raw-checksum` reports. Levels are normalized and the three-stage
crop and orientation are applied. This is where the project stops being
speculative.

**Estimated effort: 15–25 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

The oracle contract is already verified (`docs/oracle-contract.md`), so this
stage can be built against an exact pass/fail from its first commit — no judgment,
no tolerance, no tuning loop. That makes it unusually good delegated work.

It depends on STAGE-001 only for geometry and levels, and everything after it
depends on the plane being right, so any ambiguity here is worth paying for now.

## Success Criteria

- MD5 of our full-frame u16 plane equals `dnglab analyze --raw-checksum` on every tier-B file
- The unpack asserts `width × height × 14 / 8 == StripByteCounts` and fails loudly if not
- The oracle goes **red** on an injected off-by-one in the bit unpacker
- Geometry is asserted numerically: 8424×5632 → ActiveArea → DefaultCrop 8368×5584 → Rotate 90 CW
- Peak memory for a 47 MP decode is measured and recorded, not assumed

## Scope

### In scope
- Locating the full-resolution sensor IFD (⚠ *not necessarily a SubIFD* — `K3III.PEF` carries its plane in `IFD0`)'s strip and reading it
- Packed 14-bit → u16 unpack (zero-extended, NOT scaled to full 16-bit range)
- Bit-exact plane oracle harness
- BlackLevel subtraction and WhiteLevel normalization
- ActiveArea → DefaultCrop → Orientation

### Explicitly out of scope
- Tiled strip layouts — the Q2M is a single strip; tiles arrive with other cameras
- Compressed data of any kind (lossless JPEG is a later project)
- Opcodes (STAGE-003)

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-002` to promote these outlines into real specs.

- [x] SPEC-009 (shipped on 2026-09-04) [S] Pin the Structure-class membership, table-driven over all eleven tags
- [x] SPEC-012 (shipped on 2026-09-04) [M] Strip location and sample unpack, two paths per DEC-008
- [x] SPEC-013 (shipped on 2026-09-05) [S] Bit-exact plane oracle against dnglab raw-checksum, with its red-proof
- [x] SPEC-014 (shipped on 2026-09-05) [M] Level normalization, ActiveArea to DefaultCrop, and orientation
- [x] SPEC-015 (shipped on 2026-09-06) [S] Analytic levels and geometry oracle

⚠ **`SPEC-010` and `SPEC-011` were framed against this stage and have been MOVED
to `STAGE-005`.** Both are STAGE-001 debt, not plane work; leaving them here made
this stage eight specs and blurred what it is for. Neither blocks the plane.

**Count:** 5 shipped / 0 active / 0 pending

## Design Notes

### Per-spec context — the detail deliberately kept OUT of the backlog titles

`just frame-stage` derives each spec's **filename** from its backlog summary, so
summaries stay short and the constraints live here.

**Pin the Structure-class membership (`SPEC-009`) — FIRST, before the unpack.**
Carries `SPEC-008/FU-1`, `FU-2`, `FU-3`, `FU-5`. ⚠ Measured 2026-08-21 (mutation
asserted applied by `diff`, suite summed across all five targets, tree restored
byte-identical): `is_structural_tag()` (`src/ifd.rs:188-203`) has **eleven**
memberships and exactly **one** — `TAG_SUB_IFDS` — is enforced by any test.
Deleting the other ten leaves **66/66 green**. The hazard is *this stage's*:
`Compression` encoded `RATIONAL 2/2` reads `1`, `require_uncompressed()` passes,
and the unpack reads **JPEG bytes as raw samples** — a wrong image from a file
that parsed cleanly. ⚠ The fixing test must **not** derive its table from
`is_structural_tag()`; a test that reads the list it checks is a tautology, and
deleting a tag would delete its own coverage.

**Strip location and sample unpack.** ⚠ **TWO PATHS, per `DEC-008`** — sub-byte
samples (14-bit) are an MSB-first bit stream; byte-aligned samples (16-bit) are
plain integers in the **file's** byte order. `SPIKE-002` found the single-path
version produced a **byte-swapped plane** on a 16-bit file. Keep the
`max > WhiteLevel` assertion — that is what caught it. Both paths need their own
fuzz coverage; one target exercising only 14-bit recreates the exact blind spot.
The corpus holds both shapes: `L1021223.DNG` is 14-bit, `L1000622.DNG` is 16-bit.

**Bit-exact plane oracle.** `docs/oracle-contract.md` has the contract, verified
on two frames: `--raw-checksum` is the **MD5 of the uncropped `u16` plane, native
little-endian, 14-bit values zero-extended, no black subtraction, no crop**. The
comparison attaches **before** the three-stage crop. Every corpus entry already
carries its `raw_checksum` in the manifest, so the oracle pins both the file and
the tool. ⚠ Single-sourced: matching `--raw-checksum` proves we match **rawler**,
not that we are correct — acceptable for the plane, and the layer-0 packing
arithmetic (`width × height × bits == StripByteCounts × 8`) is the independent
check to keep.

**Level normalization and geometry.** Three-stage crop:
`8424×5632 → ActiveArea → DefaultCrop 8368×5584 → Rotate 90 CW`. ⚠ `Orientation`
is **per-frame, not a camera constant** — `L1026016.DNG` reads `6` where its two
siblings read `1`. That single file is why `unrun-docs-carry-errors` exists; keep
it in every geometry test.

**Analytic levels and geometry oracle (`DEC-004`).** ⚠ `SPIKE-001` measured that
the plane checksum is **structurally blind** to a levels error and the develop
oracle misses one up to **+256 (50%)**. Without this spec, levels ship with **no**
oracle coverage. Assert normalization maps `BlackLevel→0` and `WhiteLevel→1` on
tags **read from the file**, plus crop dimensions and orientation on both a
rotated and an unrotated frame.

### Two things STAGE-001 learned that bind this stage

1. **`just lint-ci` before every push, and read CI.** `PATCH-001` found the
   panic-free gate had been dark for 17 consecutive runs across the whole of
   STAGE-001 while every verify honestly reported "ten gates green" — locally.
   `constraints.yaml` now says a job that exists and has never passed is a
   deleted job, and claiming the constraint requires having **observed** the job
   green on the SHA claimed for.
2. **The three rules codified into AGENTS.md §16** — the writing rule for
   measurements, assert-your-match-count, and a gate must fail through its own
   `die`. This stage writes a bit unpacker and two oracles; all three apply
   directly.



**The comparison attaches to the UNCROPPED full frame.** `--raw-checksum`
hashes the 8424×5632 plane before ActiveArea and before DefaultCrop, in native
little-endian, with 14-bit values zero-extended. Decode → hash → compare one
string; crop afterwards. Full contract and the three wrong guesses that preceded
it are in `docs/oracle-contract.md`.

**What bit-exact does and does not prove.** It proves we match rawler. Because decompression is
deterministic and rawler is the de facto reference, that IS the goal at this layer — but say so
rather than letting "bit-exact" imply more than it does. The layer-0 packing arithmetic
(`w x h x 14 / 8 == StripByteCounts`) is the one check here that is independent of any other
implementation; keep it even though the checksum subsumes it in practice.

Memory is a live design constraint, not a footnote: 47.4 MP at f32 is ~190 MB per
plane before anything else exists.

**Stopping point B:** a correct linear plane, proven bit-exact against an
independent implementation.

## Dependencies

### Depends on
- STAGE-001 — geometry, levels and the SubIFD location
- External: `dnglab` (tool)

### Enables
- STAGE-003 — opcodes operate on this plane

## Stage-Level Reflection

*Filled in at ship, 2026-09-06.*

### Success Criteria — all five met, checked against what shipped

| criterion | verdict |
|---|---|
| MD5 of our full-frame `u16` plane equals `dnglab analyze --raw-checksum` on every tier-B file | ✅ **on every *decodable* tier-B file.** `plane_md5_matches_the_pinned_raw_checksum`, `SPEC-013`. The other three of the seven are `Compression 7`/`65535` and are rejected before any strip read — the criterion as written implies seven and means four. Worth reading exactly. |
| The unpack asserts `w × h × 14 / 8 == StripByteCounts` and fails loudly | ✅ `layer0_arithmetic_is_enforced`, `SPEC-012` |
| The oracle goes **red** on an injected off-by-one in the bit unpacker | ✅ `an_injected_unpacker_fault_turns_the_oracle_red`, watched red by three separate sessions: `honest=cb653b5b… mutant=59b032fe…` |
| Geometry asserted numerically: `8424×5632 → ActiveArea → DefaultCrop 8368×5584 → Rotate 90 CW` | ✅ `SPEC-014`, on both a rotated and an unrotated frame — `Orientation` is per-frame, and `L1026016.DNG` is why |
| Peak memory for a 47 MP decode measured, not assumed | ✅ 182,435,840 B decode (`SPEC-012`); ≈275,906,560 B develop (`SPEC-014`, corrected at `FU-5` — the measurement is page-granular, so it is `≈`, not `=`) |

### value_contribution — delivered, with one honest correction

All three `delivers` items shipped. The `advances` claim — *"proves it EXACTLY:
our decoded sensor plane is byte-identical to an independent implementation's"* —
is **true and narrower than it reads**. `SPEC-013` proves we match **rawler**, not
that we are correct; the stage's own Design Notes said so at framing and the
claim above did not inherit the qualifier. The genuinely implementation-
independent checks are the layer-0 packing arithmetic and, added late,
`SPEC-015`'s analytic oracle.

No spec's `value_link` failed to deliver what it claimed.

### Three sentences

**Built vs planned:** five specs against a planned backlog of five, and the plan
held — but `SPEC-015` was not in the original stage framing at all; it exists
because `DEC-004` measured, mid-stage, that both existing oracles are structurally
blind to the levels work `SPEC-014` was about to ship.

**Harder or easier than expected:** materially harder at the end — `SPEC-014`
came in at 88.8M tokens against a 26M estimate (**3.42×**) and `SPEC-015` at 98.2M
against 60M (**1.64×**), and the overrun in both cases was not the build but the
cycles after it: verify rounds that found real holes, then punch-list rounds to
close them.

**Emergent integration behavior:** the two oracles turned out to be blind in
*complementary* ways that only appeared when composed — the plane checksum is
bit-identical under a levels error, the develop oracle scores 95.62 (passing) on
the same fault, and the analytic oracle that covers both is itself position-blind
by construction, so the stage ends with three oracles whose union is strong and
whose individual coverage maps had to be written down to be trusted.

### The reflection fields

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** A correct
  linear plane, proven bit-exact against an independent implementation
  (stopping point B), plus the levels and geometry that turn it into an image
  and an analytic oracle over that arithmetic.
- **How many specs did it actually take?** **Five** — `SPEC-009`, `012`, `013`,
  `014`, `015` — against a backlog that also listed `SPEC-010` and `SPEC-011`,
  both moved to `STAGE-005` mid-stage as STAGE-001 debt rather than plane work.
  `SPEC-015` was added mid-stage by `DEC-004`.
- **How many outlines survived unchanged?** **Four of five.** `SPEC-015`'s
  outline was a bare template at design time and was written from scratch; the
  other four were framed at stage-open and shipped against their original scope.
- **What changed between starting and shipping?** The stage started as *"unpack
  the plane and prove it byte-exact"* and ended having also discovered that
  byte-exactness proves less than it sounds like — which is why it ships with an
  analytic oracle nobody planned for.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **`unrun-docs-carry-errors` reaches its bar in this stage and is codified**
    (N=2 → N=5, three of them here). See below.
  - **A gate can succeed mutely.** `just validate` reported "valid required
    front-matter" on two files no YAML parser could read, one of which shipped
    and was archived. Filed as `handback-sync-truncates-multi-line-scalars`; the
    generalising fix is making `validate` parse rather than grep.
  - **The gate count is undefined** — `the-gate-count-is-not-defined-anywhere`,
    filed at bar 3. One named list in `AGENTS.md` §6 ends it; deliberately not
    changed inside a spec's ship round.

### Signals owned by this close — every one touched, no silent carry

| signal | action |
|---|---|
| `unrun-docs-carry-errors` | **ACCEPT AND CODIFY.** Was `watch` at N=2 (`--srgb` called a TIFF; `Orientation` called a camera constant). This stage supplied **three more, all in `SPEC-014`**: `FU-1` (`manifest.toml` labelled `DefaultCropOrigin` as `ActiveArea`), `FU-6` (`FU-1`'s **own correction** asserted a non-zero `ActiveArea` where every Q2M frame is `(0,0)`), and `FU-9` (`conformance-matrix.md` carried the identical wrong fact a **third** time). **N=5, past the N=3 bar**, and instance 4 is the sharpest: a correction written without running is as wrong as the thing it corrects. Rule landed in `AGENTS.md` §16 as lesson 4. |
| `attribute-text-inside-doc-comments` | Already `codified`. No new instances this stage. Touched, `last_touched` bumped. |
| `measurement-over-generalised` | Already `codified`, and it **fired repeatedly** here — `SPEC-014/AC7`'s `=` that should have been `≈`, `SPEC-015/AC2`'s floor stated without its content-dependence, and my own "the positional coverage exists" which was true only for faults visible at ≤8 px. Stays codified; evidence extended. |
| `a-gate-that-fails-mutely-is-a-gate-that-never-ran` | Already `codified`. This stage found its **mirror image** — a gate that *succeeds* mutely (`just validate` on unparseable YAML). Recorded as evidence on the new `handback-sync-truncates-multi-line-scalars` rather than reopening a codified lesson. |

### Follow-up work — where it goes

- **`SPEC-016`** ("the harness stops claiming what it has not checked") is framed
  in `STAGE-005` and is now unusually well supplied: `SPEC-015/FU-4`, `FU-5` and
  `FU-12` (a gate reporting success without checking), `FU-10` (CI running **0/7**
  corpus files while green), and `FU-11` (a count no gate can verify) are four
  independent instances of its thesis, each with measurements attached. It should
  be designed next.
- **`SPEC-011`** (lint the fuzz crate) remains unblocked in `STAGE-005`.
- **No new stage.** `STAGE-003` (opcodes and output) is unchanged by anything
  learned here.
- **One residual carried deliberately:** `SPEC-015/FU-10`'s size-gate closure is
  bounded at 1024 px, and `FU-6`'s wrong-permutation blind spot is inherent to
  `DEC-020`. Both are recorded in that decision; neither is a defect, and neither
  should be rediscovered as one.

