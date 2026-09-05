---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-012
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-sonnet-5        # CORRECTED — build actually ran on Sonnet 5, not the opus dispatch hint.
  created_at: 2026-09-04

references:
  decisions: [DEC-002, DEC-008, DEC-012, DEC-016]                    # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application, provenance-recorded-per-algorithm]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-003, SPEC-009, SPEC-013]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: null

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
  tokens_estimate: 28000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-04
      notes: "main-loop, not separately metered (AGENTS.md §4). Design cycle did a real BYTE-LEVEL probe (§15 rule 4) rather than describing one: read the strip head of both corpus shapes, hand-unpacked 14-bit MSB-first and 16-bit little-endian, and cross-checked BOTH against dnglab --raw-pixel's own plane. They agree EXACTLY — [746,725,711,752,...] and [4761,4591,4622,4363,...] — so the spec can pin first-sample values as measured fact and the builder gets a first-pixel checkpoint instead of an opaque whole-plane MD5 mismatch. Also measured the WRONG paths (43019 and 39186, both impossible against WhiteLevel 16383), which is what AC3 asserts. Confirmed the decodable set is 4 of 7 — the other three are Compression 7 or 65535. Surfaced the allocation question (94,887,936 bytes of plane) as a DEC the build must write, recommending unpack_into as the primitive since DEC-002 is unresolved, offered as input rather than as the answer. ALSO FIXED a scaffolding error of my own: all four STAGE-002 specs framed by just frame-stage carried '(not yet written)' in their filenames and titles, inherited from backlog summaries I wrote with that prefix. Renamed before more artifacts inherited it."
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 29580529
      estimated_usd: 89
      duration_minutes: 35
      recorded_at: 2026-09-04
      notes: "Real number, deduped by message.id, summed from this session's own transcript (~/.claude/projects/<slug>/<session-id>.jsonl usage objects — 104 distinct messages: 208 input + 130,788 output + 350,561 cache-creation + 29,098,972 cache-read = 29,580,529). estimated_usd is a DELIBERATE OVERESTIMATE per AGENTS.md §4 (tokens_total x list rate, no cache discount): ~$3/MTok assumed Sonnet list rate x 29.58M ~= $89; a cache-aware accounting (98% of tokens were cache reads at a fraction of that rate) would land closer to $12. HANDOFF-028 return criterion 7 applies: to_agent corrected above, handback-sync NOT run by this session, PR NOT opened by this session — both left for the orchestrator per that instruction."

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-012: Strip location and sample unpack, two paths per DEC-008

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-012]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

**This is where the project stops being speculative.** Eight specs have read
metadata; this one produces **pixels**. `SPEC-009` shipped the day before this was
designed and is the reason it can be trusted: every Structure-class tag is now
load-bearing, so `require_uncompressed()` cannot be walked past by a
`RATIONAL 2/2` `Compression`, and the dimensions this unpacker reads cannot
silently be the wrong ones.

`SPIKE-001` achieved a bit-exact 14-bit decode on its first attempt. Its
unpacker took `bits` as a parameter and, across every frame it ever saw, that
parameter was **always 14** — so the two cases `DEC-008` names were
indistinguishable. `SPIKE-002` then ran it against a 16-bit body and got a
**byte-swapped plane**: wrong in a way that still decodes, still has the right
length, and **still passes the layer-0 arithmetic check**. That is the failure
this spec is shaped around.

## Goal

Locate the sensor plane's strip and unpack it into a linear `u16` plane, on
**both** of `DEC-008`'s paths, with the layer-0 arithmetic asserted and the
first samples pinned against the oracle.

**Not** the MD5 oracle — that is `SPEC-013`. This spec must be verifiable
*without* it, which is what the measured first samples below are for.

## Inputs

- **Files to read:** `src/ifd.rs` — `Sensor` (`strip_offsets`, `strip_byte_counts`,
  `bits_per_sample`, `require_uncompressed`, `packed_bits`), `Container::payload`
  for the bounds-checked slice idiom; `docs/oracle-contract.md`; `DEC-008` in full
- **Decisions:** `DEC-008` (the two paths — read the Context, not just the
  Decision), `DEC-002` (**`status: proposed`** — the allocation question below is
  gated on it), `DEC-012`
- **Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`

## Outputs

- **Files modified:** `src/` gains the unpacker — a new module is fine
- **New fuzz target** or an extension of `ifd`: the unpacker is a **new input
  surface** over attacker-controlled `bits`, `width`, `height` and strip bounds.
  §12 bar 2 applies. ⚠ **A target that only ever drives 14-bit recreates
  `SPIKE-001`'s exact blind spot** — it must reach both paths.
- **A `DEC-*` for the allocation shape** (see below). Required either way.
- **A provenance-ledger row** — this is a new algorithm, class 1 (specification):
  TIFF 6.0 §Compression=1 plus `DEC-008`. ⚠ `SPIKE-001`'s decoder is **discarded
  and must not be consulted**; re-derive from the spec.

## Acceptance Criteria

- [x] **AC1 — the 14-bit path is bit-exact on its first samples.** Unpacking
      `L1021223.DNG` yields `[746, 725, 711, 752, 646, 705, 772, 686]` as
      samples 0–7. **Measured against `dnglab`'s own plane** — see below.
      Confirmed: `tests/plane_unpack.rs::unpacks_fourteen_bit_msb_first_samples`
      against the real file, and `irr unpack` reproduces the same eight values.
- [x] **AC2 — the 16-bit path is bit-exact on its first samples**, and is **not**
      a bit stream. `L1000622.DNG` yields `[4761, 4591, 4622, 4363, 4542, 4383,
      4608, 4286]`. Confirmed:
      `tests/plane_unpack.rs::unpacks_sixteen_bit_in_file_byte_order` against
      the real file, and `irr unpack` reproduces the same eight values.
- [x] **AC3 — each path FAILS on the other's data**, asserted with the measured
      wrong values, not merely "differs". Reading the 14-bit strip as 16-bit LE
      gives `43019` for sample 0; reading the 16-bit strip as big-endian gives
      `39186`. **Both exceed `WhiteLevel 16383` and are therefore impossible** —
      that is the assertion, and it is the one that caught `SPIKE-002`. Confirmed:
      `tests/plane_unpack.rs::each_path_produces_impossible_values_on_the_others_data`
      (hand-built fixtures carrying the real strip bytes, no `WhiteLevel` tag so
      the wrong values decode without tripping AC4, and are asserted directly).
- [x] **AC4 — `max > WhiteLevel` is asserted on every decode**, as a loud error,
      not a debug assert. It is the check that found the byte-swap when the
      length, the arithmetic and the decode all looked right. Confirmed:
      `tests/plane_unpack.rs::a_plane_whose_max_exceeds_white_level_is_an_error`
      — same fixture as AC3's first case, now with `WhiteLevel: Some(16383)`,
      returns `Error::SampleExceedsWhiteLevel { index: 0, sample: 43019,
      white_level: 16383 }`. Unconditional whenever `sensor.white_level` is
      present — not behind `cfg(debug_assertions)` (`src/plane.rs::unpack_into`).
      Real files (AC1/AC2) are the negative control: both carry a real
      `WhiteLevel` and decode clean, proving the check does not misfire on
      honest data (`oracle-must-be-shown-red`'s negative-control half).
- [x] **AC5 — layer-0 holds and is enforced**:
      `width × height × bits == StripByteCounts × 8`. Measured:
      `8424×5632×14 = 664,215,552` and `5216×3472×16 = 289,759,232`, both equal
      to their `StripByteCounts × 8`. Confirmed:
      `tests/plane_unpack.rs::layer0_arithmetic_is_enforced` — tier A (a
      hand-built mismatch: `5×2×14 = 140` bits declared against a 14-byte/112-bit
      strip returns `Error::PackedSizeMismatch { expected_bits: 140,
      strip_bits: 112 }`) **and** tier B (asserts `Sensor::packed_bits()` equals
      `StripByteCounts × 8` on both real decodable files).
- [x] **AC6 — the three compressed files are rejected cleanly**, by typed error,
      with no allocation of a plane: `M2462362.DNG` and `K3III.DNG`
      (`Compression 7`), `K3III.PEF` (`65535`). The decodable set is **4 of 7**.
      Confirmed: `tests/plane_unpack.rs::compressed_files_are_rejected_without_decoding`
      calls `unpack_into` with an **empty** `dst: [u16; 0]` for all three —
      proving `Error::UnsupportedCompression` fires before the length check
      (`PlaneBufferWrongLength`) would, i.e. before any plane-sized buffer
      would even need to exist. `irr unpack` on `K3III.DNG` confirms the same
      end to end (`compression 7 is not supported by this library`, no plane
      printed).
- [x] **AC7 — panic-free on hostile input.** Truncated strips, `StripByteCounts`
      larger than the file, zero/absurd dimensions, `bits` outside {8,12,14,16}.
      All typed errors. The fuzz target reaches **both** paths. Confirmed:
      `tests/plane_unpack.rs::hostile_strip_bounds_do_not_panic` (four
      sub-cases: truncated strip → `Error::Truncated`; zero dimensions → `Ok`
      on an empty plane; `width = height = u32::MAX` → a typed `Err` via the
      buffer-length check; `bits = 10` → `Error::UnsupportedBitDepth`). Fuzzed
      `fuzz/fuzz_targets/plane.rs` — see AC9 below for run counts and how both
      paths are known to be reached.
- [x] **AC8 — peak memory for a 47 MP decode is MEASURED and recorded**, not
      assumed (STAGE-002 success criterion 5). `8424 × 5632 × 2 = 94,887,936`
      bytes of plane alone; state what the decode actually peaks at and where
      the rest goes. **Measured**, `/usr/bin/time -l` on macOS (darwin),
      `target/release/irr unpack`, this machine, this build:
      `L1021223.DNG` (14-bit, 47 MP) — **182,435,840 bytes (174 MiB) maximum
      resident set size**. Accounted for: the ~85.8 MB input file read whole
      into a `Vec<u8>` by `irr` (I/O the library never does) + the 94,887,936-byte
      output plane `irr` allocates as `unpack_into`'s caller (`DEC-016` —
      the library itself allocates nothing) ≈ 180.7 MB, matching the measured
      182.4 MB peak within run-to-run noise. `L1000622.DNG` (16-bit, 18 MP,
      smaller input) peaked at 74,399,744 bytes for comparison. Method is
      necessarily one-machine, one-build evidence (§16 confidence discipline) —
      re-measure before relying on it elsewhere.
- [x] **AC9 — eleven gates + `just lint-ci`**, and **CI observed green** on the
      shipping SHA. All eleven local gates + `lint-ci` green (see Handback for
      the full list, including a real local/CI clippy-version gap found and
      fixed by `lint-ci` itself). Pushed `feat/spec-012-strip-location-and-sample-unpack`
      to `origin` and **observed CI green** on `731a89171bfff9001af692fd0dfc291968eceafd` —
      all nine CI jobs passed (clippy, fmt, license policy x2, test, MSRV,
      lint-policy red-proof, panic-free policy, cost-capture audit):
      https://github.com/jysf/irradiance/actions/runs/33932904592

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each exists per-target and
**sum across all six targets**.

- `unpacks_fourteen_bit_msb_first_samples` — AC1, tier B
- `unpacks_sixteen_bit_in_file_byte_order` — AC2, tier B
- `each_path_produces_impossible_values_on_the_others_data` — AC3, tier A
  (hand-built fixtures; the measured constants above make this runnable in CI)
- `a_plane_whose_max_exceeds_white_level_is_an_error` — AC4, tier A
- `layer0_arithmetic_is_enforced` — AC5, tier A + B
- `compressed_files_are_rejected_without_decoding` — AC6, tier B
- `hostile_strip_bounds_do_not_panic` — AC7, tier A

## Non-Goals

- **The MD5 plane oracle** — `SPEC-013`.
- **Levels, crop, orientation** — `SPEC-014`. This spec's output is the
  **uncropped, un-normalised** plane, which is exactly what `--raw-checksum`
  compares against.
- **Tiles.** The corpus is single-strip and the tests assert it; tiles arrive
  with other cameras.
- **Compressed data of any kind.** `AC6` rejects it; decoding it is PROJ-003.
- **Reading `SPIKE-001`'s decoder.** Discarded; re-derive from TIFF 6.0 + `DEC-008`.

## Implementation Context

> **Measured 2026-09-04** on the real corpus. The first-sample values were
> obtained two independent ways and **agree exactly** — hand-unpacked from the
> raw file's strip bytes, and read out of `dnglab analyze --raw-pixel`'s own
> plane. Reproduce before trusting.

### The two shapes the corpus actually holds

| | `L1021223.DNG` (Q2M) | `L1000622.DNG` (M Mono) |
|---|---|---|
| dimensions | 8424 × 5632 | 5216 × 3472 |
| `bits_per_sample` | **14** (sub-byte) | **16** (byte-aligned) |
| byte order | `II` | `II` |
| strip offset | 2,769,920 | 213,504 |
| `StripByteCounts` | 83,026,944 | 36,219,904 |
| layer-0 | 664,215,552 bits — **closes** | 289,759,232 bits — **closes** |
| `BlackLevel` / `WhiteLevel` | 512 / 16383 | 220 / 16383 |

### ⚠ The first samples, cross-checked against the oracle

**Q2M, 14-bit, MSB-first bit stream.** Strip head:
`0b a8 2d 50 b1 c2 f0 0a 18 2c 10 c1 02 ae 0b dc`

```
hand-unpacked MSB-first : [746, 725, 711, 752, 646, 705, 772, 686]
dnglab --raw-pixel      : [746, 725, 711, 752, 646, 705, 772, 686]   ← identical
```

**M Monochrom, 16-bit, file byte order.** Strip head:
`99 12 ef 11 0e 12 0b 11 be 11 1f 11 00 12 be 10`

```
read little-endian      : [4761, 4591, 4622, 4363, 4542, 4383, 4608, 4286]
dnglab --raw-pixel      : [4761, 4591, 4622, 4363, 4542, 4383, 4608, 4286]   ← identical
```

⚠ **`dnglab`'s PGM payload is BIG-endian** (PNM spec) while `--raw-checksum` is
native LE — the values above are the decoded samples, not raw bytes. The PGM
headers confirm the plane is **uncropped**: `P5 8424 5632 65535` and
`P5 5216 3472 65535`, matching `ImageWidth`/`ImageLength`, **not** `ActiveArea`
or `DefaultCropSize`.

### The wrong paths, measured — this is `AC3`

```
14-bit strip read as 16-bit LE  -> [43019, 20525, 49841, 2800, ...]
16-bit strip read as big-endian -> [39186, 61201,  3602, 2833, ...]
```

`43019` and `39186` both exceed `WhiteLevel 16383` and are therefore
**impossible**. That is why `AC4`'s assertion is the one that matters: the
byte-swapped plane had the right length, passed layer-0, and decoded without
error. Only the value range gave it away.

### ⚠ The design question this spec must answer with a `DEC`

`8424 × 5632 × 2 = 94,887,936` bytes for the plane alone, on top of an 86 MB
input. `library-not-application` says **the consumer opens the file and picks
the allocator**, and `DEC-002` (**proposed**, 0.72) proposes `no_std` + `alloc`
with `std` behind a default-on feature.

So the API shape is a real decision, not a detail:

- **`unpack_into(&self, dst: &mut [u16]) -> Result<(), Error>`** — caller owns
  the buffer, length checked against `width × height`. Needs no allocator at
  all, so it survives `DEC-002` whichever way that lands.
- **`unpack(&self) -> Result<Vec<u16>, Error>`** — convenient, and commits the
  library to allocating 95 MB on the caller's behalf.

**The orchestrator's read, offered as input and not as the answer:** ship
`unpack_into` as the primitive and let a `Vec`-returning convenience sit on top
of it later if wanted — a caller who cannot afford the allocation has no way to
opt out of the second shape, and `DEC-002` is unresolved. **Write the `DEC`
either way, including if you disagree.**

### Traps

- `require_uncompressed()` before anything else — `AC6`'s three files must never
  reach a decode path.
- Every offset and length from the file is attacker-controlled. Bounds-check
  through `checked_*` / `.get()`; the five-lint policy will reject the
  alternatives, and `just lint-no-allow` will reject an `#[allow]`.
- `just lint-ci`, not `just lint`, and **read CI**.
- Sum across **all six** targets; tier-B tests pass whether or not the corpus is
  present, and only `just test` names what is missing.

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
